//! Blame for prose: for each block, which event last wrote each range of its
//! text. Built by folding the log and mapping attribution through patches.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::events::{EventPayload, EventRecord};
use crate::materialize::{DocState, apply};
use crate::patch::{TextOp, root_text_ops};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export, export_to = "blocks/"))]
pub struct BlameSpan {
    pub start: u32,
    pub end: u32,
    pub seq: i64,
    pub actor: String,
    pub at: String,
}

#[derive(Clone)]
struct Attribution {
    seq: i64,
    actor: String,
    at: String,
}

/// Attribution per block, as spans over the current text in scalar offsets.
#[must_use]
pub fn blame(records: &[EventRecord]) -> BTreeMap<String, Vec<BlameSpan>> {
    let mut state = DocState::default();
    // Per block, one attribution per scalar.
    let mut scalars: BTreeMap<String, Vec<Attribution>> = BTreeMap::new();
    for record in records {
        let attribution = Attribution {
            seq: record.seq,
            actor: record.actor.clone(),
            at: record.at.clone(),
        };
        match &record.payload {
            EventPayload::RichTextPatched { node_id, patch } => {
                if apply(&mut state, &record.payload).is_err() {
                    continue;
                }
                let new_len = state
                    .nodes
                    .get(node_id)
                    .map(|node| node.label().chars().count())
                    .unwrap_or(0);
                match root_text_ops(patch) {
                    Some(ops) => {
                        let existing = scalars.remove(node_id).unwrap_or_default();
                        scalars.insert(
                            node_id.clone(),
                            map_attribution(&existing, ops, &attribution),
                        );
                    }
                    None => {
                        scalars.insert(node_id.clone(), vec![attribution; new_len]);
                    }
                }
            }
            EventPayload::NodeCreated { node_id, .. }
            | EventPayload::FieldEdited { node_id, .. } => {
                let touches_text = matches!(&record.payload, EventPayload::NodeCreated { .. })
                    || matches!(&record.payload, EventPayload::FieldEdited { field, .. } if field == "content");
                if apply(&mut state, &record.payload).is_err() {
                    continue;
                }
                if touches_text {
                    let len = state
                        .nodes
                        .get(node_id)
                        .map(|node| node.label().chars().count())
                        .unwrap_or(0);
                    scalars.insert(node_id.clone(), vec![attribution; len]);
                }
            }
            EventPayload::NodeDeleted { node_id } => {
                let _ = apply(&mut state, &record.payload);
                scalars.remove(node_id);
            }
            EventPayload::NodeRestored { node_id } => {
                if apply(&mut state, &record.payload).is_ok() {
                    let len = state
                        .nodes
                        .get(node_id)
                        .map(|node| node.label().chars().count())
                        .unwrap_or(0);
                    scalars
                        .entry(node_id.clone())
                        .or_insert_with(|| vec![attribution; len]);
                }
            }
            _ => {
                let _ = apply(&mut state, &record.payload);
            }
        }
    }
    scalars
        .into_iter()
        .map(|(block, attributions)| (block, compress(&attributions)))
        .collect()
}

fn map_attribution(
    existing: &[Attribution],
    ops: &[TextOp],
    current: &Attribution,
) -> Vec<Attribution> {
    let mut out = Vec::with_capacity(existing.len());
    let mut cursor = 0usize;
    for op in ops {
        match op {
            TextOp::Retain { count, .. } => {
                let end = (cursor + count).min(existing.len());
                out.extend_from_slice(&existing[cursor..end]);
                cursor = end;
            }
            TextOp::Delete { count } => {
                cursor = (cursor + count).min(existing.len());
            }
            TextOp::Insert { text, .. } => {
                out.extend(std::iter::repeat_n(current.clone(), text.chars().count()));
            }
        }
    }
    out
}

fn compress(attributions: &[Attribution]) -> Vec<BlameSpan> {
    let mut spans: Vec<BlameSpan> = Vec::new();
    for (index, attribution) in attributions.iter().enumerate() {
        match spans.last_mut() {
            Some(last) if last.seq == attribution.seq => last.end = index as u32 + 1,
            _ => spans.push(BlameSpan {
                start: index as u32,
                end: index as u32 + 1,
                seq: attribution.seq,
                actor: attribution.actor.clone(),
                at: attribution.at.clone(),
            }),
        }
    }
    spans
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::next_record;
    use crate::materialize::{diff_docs, state_from_doc};
    use serde_json::json;

    fn paragraph(id: &str, text: &str) -> serde_json::Value {
        json!({"type":"paragraph","attrs":{"id":id},"content":[{"type":"text","text":text}]})
    }

    #[test]
    fn attributes_inserted_text_to_the_editing_event() {
        let v1 = json!({"type":"doc","content":[paragraph("a", "Mira waited.")]});
        let v2 = json!({"type":"doc","content":[paragraph("a", "Mira waited for Oren.")]});
        let mut records = Vec::new();
        let mut prev: Option<EventRecord> = None;
        for op in diff_docs(&json!({"type":"doc","content":[]}), &v1) {
            let record = next_record(prev.as_ref(), op, "author", "t1");
            prev = Some(record.clone());
            records.push(record);
        }
        for op in crate::materialize::diff_states(&state_from_doc(&v1), &state_from_doc(&v2)) {
            let record = next_record(prev.as_ref(), op, "editor", "t2");
            prev = Some(record.clone());
            records.push(record);
        }
        let blame = blame(&records);
        let spans = &blame["a"];
        assert_eq!(spans[0].actor, "author");
        assert!(
            spans
                .iter()
                .any(|span| span.actor == "editor" && span.start == 11 && span.end == 20)
        );
        assert_eq!(spans.last().unwrap().actor, "author");
    }
}
