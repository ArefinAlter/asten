//! Property tests for the materialised state, the semantic diff, the
//! three-way merge and fractional positions.

use std::collections::{BTreeMap, BTreeSet};

use asten_blocks::materialize::{
    DocState, apply, diff_docs_with_state, diff_states, doc_from_state, state_from_doc,
};
use asten_blocks::merge::{Resolutions, three_way};
use asten_blocks::{plain_text, pos};
use proptest::prelude::*;
use serde_json::{Value, json};

const CASES: u32 = 256;

// ───────────────────────── generators ─────────────────────────

const POOL: [&str; 8] = ["b0", "b1", "b2", "b3", "b4", "b5", "b6", "b7"];

fn scalar() -> impl Strategy<Value = char> {
    prop_oneof![
        6 => prop::char::range('a', 'd'),
        1 => Just(' '),
        1 => prop::char::range('\u{0980}', '\u{09FF}'),
        1 => prop::char::range('\u{1F600}', '\u{1F64F}'),
    ]
}

/// A canonical block: one or two runs, the second italic, so content diffs
/// produce rich-text patches rather than whole replacements.
fn block_content() -> impl Strategy<Value = (String, Vec<Value>)> {
    (
        prop_oneof![4 => Just("paragraph"), 1 => Just("heading")],
        prop::collection::vec(scalar(), 0..6),
        prop::collection::vec(scalar(), 0..4),
    )
        .prop_map(|(node_type, plain, italic)| {
            let mut content = Vec::new();
            if !plain.is_empty() {
                content.push(json!({"type": "text", "text": plain.iter().collect::<String>()}));
            }
            if !italic.is_empty() {
                content.push(json!({
                    "type": "text",
                    "text": italic.iter().collect::<String>(),
                    "marks": [{"type": "italic"}],
                }));
            }
            (node_type.to_owned(), content)
        })
}

fn block(id: &str, node_type: &str, content: &[Value]) -> Value {
    let mut node = json!({"type": node_type, "attrs": {"id": id}});
    if !content.is_empty() {
        node["content"] = Value::Array(content.to_vec());
    }
    node
}

/// A doc over a random subset of the id pool in a random order, with random
/// content per block. Independent docs from one pool give inserts, deletes,
/// moves and text edits all at once.
fn doc() -> impl Strategy<Value = Value> {
    (
        prop::collection::vec(any::<bool>(), POOL.len()),
        prop::sample::subsequence(POOL.to_vec(), 0..=POOL.len()).prop_shuffle(),
        prop::collection::vec(block_content(), POOL.len()),
    )
        .prop_map(|(_, order, contents)| {
            let blocks: Vec<Value> = order
                .iter()
                .map(|id| {
                    let index = POOL.iter().position(|p| p == id).unwrap();
                    let (node_type, content) = &contents[index];
                    block(id, node_type, content)
                })
                .collect();
            json!({"type": "doc", "content": blocks})
        })
}

// ───────────────────────── helpers ─────────────────────────

/// The observable shape of a state: live order plus every live node's fields,
/// independent of the fractional keys and tombstones.
type Shape = (
    Vec<String>,
    BTreeMap<String, (String, Option<String>, Value)>,
);

fn shape(state: &DocState) -> Shape {
    let order = state.live_ids();
    let nodes = state
        .live_in_order()
        .into_iter()
        .map(|node| {
            (
                node.id.clone(),
                (
                    node.node_type.clone(),
                    node.parent_id.clone(),
                    node.current_fields.clone(),
                ),
            )
        })
        .collect();
    (order, nodes)
}

fn fold_into(state: &mut DocState, ops: &[asten_blocks::EventPayload]) {
    for op in ops {
        apply(state, op).expect("op applies");
    }
}

fn live_set(state: &DocState) -> BTreeSet<String> {
    state.live_ids().into_iter().collect()
}

/// A state derived from `base` by events, as the engine builds branches.
fn branch(base: &DocState, target: &Value) -> DocState {
    let mut state = base.clone();
    fold_into(&mut state, &diff_docs_with_state(base, target));
    state
}

fn check_live_nodes(state: &DocState) -> Result<(), TestCaseError> {
    for node in state.live_in_order() {
        let content = node.content().expect("live blocks carry content");
        prop_assert_eq!(
            content.get("type").and_then(Value::as_str),
            Some(node.node_type.as_str()),
            "node type follows content"
        );
        let label = plain_text(content);
        prop_assert_eq!(node.label(), label.as_str(), "label follows content");
    }
    Ok(())
}

// ───────────────────────── diff and fold ─────────────────────────

proptest! {
    #![proptest_config(ProptestConfig { cases: CASES, .. ProptestConfig::default() })]

    /// Folding `diff(a → b)` onto the state of `a` gives the state of `b`,
    /// including labels and node types, and the diff is then exhausted.
    #[test]
    fn fold_of_diff_reaches_the_target(a in doc(), b in doc(), c in doc()) {
        let mut state = state_from_doc(&a);
        let ops = diff_states(&state, &state_from_doc(&b));
        fold_into(&mut state, &ops);
        prop_assert_eq!(doc_from_state(&state), b.clone());
        prop_assert_eq!(shape(&state), shape(&state_from_doc(&b)));
        check_live_nodes(&state)?;
        prop_assert!(diff_docs_with_state(&state, &b).is_empty(), "diff is not exhausted");

        // A second hop over the same state exercises tombstones and restorations.
        let ops = diff_docs_with_state(&state, &c);
        fold_into(&mut state, &ops);
        prop_assert_eq!(doc_from_state(&state), c.clone());
        prop_assert_eq!(shape(&state), shape(&state_from_doc(&c)));
        check_live_nodes(&state)?;
        prop_assert!(diff_docs_with_state(&state, &c).is_empty());

        // Keys stay strictly ordered among live nodes.
        let keys: Vec<&str> = state.live_in_order().iter().map(|node| node.pos.as_str()).collect();
        prop_assert!(keys.windows(2).all(|pair| pair[0] < pair[1]), "positions collide: {keys:?}");
    }

    /// Identical docs produce no events; a doc diffed against itself through a
    /// state with tombstones produces none either.
    #[test]
    fn unchanged_docs_are_silent(a in doc(), b in doc()) {
        prop_assert!(diff_states(&state_from_doc(&a), &state_from_doc(&a)).is_empty());
        let state = branch(&state_from_doc(&a), &b);
        prop_assert!(diff_docs_with_state(&state, &b).is_empty());
    }
}

// ───────────────────────── three-way merge ─────────────────────────

proptest! {
    #![proptest_config(ProptestConfig { cases: CASES, .. ProptestConfig::default() })]

    /// A side that did nothing changes nothing: merging `base` as theirs gives
    /// `ours`; merging `base` as ours gives `theirs`.
    #[test]
    fn merge_identities(a in doc(), b in doc(), c in doc()) {
        let base = state_from_doc(&a);
        let ours = branch(&base, &b);
        let theirs = branch(&base, &c);
        let none = Resolutions::new();

        let kept = three_way(&base, &ours, &base, &none);
        prop_assert!(kept.conflicts.is_empty());
        prop_assert!(kept.ops.is_empty());
        prop_assert_eq!(kept.state, ours.clone());

        let taken = three_way(&base, &base, &theirs, &none);
        prop_assert!(taken.conflicts.is_empty());
        prop_assert_eq!(taken.state, theirs.clone());

        let same = three_way(&base, &ours, &ours, &none);
        prop_assert!(same.conflicts.is_empty());
        prop_assert_eq!(same.state, ours.clone());
    }

    /// A merge never loses a block live on both sides, its ops replay onto
    /// `ours` to the merged state, and every conflict key is unique and names
    /// a node that exists.
    #[test]
    fn merge_keeps_shared_blocks(a in doc(), b in doc(), c in doc()) {
        let base = state_from_doc(&a);
        let ours = branch(&base, &b);
        let theirs = branch(&base, &c);
        let result = three_way(&base, &ours, &theirs, &Resolutions::new());

        let shared: BTreeSet<String> = live_set(&ours).intersection(&live_set(&theirs)).cloned().collect();
        let merged_live = live_set(&result.state);
        for id in &shared {
            prop_assert!(merged_live.contains(id), "block {id} lost in merge");
        }
        // Blocks that only one side touched follow that side.
        for id in live_set(&theirs).difference(&live_set(&ours)) {
            let untouched_by_us = ours.nodes.get(id) == base.nodes.get(id);
            if untouched_by_us {
                prop_assert!(merged_live.contains(id), "their addition {id} dropped");
            }
        }
        for id in live_set(&ours).difference(&live_set(&theirs)) {
            let untouched_by_them = theirs.nodes.get(id) == base.nodes.get(id);
            if untouched_by_them {
                prop_assert!(merged_live.contains(id), "our addition {id} dropped");
            }
        }

        let mut replayed = ours.clone();
        fold_into(&mut replayed, &result.ops);
        prop_assert_eq!(shape(&replayed), shape(&result.state));
        check_live_nodes(&result.state)?;

        let mut keys = BTreeSet::new();
        for conflict in &result.conflicts {
            prop_assert!(keys.insert(conflict.key.clone()), "duplicate conflict {}", conflict.key);
            prop_assert!(result.state.nodes.contains_key(&conflict.node_id));
        }
    }

    /// Resolving every conflict towards theirs yields a state with no
    /// conflicts whose conflicted fields equal theirs.
    #[test]
    fn resolutions_towards_theirs_apply(a in doc(), b in doc(), c in doc()) {
        let base = state_from_doc(&a);
        let ours = branch(&base, &b);
        let theirs = branch(&base, &c);
        let first = three_way(&base, &ours, &theirs, &Resolutions::new());
        let resolutions: Resolutions = first
            .conflicts
            .iter()
            .map(|conflict| (conflict.key.clone(), "theirs".to_owned()))
            .collect();
        let second = three_way(&base, &ours, &theirs, &resolutions);
        prop_assert!(second.conflicts.is_empty(), "{:?}", second.conflicts);
        for conflict in &first.conflicts {
            let node = &second.state.nodes[&conflict.node_id];
            match conflict.field.as_str() {
                "$node" => prop_assert_eq!(node, &theirs.nodes[&conflict.node_id]),
                "$location" => {
                    let their_node = &theirs.nodes[&conflict.node_id];
                    prop_assert_eq!(&node.pos, &their_node.pos);
                    prop_assert_eq!(&node.parent_id, &their_node.parent_id);
                }
                field => prop_assert_eq!(
                    node.current_fields.get(field),
                    theirs.nodes[&conflict.node_id].current_fields.get(field)
                ),
            }
        }
    }
}

// ───────────────────────── fractional positions ─────────────────────────

const ALPHABET: &[u8; 62] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

/// A valid key: digits of the alphabet, never ending in the lowest digit.
fn key() -> impl Strategy<Value = String> {
    prop::collection::vec(0..62usize, 1..6).prop_map(|digits| {
        let mut key: String = digits
            .iter()
            .map(|digit| char::from(ALPHABET[*digit]))
            .collect();
        if key.ends_with('0') {
            key.pop();
            key.push('V');
        }
        key
    })
}

fn ordered_pair() -> impl Strategy<Value = (String, String)> {
    (key(), key())
        .prop_filter("distinct", |(a, b)| a != b)
        .prop_map(|(a, b)| if a < b { (a, b) } else { (b, a) })
}

fn depth(count: usize) -> usize {
    (usize::BITS - count.leading_zeros()) as usize
}

proptest! {
    #![proptest_config(ProptestConfig { cases: CASES, .. ProptestConfig::default() })]

    #[test]
    fn between_is_strictly_between((prev, next) in ordered_pair()) {
        let mid = pos::between(Some(&prev), Some(&next));
        prop_assert!(prev < mid && mid < next, "{prev} < {mid} < {next}");
        prop_assert!(!mid.ends_with('0'));
        let before = pos::between(None, Some(&prev));
        prop_assert!(before < prev, "{before} < {prev}");
        let after = pos::between(Some(&next), None);
        prop_assert!(next < after, "{next} < {after}");
        prop_assert!(mid.len() <= prev.len().max(next.len()) + 2, "{prev} {next} -> {mid}");
    }

    /// Spread keys are strictly increasing, strictly inside the gap, and grow
    /// in length with log2(count), not with count.
    #[test]
    fn spread_keeps_order_and_short_keys((prev, next) in ordered_pair(), count in 0..300usize) {
        let keys = pos::spread(Some(&prev), Some(&next), count);
        prop_assert_eq!(keys.len(), count);
        prop_assert!(keys.windows(2).all(|pair| pair[0] < pair[1]), "{keys:?}");
        prop_assert!(keys.iter().all(|key| prev < *key && *key < next), "{prev} {next} {keys:?}");
        let longest = keys.iter().map(String::len).max().unwrap_or(0);
        let bound = prev.len().max(next.len()) + 2 * (depth(count) + 1);
        prop_assert!(longest <= bound, "longest {longest} > {bound} for {count} keys in {prev}..{next}");
        // Every gap can still be bisected.
        for pair in keys.windows(2) {
            let mid = pos::between(Some(&pair[0]), Some(&pair[1]));
            prop_assert!(pair[0] < mid && mid < pair[1]);
        }
    }

    #[test]
    fn sequence_is_ordered_and_bisectable(count in 0..2000usize) {
        let keys = pos::sequence(count);
        prop_assert_eq!(keys.len(), count);
        prop_assert!(keys.windows(2).all(|pair| pair[0] < pair[1]));
        let longest = keys.iter().map(String::len).max().unwrap_or(0);
        prop_assert!(longest <= 2 * (depth(count) + 1), "longest {longest} for {count}");
    }
}
