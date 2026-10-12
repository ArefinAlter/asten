//! Canonical JSON and fingerprints.
//!
//! Object keys are sorted by their UTF-8 bytes, no whitespace is emitted, and
//! scalars are written by serde_json. This matches `dynodoc-engine`'s
//! `canonical_json`, so fingerprints agree across both engines.

use serde_json::Value;
use sha2::{Digest, Sha256};

/// Canonical JSON bytes of a value.
#[must_use]
pub fn canonical_json(value: &Value) -> Vec<u8> {
    let mut out = Vec::new();
    write_canonical(value, &mut out);
    out
}

fn write_canonical(value: &Value, out: &mut Vec<u8>) {
    match value {
        Value::Object(map) => {
            out.push(b'{');
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort_unstable_by(|a, b| a.as_bytes().cmp(b.as_bytes()));
            for (index, key) in keys.iter().enumerate() {
                if index > 0 {
                    out.push(b',');
                }
                write_scalar(&Value::String((*key).clone()), out);
                out.push(b':');
                write_canonical(&map[*key], out);
            }
            out.push(b'}');
        }
        Value::Array(items) => {
            out.push(b'[');
            for (index, item) in items.iter().enumerate() {
                if index > 0 {
                    out.push(b',');
                }
                write_canonical(item, out);
            }
            out.push(b']');
        }
        other => write_scalar(other, out),
    }
}

fn write_scalar(value: &Value, out: &mut Vec<u8>) {
    // serde_json's scalar encoding is deterministic.
    if let Ok(bytes) = serde_json::to_vec(value) {
        out.extend_from_slice(&bytes);
    }
}

/// Lowercase hex SHA-256 of the canonical JSON of a value.
#[must_use]
pub fn fingerprint(value: &Value) -> String {
    hex(&Sha256::digest(canonical_json(value)))
}

/// Lowercase hex of bytes.
#[must_use]
pub fn hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

/// Decode lowercase or uppercase hex.
#[must_use]
pub fn unhex(text: &str) -> Option<Vec<u8>> {
    if text.len() % 2 != 0 {
        return None;
    }
    (0..text.len())
        .step_by(2)
        .map(|index| u8::from_str_radix(text.get(index..index + 2)?, 16).ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn sorts_keys_and_strips_whitespace() {
        let value = json!({"b": [1, {"z": null, "a": "x"}], "a": true});
        assert_eq!(
            String::from_utf8(canonical_json(&value)).unwrap(),
            r#"{"a":true,"b":[1,{"a":"x","z":null}]}"#
        );
    }

    #[test]
    fn fingerprints_are_stable() {
        let left = json!({"type":"paragraph","content":[{"type":"text","text":"hi"}]});
        let right = json!({"content":[{"text":"hi","type":"text"}],"type":"paragraph"});
        assert_eq!(fingerprint(&left), fingerprint(&right));
        assert_eq!(fingerprint(&left).len(), 64);
        assert_eq!(unhex(&fingerprint(&left)).unwrap().len(), 32);
    }
}
