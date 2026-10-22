//! The block model behind the Asten desk.
//!
//! A document is a ProseMirror `doc` whose top-level nodes are **blocks**. Each
//! block carries a stable id (`attrs.id`, a UUID today or a ULID when Dynodoc
//! mints it) that never changes and is never reused. Every change to a document
//! is a typed event in an append-only, hash-chained log; current state is a fold
//! over the log; snapshots pin intermediate states with a Merkle commitment.
//!
//! The shapes follow the MIT `dynodoc-engine` where they are visible on the wire
//! (`RichTextPatch` v1 with Unicode-scalar counts and SHA-256 fingerprints,
//! `NodeCreated`/`NodeMoved`/`NodeDeleted`/`FieldEdited`/`RichTextPatched`,
//! `chain_hash = SHA-256(content_hash ‖ prev_chain_hash)`, a Merkle root over
//! materialised nodes). What Dynodoc lacks and the canon engine needs is added
//! here: deterministic diffs, position mapping through patches, anchor
//! re-attachment, and blame.

pub mod anchor;
pub mod blame;
pub mod canonical;
pub mod events;
pub mod materialize;
pub mod merge;
pub mod patch;
pub mod pos;
pub mod snapshot;

use serde_json::Value;

pub use anchor::{Mapped, map_span, reattach};
pub use blame::{BlameSpan, blame};
pub use canonical::{canonical_json, fingerprint};
pub use events::{
    ChainError, EventPayload, EventRecord, ZERO_HASH_HEX, chain_hash, content_hash, next_record,
    next_record_after, verify,
};
pub use materialize::{
    ApplyError, DocState, MaterializedNode, apply, diff_docs, diff_docs_with_state, diff_states,
    doc_from_state, fold, state_from_doc,
};
pub use merge::{Conflict, MergeResult, three_way};
pub use patch::{Bias, PatchError, RichTextEdit, RichTextPatch, TextOp, apply_patch, make_patch};
pub use snapshot::merkle_root;

/// A top-level block of a document.
#[derive(Clone, Debug, PartialEq)]
pub struct Block {
    pub id: String,
    pub node_type: String,
    /// Plain text in Unicode scalars, as the anchors count it.
    pub text: String,
    pub node: Value,
}

/// The stable id of a ProseMirror node, if it has one.
#[must_use]
pub fn node_id(node: &Value) -> Option<&str> {
    let attrs = node.get("attrs")?;
    attrs
        .get("id")
        .or_else(|| attrs.get("blockId"))
        .and_then(Value::as_str)
        .filter(|id| !id.is_empty())
}

/// The top-level blocks of a `doc` as `(id, node)`, in order and without
/// copying the nodes. Blocks without an id get a positional placeholder,
/// which is not stable; callers should run `ensure_block_ids` first.
#[must_use]
pub fn block_nodes(doc: &Value) -> Vec<(String, &Value)> {
    let Some(content) = doc.get("content").and_then(Value::as_array) else {
        return Vec::new();
    };
    content
        .iter()
        .enumerate()
        .map(|(index, node)| {
            (
                node_id(node)
                    .map(str::to_owned)
                    .unwrap_or_else(|| format!("anon-{index}")),
                node,
            )
        })
        .collect()
}

/// The top-level blocks of a `doc`, in order, each with a copy of its node.
/// See `block_nodes` for the ids.
#[must_use]
pub fn blocks_of(doc: &Value) -> Vec<Block> {
    block_nodes(doc)
        .into_iter()
        .map(|(id, node)| Block {
            id,
            node_type: node
                .get("type")
                .and_then(Value::as_str)
                .unwrap_or("paragraph")
                .to_owned(),
            text: plain_text(node),
            node: node.clone(),
        })
        .collect()
}

/// Words in a document: its plain text split on Unicode whitespace
/// (`White_Space`), empty runs dropped. The desk's `countWords` applies the
/// same rule to the open page, so the two counts agree.
#[must_use]
pub fn word_count(doc: &Value) -> usize {
    plain_text(doc).split_whitespace().count()
}

/// Plain text of a node. Text nodes are concatenated; a `hardBreak` is a
/// newline; children of a `doc` are joined with newlines, everything else with
/// nothing, matching Dynodoc's `plain_text`.
#[must_use]
pub fn plain_text(node: &Value) -> String {
    let mut out = String::new();
    write_text(node, &mut out);
    out
}

fn write_text(node: &Value, out: &mut String) {
    if let Some(text) = node.get("text").and_then(Value::as_str) {
        out.push_str(text);
        return;
    }
    let node_type = node.get("type").and_then(Value::as_str).unwrap_or("");
    if node_type == "hardBreak" {
        out.push('\n');
        return;
    }
    if let Some(latex) = node.pointer("/attrs/latex").and_then(Value::as_str) {
        out.push_str(latex);
        return;
    }
    let Some(children) = node.get("content").and_then(Value::as_array) else {
        return;
    };
    let separator = if node_type == "doc" { "\n" } else { "" };
    for (index, child) in children.iter().enumerate() {
        if index > 0 {
            out.push_str(separator);
        }
        write_text(child, out);
    }
}

/// Count of Unicode scalar values in a string.
#[must_use]
pub fn scalar_len(text: &str) -> u32 {
    text.chars().count() as u32
}

/// The scalar offset of a byte offset in `text`.
#[must_use]
pub fn scalar_offset(text: &str, byte_offset: usize) -> u32 {
    text[..byte_offset.min(text.len())].chars().count() as u32
}

/// Give every block node a stable id if it lacks one. Nested nodes keep their
/// own ids when present; the engine treats the top-level block as the unit of
/// identity.
pub fn ensure_block_ids(doc: &mut Value, mut mint: impl FnMut() -> String) {
    let Some(content) = doc.get_mut("content").and_then(Value::as_array_mut) else {
        return;
    };
    for node in content {
        if node_id(node).is_some() {
            continue;
        }
        let Some(object) = node.as_object_mut() else {
            continue;
        };
        let attrs = object
            .entry("attrs")
            .or_insert_with(|| Value::Object(serde_json::Map::new()));
        if let Some(attrs) = attrs.as_object_mut() {
            attrs.insert("id".into(), Value::String(mint()));
        }
    }
}

/// Replace the scalar range `[start, end)` of a block's plain text with
/// `replacement`, editing the text nodes in place and leaving marks on the
/// surrounding text as they were. Returns false when the range does not fit.
pub fn replace_text(node: &mut Value, start: u32, end: u32, replacement: &str) -> bool {
    let total = scalar_len(&plain_text(node));
    if start > end || end > total {
        return false;
    }
    let mut cursor = 0u32;
    let mut inserted = false;
    let mut remaining_start = start;
    let mut remaining_end = end;
    fn walk(
        node: &mut Value,
        cursor: &mut u32,
        start: &mut u32,
        end: &mut u32,
        replacement: &str,
        inserted: &mut bool,
    ) {
        if let Some(text) = node.get("text").and_then(Value::as_str).map(str::to_owned) {
            let len = scalar_len(&text);
            let node_start = *cursor;
            let node_end = *cursor + len;
            *cursor = node_end;
            // Overlap of [start, end) with this node.
            let lo = (*start).max(node_start);
            let hi = (*end).min(node_end);
            if lo > hi
                || (lo == hi
                    && !(*start == *end
                        && *start >= node_start
                        && *start <= node_end
                        && !*inserted))
            {
                return;
            }
            let chars: Vec<char> = text.chars().collect();
            let mut out: String = chars[..(lo - node_start) as usize].iter().collect();
            if !*inserted {
                out.push_str(replacement);
                *inserted = true;
            }
            out.extend(chars[(hi - node_start) as usize..].iter());
            if let Some(object) = node.as_object_mut() {
                object.insert("text".into(), Value::String(out));
            }
            return;
        }
        if node.get("type").and_then(Value::as_str) == Some("hardBreak") {
            *cursor += 1;
            return;
        }
        if let Some(children) = node.get_mut("content").and_then(Value::as_array_mut) {
            for child in children.iter_mut() {
                walk(child, cursor, start, end, replacement, inserted);
            }
        }
    }
    walk(
        node,
        &mut cursor,
        &mut remaining_start,
        &mut remaining_end,
        replacement,
        &mut inserted,
    );
    if !inserted && start == end && start == total {
        // Appending at the very end of an empty or exhausted block.
        if let Some(children) = node.get_mut("content").and_then(Value::as_array_mut) {
            children.push(serde_json::json!({"type":"text","text":replacement}));
            inserted = true;
        }
    }
    // Drop text nodes emptied by the edit.
    if let Some(children) = node.get_mut("content").and_then(Value::as_array_mut) {
        children.retain(|child| {
            child
                .get("text")
                .and_then(Value::as_str)
                .is_none_or(|text| !text.is_empty())
        });
    }
    inserted
}

/// Materialise a document at `through_seq` from its full event list.
pub fn state_at(
    records: &[events::EventRecord],
    through_seq: i64,
) -> Result<materialize::DocState, materialize::ApplyError> {
    materialize::fold(
        records
            .iter()
            .filter(|record| record.seq <= through_seq)
            .map(|record| &record.payload),
    )
}

/// A new ULID string, the id scheme Dynodoc uses for nodes and events.
#[must_use]
pub fn new_ulid() -> String {
    ulid::Ulid::new().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn extracts_blocks_and_text() {
        let doc = json!({"type":"doc","content":[
            {"type":"paragraph","attrs":{"id":"a"},"content":[{"type":"text","text":"Mira "},{"type":"text","text":"waited.","marks":[{"type":"italic"}]}]},
            {"type":"heading","attrs":{"id":"b","level":2},"content":[{"type":"text","text":"Two"}]},
            {"type":"paragraph","content":[{"type":"text","text":"no id"}]}
        ]});
        let blocks = blocks_of(&doc);
        assert_eq!(blocks.len(), 3);
        assert_eq!(blocks[0].text, "Mira waited.");
        assert_eq!(blocks[1].node_type, "heading");
        assert_eq!(blocks[2].id, "anon-2");
        assert_eq!(plain_text(&doc), "Mira waited.\nTwo\nno id");
    }

    #[test]
    fn replaces_text_across_marked_runs() {
        let mut node = json!({"type":"paragraph","attrs":{"id":"a"},"content":[
            {"type":"text","text":"Mira "},
            {"type":"text","text":"waited","marks":[{"type":"italic"}]},
            {"type":"text","text":" for Oren."}
        ]});
        // "Mira waited for Oren." -> rename Oren (16..20) to Aric
        assert!(replace_text(&mut node, 16, 20, "Aric"));
        assert_eq!(plain_text(&node), "Mira waited for Aric.");
        // Replace across the italic boundary.
        assert!(replace_text(&mut node, 2, 8, "XX"));
        assert_eq!(plain_text(&node), "MiXXted for Aric.");
        assert!(!replace_text(&mut node, 5, 99, "no"));
    }

    #[test]
    fn mints_missing_ids_only() {
        let mut doc = json!({"type":"doc","content":[
            {"type":"paragraph","attrs":{"id":"keep"}},
            {"type":"paragraph"}
        ]});
        let mut counter = 0;
        ensure_block_ids(&mut doc, || {
            counter += 1;
            format!("new-{counter}")
        });
        let blocks = blocks_of(&doc);
        assert_eq!(blocks[0].id, "keep");
        assert_eq!(blocks[1].id, "new-1");
    }
}

#[cfg(test)]
mod word_count_tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn counts_whitespace_separated_runs_of_the_plain_text() {
        let doc = json!({"type":"doc","content":[
            {"type":"paragraph","attrs":{"id":"a"},"content":[
                {"type":"text","text":"Mira "},
                {"type":"text","text":"tied","marks":[{"type":"italic"}]},
                {"type":"text","text":" back her hair.  "}
            ]},
            {"type":"paragraph","attrs":{"id":"b"},"content":[
                {"type":"text","text":"one\u{a0}two\u{2003}three"},
                {"type":"hardBreak"},
                {"type":"text","text":"four"}
            ]},
            {"type":"paragraph","attrs":{"id":"c"}}
        ]});
        // A mark boundary does not split a word; NBSP and an em space do;
        // a hard break does; an empty paragraph adds nothing.
        assert_eq!(word_count(&doc), 9);
        assert_eq!(word_count(&json!({"type":"doc","content":[]})), 0);
        assert_eq!(word_count(&json!(null)), 0);
    }

    #[test]
    fn block_nodes_and_blocks_of_agree_on_ids() {
        let doc = json!({"type":"doc","content":[
            {"type":"paragraph","attrs":{"id":"keep"}},
            {"type":"paragraph"}
        ]});
        let ids: Vec<String> = block_nodes(&doc).into_iter().map(|(id, _)| id).collect();
        assert_eq!(ids, ["keep", "anon-1"]);
        let blocks = blocks_of(&doc);
        assert_eq!(blocks[1].id, "anon-1");
    }
}
