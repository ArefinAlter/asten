//! Snapshots commit to a materialised state with a Merkle root over its nodes:
//! leaves are `SHA-256(0x00 ‖ canonical node JSON)` in id order, internal nodes
//! are `SHA-256(0x01 ‖ left ‖ right)`, and an odd layer is padded with the zero
//! hash. The same construction as `dynodoc-engine`'s `merkle_root`.

use sha2::{Digest, Sha256};

use crate::canonical::{canonical_json, hex};
use crate::events::ZERO_HASH;
use crate::materialize::DocState;

/// Lowercase hex Merkle root of the state's nodes.
#[must_use]
pub fn merkle_root(state: &DocState) -> String {
    let mut layer: Vec<[u8; 32]> = state
        .nodes
        .values()
        .map(|node| {
            let mut hasher = Sha256::new();
            hasher.update([0u8]);
            let json = serde_json::to_value(node).unwrap_or_default();
            hasher.update(canonical_json(&json));
            hasher.finalize().into()
        })
        .collect();
    if layer.is_empty() {
        return hex(&ZERO_HASH);
    }
    while layer.len() > 1 {
        let mut next = Vec::with_capacity(layer.len().div_ceil(2));
        for pair in layer.chunks(2) {
            let left = pair[0];
            let right = pair.get(1).copied().unwrap_or(ZERO_HASH);
            let mut hasher = Sha256::new();
            hasher.update([1u8]);
            hasher.update(left);
            hasher.update(right);
            next.push(hasher.finalize().into());
        }
        layer = next;
    }
    hex(&layer[0])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::materialize::state_from_doc;
    use serde_json::json;

    #[test]
    fn roots_change_with_content_and_not_with_meta() {
        let a = state_from_doc(
            &json!({"type":"doc","content":[{"type":"paragraph","attrs":{"id":"a"},"content":[{"type":"text","text":"one"}]}]}),
        );
        let mut b = a.clone();
        b.meta.insert("title".into(), json!("x"));
        assert_eq!(merkle_root(&a), merkle_root(&b));
        let c = state_from_doc(
            &json!({"type":"doc","content":[{"type":"paragraph","attrs":{"id":"a"},"content":[{"type":"text","text":"two"}]}]}),
        );
        assert_ne!(merkle_root(&a), merkle_root(&c));
        assert_eq!(merkle_root(&DocState::default()), hex(&ZERO_HASH));
    }
}
