//! State-based three-way merge of two document states against their common
//! base, after `dynodoc-engine`'s `workspace_merge`. Disjoint edits combine;
//! competing values and overlapping wording surface as conflicts, which the
//! author resolves. Nested structures (lists, tables) merge only when one side
//! left them alone.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::events::EventPayload;
use crate::materialize::{DocState, MaterializedNode, diff_states};
use crate::patch::{Hunk, Styles, align, children, is_inline_container, props_of, scalars};
use crate::plain_text;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Conflict {
    /// `"{node_id}:{field}"`.
    pub key: String,
    pub node_id: String,
    /// A field name, `$node` for a whole-node disagreement, or `$location`.
    pub field: String,
    pub base: Value,
    pub ours: Value,
    pub theirs: Value,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MergeResult {
    /// The merged state, with conflicts resolved towards `ours` unless a
    /// resolution says otherwise.
    pub state: DocState,
    pub conflicts: Vec<Conflict>,
    /// The events that turn `ours` into the merged state.
    pub ops: Vec<EventPayload>,
}

/// Which side wins a conflicted key: `"ours"` or `"theirs"`.
pub type Resolutions = BTreeMap<String, String>;

/// Merge `theirs` into `ours` relative to `base`.
#[must_use]
pub fn three_way(
    base: &DocState,
    ours: &DocState,
    theirs: &DocState,
    resolutions: &Resolutions,
) -> MergeResult {
    let mut out = ours.clone();
    let mut conflicts = Vec::new();

    for (id, their_node) in &theirs.nodes {
        let base_node = base.nodes.get(id);
        if base_node == Some(their_node) {
            continue; // theirs did not touch it
        }
        let our_node = ours.nodes.get(id);
        match (base_node, our_node) {
            // New on their side only: take it.
            (None, None) => {
                out.nodes.insert(id.clone(), their_node.clone());
            }
            // Both created the same id independently (or ours deleted/changed a node theirs created).
            (None, Some(our_node)) => {
                if our_node != their_node {
                    whole_node(
                        id,
                        &Value::Null,
                        our_node,
                        their_node,
                        resolutions,
                        &mut out,
                        &mut conflicts,
                    );
                }
            }
            (Some(base_node), None) => {
                // Ours removed the node entirely (should not happen with tombstones); treat as deleted.
                let deleted = MaterializedNode {
                    deleted: true,
                    ..base_node.clone()
                };
                whole_node(
                    id,
                    &json(base_node),
                    &deleted,
                    their_node,
                    resolutions,
                    &mut out,
                    &mut conflicts,
                );
            }
            (Some(base_node), Some(our_node)) => {
                if our_node == base_node {
                    out.nodes.insert(id.clone(), their_node.clone());
                    continue;
                }
                if our_node == their_node {
                    continue;
                }
                if our_node.deleted != their_node.deleted
                    || our_node.node_type != their_node.node_type
                {
                    whole_node(
                        id,
                        &json(base_node),
                        our_node,
                        their_node,
                        resolutions,
                        &mut out,
                        &mut conflicts,
                    );
                    continue;
                }
                let merged = out.nodes.get_mut(id).expect("present");
                // Location.
                let base_loc = (base_node.parent_id.clone(), base_node.pos.clone());
                let our_loc = (our_node.parent_id.clone(), our_node.pos.clone());
                let their_loc = (their_node.parent_id.clone(), their_node.pos.clone());
                if their_loc != base_loc && our_loc != their_loc {
                    if our_loc == base_loc {
                        merged.parent_id = their_node.parent_id.clone();
                        merged.pos = their_node.pos.clone();
                    } else {
                        let key = format!("{id}:$location");
                        if resolutions.get(&key).map(String::as_str) == Some("theirs") {
                            merged.parent_id = their_node.parent_id.clone();
                            merged.pos = their_node.pos.clone();
                        } else {
                            conflicts.push(Conflict {
                                key,
                                node_id: id.clone(),
                                field: "$location".into(),
                                base: serde_json::json!({"parent_id": base_loc.0, "pos": base_loc.1}),
                                ours: serde_json::json!({"parent_id": our_loc.0, "pos": our_loc.1}),
                                theirs: serde_json::json!({"parent_id": their_loc.0, "pos": their_loc.1}),
                            });
                        }
                    }
                }
                // Fields.
                let empty = serde_json::Map::new();
                let base_fields = base_node.current_fields.as_object().unwrap_or(&empty);
                let our_fields = our_node.current_fields.as_object().unwrap_or(&empty);
                let their_fields = their_node.current_fields.as_object().unwrap_or(&empty);
                let mut keys: Vec<&String> = base_fields
                    .keys()
                    .chain(their_fields.keys())
                    .chain(our_fields.keys())
                    .collect();
                keys.sort();
                keys.dedup();
                for key in keys {
                    if key == "label" {
                        continue;
                    }
                    let b = base_fields.get(key).cloned().unwrap_or(Value::Null);
                    let o = our_fields.get(key).cloned().unwrap_or(Value::Null);
                    let t = their_fields.get(key).cloned().unwrap_or(Value::Null);
                    if t == b || o == t {
                        continue;
                    }
                    let chosen = if o == b {
                        Some(t.clone())
                    } else if key == "content" {
                        merge_content(&b, &o, &t)
                    } else {
                        None
                    };
                    let value = match chosen {
                        Some(value) => value,
                        None => {
                            let conflict_key = format!("{id}:{key}");
                            match resolutions.get(&conflict_key).map(String::as_str) {
                                Some("theirs") => t.clone(),
                                Some(_) => o.clone(),
                                None => {
                                    conflicts.push(Conflict {
                                        key: conflict_key,
                                        node_id: id.clone(),
                                        field: key.clone(),
                                        base: b.clone(),
                                        ours: o.clone(),
                                        theirs: t.clone(),
                                    });
                                    o.clone()
                                }
                            }
                        }
                    };
                    if let Some(fields) = merged.current_fields.as_object_mut() {
                        fields.insert(key.clone(), value);
                    }
                }
                if let Some(content) = merged.current_fields.get("content").cloned() {
                    let label = plain_text(&content);
                    if let Some(fields) = merged.current_fields.as_object_mut() {
                        fields.insert("label".into(), Value::String(label));
                    }
                }
            }
        }
    }

    // Metadata merges per key the same way.
    for (key, their_value) in &theirs.meta {
        let base_value = base.meta.get(key);
        if base_value == Some(their_value) {
            continue;
        }
        match ours.meta.get(key) {
            None => {
                out.meta.insert(key.clone(), their_value.clone());
            }
            Some(our_value) if our_value == their_value => {}
            Some(our_value) if Some(our_value) == base_value => {
                out.meta.insert(key.clone(), their_value.clone());
            }
            Some(our_value) => {
                let conflict_key = format!("$meta:{key}");
                match resolutions.get(&conflict_key).map(String::as_str) {
                    Some("theirs") => {
                        out.meta.insert(key.clone(), their_value.clone());
                    }
                    Some(_) => {}
                    None => conflicts.push(Conflict {
                        key: conflict_key,
                        node_id: String::new(),
                        field: key.clone(),
                        base: base_value.cloned().unwrap_or(Value::Null),
                        ours: our_value.clone(),
                        theirs: their_value.clone(),
                    }),
                }
            }
        }
    }

    let ops = diff_states(ours, &out);
    MergeResult {
        state: out,
        conflicts,
        ops,
    }
}

fn json(node: &MaterializedNode) -> Value {
    serde_json::to_value(node).unwrap_or(Value::Null)
}

fn whole_node(
    id: &str,
    base: &Value,
    ours: &MaterializedNode,
    theirs: &MaterializedNode,
    resolutions: &Resolutions,
    out: &mut DocState,
    conflicts: &mut Vec<Conflict>,
) {
    let key = format!("{id}:$node");
    match resolutions.get(&key).map(String::as_str) {
        Some("theirs") => {
            out.nodes.insert(id.to_owned(), theirs.clone());
        }
        Some(_) => {}
        None => conflicts.push(Conflict {
            key,
            node_id: id.to_owned(),
            field: "$node".into(),
            base: base.clone(),
            ours: json(ours),
            theirs: json(theirs),
        }),
    }
}

/// Two hunks overlap when their base ranges intersect, counting an insertion at
/// a point as overlapping any hunk that touches that point.
fn overlaps(a: &Hunk, b: &Hunk) -> bool {
    let a_point = a.base_start == a.base_end;
    let b_point = b.base_start == b.base_end;
    match (a_point, b_point) {
        (true, true) => a.base_start == b.base_start,
        (true, false) => b.base_start <= a.base_start && a.base_start <= b.base_end,
        (false, true) => a.base_start <= b.base_start && b.base_start <= a.base_end,
        (false, false) => a.base_start < b.base_end && b.base_start < a.base_end,
    }
}

/// Merge two edited versions of one inline block against their base. `None`
/// means the edits overlap or the structure diverged.
#[must_use]
pub fn merge_content(base: &Value, ours: &Value, theirs: &Value) -> Option<Value> {
    if theirs == base || ours == theirs {
        return Some(ours.clone());
    }
    if ours == base {
        return Some(theirs.clone());
    }
    if !(is_inline_container(base) && is_inline_container(ours) && is_inline_container(theirs)) {
        return None;
    }
    if base.get("type") != ours.get("type") || base.get("type") != theirs.get("type") {
        return None;
    }
    // Properties: take whichever side changed them, or fail when both did differently.
    let (pb, po, pt) = (props_of(base), props_of(ours), props_of(theirs));
    let props = if pt == pb || po == pt {
        po
    } else if po == pb {
        pt
    } else {
        return None;
    };

    let mut styles = Styles(Vec::new());
    let b = scalars(base, &mut styles);
    let o = scalars(ours, &mut styles);
    let t = scalars(theirs, &mut styles);
    let (hunks_o, kept_o) = align(&b, &o);
    let (hunks_t, kept_t) = align(&b, &t);
    for ho in &hunks_o {
        if hunks_t.iter().any(|ht| overlaps(ho, ht)) {
            return None;
        }
    }
    let mut hunks: Vec<(&Hunk, bool)> = hunks_o
        .iter()
        .map(|h| (h, true))
        .chain(hunks_t.iter().map(|h| (h, false)))
        .collect();
    hunks.sort_by_key(|(hunk, _)| (hunk.base_start, hunk.base_end));

    let mut result: Vec<(char, usize)> = Vec::with_capacity(b.len().max(o.len()).max(t.len()));
    let mut cursor = 0usize;
    let copy_base = |from: usize, to: usize, result: &mut Vec<(char, usize)>| -> Option<()> {
        for k in from..to {
            let base_style = b[k].1;
            let our_style = kept_o[k].map(|j| o[j].1);
            let their_style = kept_t[k].map(|j| t[j].1);
            let style = match (our_style, their_style) {
                (Some(os), Some(ts)) => {
                    if os == base_style {
                        ts
                    } else if ts == base_style || os == ts {
                        os
                    } else {
                        return None;
                    }
                }
                (Some(os), None) => os,
                (None, Some(ts)) => ts,
                (None, None) => base_style,
            };
            result.push((b[k].0, style));
        }
        Some(())
    };
    for (hunk, _) in hunks {
        if hunk.base_start < cursor {
            return None;
        }
        copy_base(cursor, hunk.base_start, &mut result)?;
        result.extend(hunk.replacement.iter().copied());
        cursor = hunk.base_end;
    }
    copy_base(cursor, b.len(), &mut result)?;

    let rebuilt = children(&result, &styles.0).ok()?;
    let mut node = props;
    if let Some(object) = node.as_object_mut() {
        if rebuilt.is_empty() {
            object.remove("content");
        } else {
            object.insert("content".into(), Value::Array(rebuilt));
        }
    }
    Some(node)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::materialize::state_from_doc;
    use serde_json::json;

    fn paragraph(id: &str, text: &str) -> Value {
        json!({"type":"paragraph","attrs":{"id":id},"content":[{"type":"text","text":text}]})
    }

    fn doc(blocks: Vec<Value>) -> Value {
        json!({"type":"doc","content":blocks})
    }

    #[test]
    fn disjoint_inline_edits_combine() {
        let base = paragraph("a", "Mira pushed her hair back and waited.");
        let ours = paragraph("a", "Mira pushed her auburn hair back and waited.");
        let theirs = paragraph("a", "Mira pushed her hair back and waited for Oren.");
        let merged = merge_content(&base, &ours, &theirs).expect("merge");
        assert_eq!(
            plain_text(&merged),
            "Mira pushed her auburn hair back and waited for Oren."
        );
    }

    #[test]
    fn overlapping_edits_conflict() {
        let base = paragraph("a", "Mira pushed her hair back.");
        let ours = paragraph("a", "Mira pushed her auburn hair back.");
        let theirs = paragraph("a", "Mira pushed her black hair back.");
        assert!(merge_content(&base, &ours, &theirs).is_none());
    }

    #[test]
    fn document_merge_reports_conflicts_and_ops() {
        let base = state_from_doc(&doc(vec![paragraph("a", "one"), paragraph("b", "two")]));
        let ours = state_from_doc(&doc(vec![paragraph("a", "one!"), paragraph("b", "two")]));
        let mut theirs_doc = doc(vec![
            paragraph("a", "one"),
            paragraph("b", "two?"),
            paragraph("c", "three"),
        ]);
        // Keep positions comparable with the base by deriving theirs from base edits.
        let theirs = {
            let ops = crate::materialize::diff_states(&base, &state_from_doc(&theirs_doc));
            let mut state = base.clone();
            for op in &ops {
                crate::materialize::apply(&mut state, op).unwrap();
            }
            state
        };
        theirs_doc = Value::Null;
        let _ = theirs_doc;
        let result = three_way(&base, &ours, &theirs, &Resolutions::new());
        assert!(result.conflicts.is_empty(), "{:?}", result.conflicts);
        let merged = crate::materialize::doc_from_state(&result.state);
        assert_eq!(plain_text(&merged), "one!\ntwo?\nthree");
        assert!(!result.ops.is_empty());

        // Both edit the same word: a conflict on content, resolved towards ours by default.
        let ours2 = state_from_doc(&doc(vec![paragraph("a", "uno"), paragraph("b", "two")]));
        let theirs2 = state_from_doc(&doc(vec![paragraph("a", "ein"), paragraph("b", "two")]));
        let result = three_way(&base, &ours2, &theirs2, &Resolutions::new());
        assert_eq!(result.conflicts.len(), 1);
        assert_eq!(result.conflicts[0].field, "content");
        let mut resolutions = Resolutions::new();
        resolutions.insert("a:content".into(), "theirs".into());
        let resolved = three_way(&base, &ours2, &theirs2, &resolutions);
        assert!(resolved.conflicts.is_empty());
        assert_eq!(resolved.state.nodes["a"].label(), "ein");
    }
}
