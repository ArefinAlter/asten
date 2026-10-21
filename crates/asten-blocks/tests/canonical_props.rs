//! Property tests for canonical JSON fingerprints and the hash-chained log.

use asten_blocks::canonical::{canonical_json, fingerprint, hex, unhex};
use asten_blocks::events::{
    ChainError, EventPayload, EventRecord, next_record, next_record_after, verify,
};
use asten_blocks::patch::RichTextPatch;
use proptest::prelude::*;
use serde_json::{Map, Value};

const CASES: u32 = 256;

// ───────────────────────── generators ─────────────────────────

fn text() -> impl Strategy<Value = String> {
    prop_oneof![
        3 => "[a-z]{0,6}",
        1 => "[\\x00-\\x7f]{0,6}",
        1 => "\\PC{0,6}",
        1 => Just(String::new()),
    ]
}

fn leaf() -> impl Strategy<Value = Value> {
    prop_oneof![
        Just(Value::Null),
        any::<bool>().prop_map(Value::Bool),
        any::<i64>().prop_map(Value::from),
        any::<u64>().prop_map(Value::from),
        // Short dyadic decimals: serde_json without `float_roundtrip` may parse
        // an extreme float one ulp off, which is a parser property, not a
        // canonicalisation one.
        (-1_000_000..1_000_000i32, 0..8u32).prop_map(|(mantissa, shift)| Value::from(
            f64::from(mantissa) / f64::from(1u32 << shift)
        )),
        text().prop_map(Value::String),
    ]
}

fn value() -> impl Strategy<Value = Value> {
    leaf().prop_recursive(4, 48, 6, |inner| {
        prop_oneof![
            prop::collection::vec(inner.clone(), 0..6).prop_map(Value::Array),
            prop::collection::btree_map(text(), inner, 0..6)
                .prop_map(|map| Value::Object(map.into_iter().collect::<Map<_, _>>())),
        ]
    })
}

/// A tiny deterministic generator for shuffles and whitespace choices.
struct Lcg(u64);

impl Lcg {
    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        self.0 >> 33
    }

    fn space(&mut self) -> &'static str {
        match self.next() % 5 {
            0 => "",
            1 => " ",
            2 => "\n",
            3 => "\t",
            _ => " \r\n  ",
        }
    }
}

/// Write `value` as JSON text with object keys in a seeded random order and
/// random insignificant whitespace, as another serialiser might.
fn scrambled_json(value: &Value, out: &mut String, rng: &mut Lcg) {
    match value {
        Value::Object(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            for index in (1..keys.len()).rev() {
                let swap = (rng.next() as usize) % (index + 1);
                keys.swap(index, swap);
            }
            out.push('{');
            out.push_str(rng.space());
            for (position, key) in keys.iter().enumerate() {
                if position > 0 {
                    out.push(',');
                    out.push_str(rng.space());
                }
                out.push_str(&serde_json::to_string(key).unwrap());
                out.push_str(rng.space());
                out.push(':');
                out.push_str(rng.space());
                scrambled_json(&map[*key], out, rng);
            }
            out.push_str(rng.space());
            out.push('}');
        }
        Value::Array(items) => {
            out.push('[');
            for (position, item) in items.iter().enumerate() {
                if position > 0 {
                    out.push(',');
                }
                out.push_str(rng.space());
                scrambled_json(item, out, rng);
                out.push_str(rng.space());
            }
            out.push(']');
        }
        scalar => out.push_str(&serde_json::to_string(scalar).unwrap()),
    }
}

fn payload() -> impl Strategy<Value = EventPayload> {
    prop_oneof![
        (text(), text(), prop::option::of(text()), text(), value()).prop_map(
            |(node_id, node_type, parent_id, pos, fields)| EventPayload::NodeCreated {
                node_id,
                node_type,
                parent_id,
                pos,
                fields,
            }
        ),
        (text(), prop::option::of(text()), text()).prop_map(|(node_id, new_parent_id, new_pos)| {
            EventPayload::NodeMoved {
                node_id,
                new_parent_id,
                new_pos,
            }
        }),
        text().prop_map(|node_id| EventPayload::NodeDeleted { node_id }),
        text().prop_map(|node_id| EventPayload::NodeRestored { node_id }),
        (text(), text(), value()).prop_map(|(node_id, field, value)| EventPayload::FieldEdited {
            node_id,
            field,
            value,
        }),
        (text(), text(), text(), prop::collection::vec(value(), 0..3)).prop_map(
            |(node_id, before, after, styles)| EventPayload::RichTextPatched {
                node_id,
                patch: RichTextPatch {
                    version: 1,
                    before,
                    after,
                    styles,
                    edits: Vec::new(),
                },
            }
        ),
        (text(), value()).prop_map(|(field, value)| EventPayload::MetaEdited { field, value }),
    ]
}

fn chain(payloads: &[EventPayload], actors: &[String]) -> Vec<EventRecord> {
    let mut records: Vec<EventRecord> = Vec::new();
    for (index, payload) in payloads.iter().enumerate() {
        let actor = &actors[index % actors.len()];
        let record = next_record(
            records.last(),
            payload.clone(),
            actor,
            "2026-10-21T00:00:00Z",
        );
        records.push(record);
    }
    records
}

fn flip_hex(text: &str) -> String {
    let mut bytes = unhex(text).unwrap_or_else(|| vec![0; 32]);
    bytes[0] ^= 0x01;
    hex(&bytes)
}

#[derive(Clone, Copy, Debug)]
enum Tamper {
    Actor,
    Payload,
    ContentHash,
    PrevChainHash,
    ChainHash,
    Seq,
    /// `id` and `at` are not hashed, so changing them must still verify.
    Unhashed,
}

fn tamper() -> impl Strategy<Value = Tamper> {
    prop_oneof![
        Just(Tamper::Actor),
        Just(Tamper::Payload),
        Just(Tamper::ContentHash),
        Just(Tamper::PrevChainHash),
        Just(Tamper::ChainHash),
        Just(Tamper::Seq),
        Just(Tamper::Unhashed),
    ]
}

// ───────────────────────── canonical JSON ─────────────────────────

proptest! {
    #![proptest_config(ProptestConfig { cases: CASES, .. ProptestConfig::default() })]

    /// Key order and whitespace in the source text never change the fingerprint,
    /// and canonical bytes parse back to the same value.
    #[test]
    fn fingerprint_ignores_key_order_and_whitespace(value in value(), seed in any::<u64>()) {
        let mut rng = Lcg(seed);
        let mut scrambled = String::new();
        scrambled_json(&value, &mut scrambled, &mut rng);
        let parsed: Value = serde_json::from_str(&scrambled).expect("scrambled text is JSON");
        prop_assert_eq!(&parsed, &value);
        prop_assert_eq!(fingerprint(&parsed), fingerprint(&value));
        prop_assert_eq!(canonical_json(&parsed), canonical_json(&value));

        let canonical = canonical_json(&value);
        let reparsed: Value = serde_json::from_slice(&canonical).expect("canonical bytes are JSON");
        prop_assert_eq!(reparsed, value.clone());
        prop_assert_eq!(fingerprint(&value).len(), 64);
        prop_assert_eq!(unhex(&fingerprint(&value)).map(|bytes| bytes.len()), Some(32));
    }

    /// Canonical bytes carry no whitespace outside strings: the only bytes
    /// between tokens are structural.
    #[test]
    fn canonical_json_is_compact(value in value()) {
        let canonical = canonical_json(&value);
        let mut in_string = false;
        let mut escaped = false;
        for byte in &canonical {
            if in_string {
                if escaped {
                    escaped = false;
                } else if *byte == b'\\' {
                    escaped = true;
                } else if *byte == b'"' {
                    in_string = false;
                }
            } else if *byte == b'"' {
                in_string = true;
            } else {
                prop_assert!(!byte.is_ascii_whitespace(), "whitespace outside a string");
            }
        }
    }

    #[test]
    fn hex_round_trips(bytes in prop::collection::vec(any::<u8>(), 0..40)) {
        let text = hex(&bytes);
        prop_assert_eq!(unhex(&text), Some(bytes.clone()));
        prop_assert_eq!(unhex(&text.to_uppercase()), Some(bytes));
    }
}

// ───────────────────────── the chain ─────────────────────────

proptest! {
    #![proptest_config(ProptestConfig { cases: CASES, .. ProptestConfig::default() })]

    /// Records chained with `next_record` verify, `next_record_after` agrees
    /// with it, and sequence numbers are gap-free.
    #[test]
    fn chains_verify(
        payloads in prop::collection::vec(payload(), 1..8),
        actors in prop::collection::vec(text(), 1..3),
    ) {
        let records = chain(&payloads, &actors);
        prop_assert_eq!(verify(&records), Ok(()));
        for (index, record) in records.iter().enumerate() {
            prop_assert_eq!(record.seq, index as i64 + 1);
            let expected_prev = if index == 0 {
                asten_blocks::events::ZERO_HASH_HEX.to_owned()
            } else {
                records[index - 1].chain_hash.clone()
            };
            prop_assert_eq!(&record.prev_chain_hash, &expected_prev);
        }
        // Every prefix is a valid chain too.
        for len in 0..=records.len() {
            prop_assert_eq!(verify(&records[..len]), Ok(()));
        }
        // Building from a known head gives the same hashes.
        let head = records.last().unwrap();
        let by_record = next_record(Some(head), payloads[0].clone(), &actors[0], "t");
        let by_head = next_record_after(head.seq, &head.chain_hash, payloads[0].clone(), &actors[0], "t");
        prop_assert_eq!(by_record.seq, by_head.seq);
        prop_assert_eq!(by_record.content_hash, by_head.content_hash);
        prop_assert_eq!(by_record.prev_chain_hash, by_head.prev_chain_hash);
        prop_assert_eq!(by_record.chain_hash, by_head.chain_hash);
    }

    /// Any change to a hashed field of any record fails verification at that
    /// record; changes to unhashed fields do not.
    #[test]
    fn tampering_is_detected(
        payloads in prop::collection::vec(payload(), 1..8),
        actors in prop::collection::vec(text(), 1..3),
        which in any::<prop::sample::Index>(),
        how in tamper(),
        other in payload(),
    ) {
        let mut records = chain(&payloads, &actors);
        let index = which.index(records.len());
        let seq = records[index].seq;
        let record = &mut records[index];
        let expected = match how {
            Tamper::Actor => {
                record.actor.push('!');
                Some(ChainError::Content { seq })
            }
            Tamper::Payload => {
                if record.payload == other {
                    return Ok(());
                }
                record.payload = other;
                Some(ChainError::Content { seq })
            }
            Tamper::ContentHash => {
                record.content_hash = flip_hex(&record.content_hash);
                Some(ChainError::Content { seq })
            }
            Tamper::PrevChainHash => {
                record.prev_chain_hash = flip_hex(&record.prev_chain_hash);
                Some(ChainError::Link { seq })
            }
            Tamper::ChainHash => {
                record.chain_hash = flip_hex(&record.chain_hash);
                Some(ChainError::Chain { seq })
            }
            Tamper::Seq => {
                record.seq += 1;
                Some(ChainError::Gap { seq: seq + 1 })
            }
            Tamper::Unhashed => {
                record.id.push('x');
                record.at.push('x');
                None
            }
        };
        match expected {
            Some(error) => prop_assert_eq!(verify(&records), Err(error)),
            None => prop_assert_eq!(verify(&records), Ok(())),
        }
    }

    /// Reordering two distinct records breaks the chain.
    #[test]
    fn reordering_is_detected(
        payloads in prop::collection::vec(payload(), 2..8),
        actors in prop::collection::vec(text(), 1..3),
        first in any::<prop::sample::Index>(),
        second in any::<prop::sample::Index>(),
    ) {
        let mut records = chain(&payloads, &actors);
        let a = first.index(records.len());
        let b = second.index(records.len());
        if a == b {
            return Ok(());
        }
        records.swap(a, b);
        prop_assert!(verify(&records).is_err());
    }
}
