//! The hash-chained event log.
//!
//! Every change is a typed event on a node. `content_hash` covers the event
//! type, the target node, the canonical payload and the actor, each
//! length-framed; `chain_hash = SHA-256(content_hash ‖ prev_chain_hash)` with a
//! 32-zero-byte genesis. Sequence numbers are 1-based and gap-free per document.
//! The layout follows `dynodoc-engine`'s `log.rs`, except that Asten actors are
//! strings rather than UUIDs, so the two chains are not byte-compatible.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::canonical::{canonical_json, hex, unhex};
use crate::patch::RichTextPatch;

pub const ZERO_HASH: [u8; 32] = [0u8; 32];
pub const ZERO_HASH_HEX: &str = "0000000000000000000000000000000000000000000000000000000000000000";

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export, export_to = "blocks/"))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(tag = "type")]
pub enum EventPayload {
    NodeCreated {
        node_id: String,
        node_type: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        parent_id: Option<String>,
        /// Dense order key among siblings (fractional index).
        pos: String,
        #[serde(default)]
        fields: Value,
    },
    NodeMoved {
        node_id: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        new_parent_id: Option<String>,
        new_pos: String,
    },
    NodeDeleted {
        node_id: String,
    },
    NodeRestored {
        node_id: String,
    },
    FieldEdited {
        node_id: String,
        field: String,
        value: Value,
    },
    RichTextPatched {
        node_id: String,
        patch: RichTextPatch,
    },
    /// Document-level metadata: title, scene placement, language.
    MetaEdited {
        field: String,
        value: Value,
    },
}

impl EventPayload {
    #[must_use]
    pub const fn event_type(&self) -> &'static str {
        match self {
            EventPayload::NodeCreated { .. } => "NodeCreated",
            EventPayload::NodeMoved { .. } => "NodeMoved",
            EventPayload::NodeDeleted { .. } => "NodeDeleted",
            EventPayload::NodeRestored { .. } => "NodeRestored",
            EventPayload::FieldEdited { .. } => "FieldEdited",
            EventPayload::RichTextPatched { .. } => "RichTextPatched",
            EventPayload::MetaEdited { .. } => "MetaEdited",
        }
    }

    #[must_use]
    pub fn target_node_id(&self) -> Option<&str> {
        match self {
            EventPayload::NodeCreated { node_id, .. }
            | EventPayload::NodeMoved { node_id, .. }
            | EventPayload::NodeDeleted { node_id }
            | EventPayload::NodeRestored { node_id }
            | EventPayload::FieldEdited { node_id, .. }
            | EventPayload::RichTextPatched { node_id, .. } => Some(node_id),
            EventPayload::MetaEdited { .. } => None,
        }
    }
}

/// A stored event with its chain position and hashes (lowercase hex).
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export, export_to = "blocks/"))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct EventRecord {
    pub id: String,
    pub seq: i64,
    pub actor: String,
    /// RFC 3339 timestamp supplied by the caller. Not hashed.
    pub at: String,
    pub payload: EventPayload,
    pub content_hash: String,
    pub prev_chain_hash: String,
    pub chain_hash: String,
}

fn feed(hasher: &mut Sha256, bytes: &[u8]) {
    hasher.update((bytes.len() as u64).to_be_bytes());
    hasher.update(bytes);
}

/// SHA-256 over the length-framed type, target, canonical payload and actor.
#[must_use]
pub fn content_hash(payload: &EventPayload, actor: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    feed(&mut hasher, payload.event_type().as_bytes());
    feed(
        &mut hasher,
        payload.target_node_id().unwrap_or("").as_bytes(),
    );
    let canonical = serde_json::to_value(payload)
        .map(|value| canonical_json(&value))
        .unwrap_or_default();
    feed(&mut hasher, &canonical);
    feed(&mut hasher, actor.as_bytes());
    hasher.finalize().into()
}

/// `SHA-256(content_hash ‖ prev_chain_hash)`.
#[must_use]
pub fn chain_hash(content: &[u8; 32], prev: &[u8; 32]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(content);
    hasher.update(prev);
    hasher.finalize().into()
}

/// Build the next record after `prev` (or the genesis record).
#[must_use]
pub fn next_record(
    prev: Option<&EventRecord>,
    payload: EventPayload,
    actor: &str,
    at: &str,
) -> EventRecord {
    let prev_chain = prev
        .and_then(|record| unhex(&record.chain_hash))
        .and_then(|bytes| <[u8; 32]>::try_from(bytes).ok())
        .unwrap_or(ZERO_HASH);
    let content = content_hash(&payload, actor);
    let chain = chain_hash(&content, &prev_chain);
    EventRecord {
        id: crate::new_ulid(),
        seq: prev.map_or(1, |record| record.seq + 1),
        actor: actor.to_owned(),
        at: at.to_owned(),
        payload,
        content_hash: hex(&content),
        prev_chain_hash: hex(&prev_chain),
        chain_hash: hex(&chain),
    }
}

/// Build the next record after a known head `(seq, chain_hash)`, without the
/// previous record in hand.
#[must_use]
pub fn next_record_after(
    head_seq: i64,
    head_chain_hash: &str,
    payload: EventPayload,
    actor: &str,
    at: &str,
) -> EventRecord {
    let prev_chain = unhex(head_chain_hash)
        .and_then(|bytes| <[u8; 32]>::try_from(bytes).ok())
        .unwrap_or(ZERO_HASH);
    let content = content_hash(&payload, actor);
    let chain = chain_hash(&content, &prev_chain);
    EventRecord {
        id: crate::new_ulid(),
        seq: head_seq + 1,
        actor: actor.to_owned(),
        at: at.to_owned(),
        payload,
        content_hash: hex(&content),
        prev_chain_hash: hex(&prev_chain),
        chain_hash: hex(&chain),
    }
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum ChainError {
    #[error("event {seq}: sequence is not contiguous")]
    Gap { seq: i64 },
    #[error("event {seq}: content hash does not match the payload")]
    Content { seq: i64 },
    #[error("event {seq}: previous chain hash does not match the preceding event")]
    Link { seq: i64 },
    #[error("event {seq}: chain hash does not recompute")]
    Chain { seq: i64 },
}

/// Verify a document's full chain in sequence order.
pub fn verify(records: &[EventRecord]) -> Result<(), ChainError> {
    let mut prev = ZERO_HASH;
    for (index, record) in records.iter().enumerate() {
        let expected_seq = index as i64 + 1;
        if record.seq != expected_seq {
            return Err(ChainError::Gap { seq: record.seq });
        }
        let content = content_hash(&record.payload, &record.actor);
        if hex(&content) != record.content_hash {
            return Err(ChainError::Content { seq: record.seq });
        }
        if hex(&prev) != record.prev_chain_hash {
            return Err(ChainError::Link { seq: record.seq });
        }
        let chain = chain_hash(&content, &prev);
        if hex(&chain) != record.chain_hash {
            return Err(ChainError::Chain { seq: record.seq });
        }
        prev = chain;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn created(id: &str) -> EventPayload {
        EventPayload::NodeCreated {
            node_id: id.into(),
            node_type: "paragraph".into(),
            parent_id: None,
            pos: "V".into(),
            fields: json!({"content": {"type":"paragraph","attrs":{"id":id}}, "label": ""}),
        }
    }

    #[test]
    fn chains_and_verifies() {
        let first = next_record(None, created("a"), "author", "2026-10-11T00:00:00Z");
        assert_eq!(first.seq, 1);
        assert_eq!(first.prev_chain_hash, ZERO_HASH_HEX);
        let second = next_record(Some(&first), created("b"), "author", "2026-10-11T00:00:01Z");
        assert_eq!(second.seq, 2);
        assert_eq!(second.prev_chain_hash, first.chain_hash);
        assert_eq!(verify(&[first.clone(), second.clone()]), Ok(()));

        let mut tampered = second.clone();
        tampered.actor = "someone else".into();
        assert_eq!(
            verify(&[first.clone(), tampered]),
            Err(ChainError::Content { seq: 2 })
        );

        let mut relinked = second.clone();
        relinked.prev_chain_hash = ZERO_HASH_HEX.into();
        assert_eq!(verify(&[first, relinked]), Err(ChainError::Link { seq: 2 }));
    }

    #[test]
    fn payloads_serialise_with_a_type_tag() {
        let text = serde_json::to_string(&EventPayload::NodeDeleted {
            node_id: "x".into(),
        })
        .unwrap();
        assert_eq!(text, r#"{"type":"NodeDeleted","node_id":"x"}"#);
    }
}
