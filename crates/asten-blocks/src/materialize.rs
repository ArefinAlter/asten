//! Materialised document state: a fold over the event log, and the semantic diff
//! that turns two states (or two ProseMirror docs) into events.

use std::collections::{BTreeMap, HashSet};

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use thiserror::Error;

use crate::events::EventPayload;
use crate::patch::{apply_patch, make_patch};
use crate::{blocks_of, node_id, plain_text, pos};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MaterializedNode {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
    #[serde(rename = "type")]
    pub node_type: String,
    pub pos: String,
    /// `content` (the ProseMirror block) and `label` (its plain text), plus any
    /// other fields a client stores.
    pub current_fields: Value,
    #[serde(default)]
    pub deleted: bool,
}

impl MaterializedNode {
    #[must_use]
    pub fn content(&self) -> Option<&Value> {
        self.current_fields.get("content")
    }

    #[must_use]
    pub fn label(&self) -> &str {
        self.current_fields
            .get("label")
            .and_then(Value::as_str)
            .unwrap_or("")
    }
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct DocState {
    pub nodes: BTreeMap<String, MaterializedNode>,
    #[serde(default)]
    pub meta: BTreeMap<String, Value>,
}

impl DocState {
    /// Live nodes in sibling order.
    #[must_use]
    pub fn live_in_order(&self) -> Vec<&MaterializedNode> {
        let mut nodes: Vec<&MaterializedNode> =
            self.nodes.values().filter(|node| !node.deleted).collect();
        nodes.sort_by(|a, b| a.pos.cmp(&b.pos).then_with(|| a.id.cmp(&b.id)));
        nodes
    }

    #[must_use]
    pub fn live_ids(&self) -> Vec<String> {
        self.live_in_order()
            .into_iter()
            .map(|node| node.id.clone())
            .collect()
    }
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum ApplyError {
    #[error("rich-text patch failed: {0}")]
    RichText(String),
    #[error("node {node_id} does not exist")]
    MissingNode { node_id: String },
    #[error("node {node_id} has non-object fields")]
    NonObjectFields { node_id: String },
}

fn object_or_value(value: Value) -> Value {
    if value.is_object() {
        value
    } else {
        json!({ "value": value })
    }
}

fn set_label(node: &mut MaterializedNode) {
    let label = node.content().map(plain_text).unwrap_or_default();
    if let Some(fields) = node.current_fields.as_object_mut() {
        fields.insert("label".into(), Value::String(label));
    }
}

/// Apply one event to the state.
pub fn apply(state: &mut DocState, payload: &EventPayload) -> Result<(), ApplyError> {
    match payload {
        EventPayload::NodeCreated {
            node_id,
            node_type,
            parent_id,
            pos,
            fields,
        } => {
            let mut node = MaterializedNode {
                id: node_id.clone(),
                parent_id: parent_id.clone(),
                node_type: node_type.clone(),
                pos: pos.clone(),
                current_fields: object_or_value(fields.clone()),
                deleted: false,
            };
            if node.content().is_some() {
                set_label(&mut node);
            }
            state.nodes.insert(node_id.clone(), node);
        }
        EventPayload::NodeMoved {
            node_id,
            new_parent_id,
            new_pos,
        } => {
            let node = state
                .nodes
                .get_mut(node_id)
                .ok_or_else(|| ApplyError::MissingNode {
                    node_id: node_id.clone(),
                })?;
            node.parent_id = new_parent_id.clone();
            node.pos = new_pos.clone();
        }
        EventPayload::NodeDeleted { node_id } => {
            let node = state
                .nodes
                .get_mut(node_id)
                .ok_or_else(|| ApplyError::MissingNode {
                    node_id: node_id.clone(),
                })?;
            node.deleted = true;
        }
        EventPayload::NodeRestored { node_id } => {
            let node = state
                .nodes
                .get_mut(node_id)
                .ok_or_else(|| ApplyError::MissingNode {
                    node_id: node_id.clone(),
                })?;
            node.deleted = false;
        }
        EventPayload::FieldEdited {
            node_id,
            field,
            value,
        } => {
            let node = state
                .nodes
                .get_mut(node_id)
                .ok_or_else(|| ApplyError::MissingNode {
                    node_id: node_id.clone(),
                })?;
            let fields =
                node.current_fields
                    .as_object_mut()
                    .ok_or_else(|| ApplyError::NonObjectFields {
                        node_id: node_id.clone(),
                    })?;
            fields.insert(field.clone(), value.clone());
            if field == "content" {
                if let Some(node_type) = value.get("type").and_then(Value::as_str) {
                    node.node_type = node_type.to_owned();
                }
                set_label(node);
            }
        }
        EventPayload::RichTextPatched { node_id, patch } => {
            let node = state
                .nodes
                .get_mut(node_id)
                .ok_or_else(|| ApplyError::MissingNode {
                    node_id: node_id.clone(),
                })?;
            let before = node.content().cloned().unwrap_or(Value::Null);
            let after = apply_patch(&before, patch)
                .map_err(|error| ApplyError::RichText(error.to_string()))?;
            if let Some(node_type) = after.get("type").and_then(Value::as_str) {
                node.node_type = node_type.to_owned();
            }
            let fields =
                node.current_fields
                    .as_object_mut()
                    .ok_or_else(|| ApplyError::NonObjectFields {
                        node_id: node_id.clone(),
                    })?;
            fields.insert("content".into(), after);
            set_label(node);
        }
        EventPayload::MetaEdited { field, value } => {
            state.meta.insert(field.clone(), value.clone());
        }
    }
    Ok(())
}

/// Fold events into a state.
pub fn fold<'a>(
    payloads: impl IntoIterator<Item = &'a EventPayload>,
) -> Result<DocState, ApplyError> {
    let mut state = DocState::default();
    for payload in payloads {
        apply(&mut state, payload)?;
    }
    Ok(state)
}

/// Build a state from a ProseMirror doc. Blocks are nodes in order with fresh
/// fractional positions.
#[must_use]
pub fn state_from_doc(doc: &Value) -> DocState {
    let blocks = blocks_of(doc);
    let positions = pos::sequence(blocks.len());
    let mut state = DocState::default();
    for (block, position) in blocks.into_iter().zip(positions) {
        let mut node = MaterializedNode {
            id: block.id.clone(),
            parent_id: None,
            node_type: block.node_type.clone(),
            pos: position,
            current_fields: json!({ "content": block.node }),
            deleted: false,
        };
        set_label(&mut node);
        state.nodes.insert(block.id, node);
    }
    state
}

/// Rebuild the ProseMirror doc from the live nodes.
#[must_use]
pub fn doc_from_state(state: &DocState) -> Value {
    let content: Vec<Value> = state
        .live_in_order()
        .into_iter()
        .filter_map(|node| node.content().cloned())
        .collect();
    json!({ "type": "doc", "content": content })
}

/// Longest increasing subsequence over the `from` order of common nodes.
fn lis(sequence: &[usize]) -> HashSet<usize> {
    // Patience sorting with predecessor links; returns the indices into `sequence`.
    let mut tails: Vec<usize> = Vec::new(); // indices into sequence
    let mut prev: Vec<Option<usize>> = vec![None; sequence.len()];
    for (index, &value) in sequence.iter().enumerate() {
        let position = tails.partition_point(|&tail| sequence[tail] < value);
        if position > 0 {
            prev[index] = Some(tails[position - 1]);
        }
        if position == tails.len() {
            tails.push(index);
        } else {
            tails[position] = index;
        }
    }
    let mut kept = HashSet::new();
    let mut cursor = tails.last().copied();
    while let Some(index) = cursor {
        kept.insert(index);
        cursor = prev[index];
    }
    kept
}

/// The events that turn `from` into `to`. Order comes from `to`; positions are
/// kept from `from` wherever a node did not move, so the result is minimal.
#[must_use]
pub fn diff_states(from: &DocState, to: &DocState) -> Vec<EventPayload> {
    let mut ops = Vec::new();
    let from_order = from.live_ids();
    let to_order = to.live_ids();
    let from_index: BTreeMap<&str, usize> = from_order
        .iter()
        .enumerate()
        .map(|(index, id)| (id.as_str(), index))
        .collect();

    // Deletions: live in `from`, missing or deleted in `to`.
    for id in &from_order {
        let gone = to.nodes.get(id).is_none_or(|node| node.deleted);
        if gone {
            ops.push(EventPayload::NodeDeleted {
                node_id: id.clone(),
            });
        }
    }

    // Restorations and field changes for nodes present in both.
    for id in &to_order {
        let Some(to_node) = to.nodes.get(id) else {
            continue;
        };
        let Some(from_node) = from.nodes.get(id) else {
            continue;
        };
        if from_node.deleted {
            ops.push(EventPayload::NodeRestored {
                node_id: id.clone(),
            });
        }
        ops.extend(field_ops(from_node, to_node));
    }

    // Order: keep the longest run of common nodes already in order; move or create the rest.
    let common_positions: Vec<usize> = to_order
        .iter()
        .filter_map(|id| from_index.get(id.as_str()).copied())
        .collect();
    let common_ids: Vec<&String> = to_order
        .iter()
        .filter(|id| from_index.contains_key(id.as_str()))
        .collect();
    let kept = lis(&common_positions);
    let kept_ids: HashSet<&str> = kept
        .iter()
        .map(|index| common_ids[*index].as_str())
        .collect();

    let mut prev_pos: Option<String> = None;
    // Keys for a run of consecutive non-kept nodes are spread together, so a
    // thousand pasted paragraphs get short keys instead of ever longer ones.
    let mut run: std::collections::VecDeque<String> = std::collections::VecDeque::new();
    for (index, id) in to_order.iter().enumerate() {
        let to_node = &to.nodes[id];
        if kept_ids.contains(id.as_str()) {
            prev_pos = Some(from.nodes[id].pos.clone());
            run.clear();
            continue;
        }
        if run.is_empty() {
            let run_len = to_order[index..]
                .iter()
                .take_while(|next| !kept_ids.contains(next.as_str()))
                .count();
            let next_pos = to_order[index + run_len..]
                .first()
                .map(|next| from.nodes[next].pos.clone());
            run = pos::spread(prev_pos.as_deref(), next_pos.as_deref(), run_len).into();
        }
        let new_pos = run
            .pop_front()
            .unwrap_or_else(|| pos::between(prev_pos.as_deref(), None));
        if from.nodes.contains_key(id) {
            ops.push(EventPayload::NodeMoved {
                node_id: id.clone(),
                new_parent_id: to_node.parent_id.clone(),
                new_pos: new_pos.clone(),
            });
        } else {
            ops.push(EventPayload::NodeCreated {
                node_id: id.clone(),
                node_type: to_node.node_type.clone(),
                parent_id: to_node.parent_id.clone(),
                pos: new_pos.clone(),
                fields: to_node.current_fields.clone(),
            });
        }
        prev_pos = Some(new_pos);
    }

    // Metadata.
    for (field, value) in &to.meta {
        if from.meta.get(field) != Some(value) {
            ops.push(EventPayload::MetaEdited {
                field: field.clone(),
                value: value.clone(),
            });
        }
    }
    ops
}

fn field_ops(from: &MaterializedNode, to: &MaterializedNode) -> Vec<EventPayload> {
    let mut ops = Vec::new();
    let empty = serde_json::Map::new();
    let from_fields = from.current_fields.as_object().unwrap_or(&empty);
    let to_fields = to.current_fields.as_object().unwrap_or(&empty);
    let mut keys: Vec<&String> = from_fields.keys().chain(to_fields.keys()).collect();
    keys.sort();
    keys.dedup();
    for key in keys {
        if key == "label" {
            // Derived from content; never diffed on its own.
            continue;
        }
        let before = from_fields.get(key);
        let after = to_fields.get(key);
        if before == after {
            continue;
        }
        if key == "content" {
            if let (Some(before), Some(after)) = (before, after) {
                if let Some(patch) = make_patch(before, after) {
                    // A patch keeps anchors mappable, so it is preferred even when
                    // the full value would be smaller.
                    ops.push(EventPayload::RichTextPatched {
                        node_id: to.id.clone(),
                        patch,
                    });
                    continue;
                }
            }
        }
        ops.push(EventPayload::FieldEdited {
            node_id: to.id.clone(),
            field: key.clone(),
            value: after.cloned().unwrap_or(Value::Null),
        });
    }
    ops
}

/// The events that turn one ProseMirror doc into another, keyed on block ids.
#[must_use]
pub fn diff_docs(from_doc: &Value, to_doc: &Value) -> Vec<EventPayload> {
    diff_docs_with_state(&state_from_doc(from_doc), to_doc)
}

/// Like [`diff_docs`] but starting from an existing materialised state, so
/// positions stay stable across edits.
#[must_use]
pub fn diff_docs_with_state(from: &DocState, to_doc: &Value) -> Vec<EventPayload> {
    let to = state_from_doc(to_doc);
    diff_states(from, &to)
}

/// Whether every top-level node of a doc carries an id.
#[must_use]
pub fn all_blocks_have_ids(doc: &Value) -> bool {
    doc.get("content")
        .and_then(Value::as_array)
        .is_none_or(|content| content.iter().all(|node| node_id(node).is_some()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn paragraph(id: &str, text: &str) -> Value {
        json!({"type":"paragraph","attrs":{"id":id},"content":[{"type":"text","text":text}]})
    }

    fn doc(blocks: Vec<Value>) -> Value {
        json!({"type":"doc","content":blocks})
    }

    fn replay(from: &Value, ops: &[EventPayload]) -> Value {
        let mut state = state_from_doc(from);
        for op in ops {
            apply(&mut state, op).unwrap();
        }
        doc_from_state(&state)
    }

    #[test]
    fn diff_reproduces_text_edits_moves_creates_and_deletes() {
        let before = doc(vec![
            paragraph("a", "one"),
            paragraph("b", "two"),
            paragraph("c", "three"),
            paragraph("d", "four"),
        ]);
        let after = doc(vec![
            paragraph("c", "three"),
            paragraph("a", "one!"),
            paragraph("e", "five"),
            paragraph("d", "four"),
        ]);
        let ops = diff_docs(&before, &after);
        let kinds: Vec<&str> = ops.iter().map(EventPayload::event_type).collect();
        assert!(kinds.contains(&"NodeDeleted"));
        assert!(kinds.contains(&"RichTextPatched"));
        assert!(kinds.contains(&"NodeCreated"));
        assert!(kinds.contains(&"NodeMoved"));
        assert_eq!(replay(&before, &ops), after);
    }

    #[test]
    fn unchanged_docs_produce_no_events() {
        let before = doc(vec![paragraph("a", "one"), paragraph("b", "two")]);
        assert!(diff_docs(&before, &before).is_empty());
    }

    #[test]
    fn labels_follow_content() {
        let before = doc(vec![paragraph("a", "one")]);
        let after = doc(vec![paragraph("a", "one and two")]);
        let ops = diff_docs(&before, &after);
        let mut state = state_from_doc(&before);
        for op in &ops {
            apply(&mut state, op).unwrap();
        }
        assert_eq!(state.nodes["a"].label(), "one and two");
    }

    #[test]
    fn split_paragraph_keeps_the_first_id() {
        let before = doc(vec![paragraph("a", "one two")]);
        let after = doc(vec![paragraph("a", "one"), paragraph("b", "two")]);
        let ops = diff_docs(&before, &after);
        assert_eq!(replay(&before, &ops), after);
        let created: Vec<&str> = ops
            .iter()
            .filter_map(|op| match op {
                EventPayload::NodeCreated { node_id, .. } => Some(node_id.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(created, vec!["b"]);
    }

    #[test]
    fn deep_reorders_only_move_the_minority() {
        let before = doc((0..10)
            .map(|i| paragraph(&format!("n{i}"), &i.to_string()))
            .collect());
        let mut blocks: Vec<Value> = (0..10)
            .map(|i| paragraph(&format!("n{i}"), &i.to_string()))
            .collect();
        let moved = blocks.remove(9);
        blocks.insert(0, moved);
        let after = doc(blocks);
        let ops = diff_docs(&before, &after);
        assert_eq!(ops.len(), 1);
        assert!(matches!(ops[0], EventPayload::NodeMoved { .. }));
        assert_eq!(replay(&before, &ops), after);
    }
}
