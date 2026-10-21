//! Rich-text patches: the Dynodoc v1 wire format, with a deterministic Myers
//! diff, base-checked application, and position mapping for anchors.
//!
//! Counts are Unicode scalar values. A patch applies only to the exact
//! fingerprinted base and must reproduce the exact fingerprinted result.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

use crate::canonical::fingerprint;

const MAX_SCALARS: usize = 100_000;
const MAX_EDITS: usize = 10_000;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export, export_to = "blocks/"))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct RichTextPatch {
    pub version: u8,
    /// Fingerprint of the block before the patch.
    pub before: String,
    /// Fingerprint of the block after the patch.
    pub after: String,
    /// Interned text-node metadata (`{"type":"text","marks":[…]}` without `text`).
    pub styles: Vec<Value>,
    pub edits: Vec<RichTextEdit>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export, export_to = "blocks/"))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RichTextEdit {
    /// Edit the inline text of the node at `path` (child indices from the block root).
    Text { path: Vec<usize>, ops: Vec<TextOp> },
    /// Replace every non-`content` property of the node at `path`.
    Props { path: Vec<usize>, value: Value },
    /// Replace the whole subtree at `path`.
    Replace { path: Vec<usize>, value: Value },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export, export_to = "blocks/"))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TextOp {
    Retain {
        count: usize,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        style: Option<usize>,
    },
    Delete {
        count: usize,
    },
    Insert {
        text: String,
        style: usize,
    },
}

#[derive(Debug, Error, Eq, PartialEq)]
#[error("invalid rich-text patch: {0}")]
pub struct PatchError(pub &'static str);

// ───────────────────────── inline containers ─────────────────────────

/// A scalar with its style index.
pub(crate) type Styled = (char, usize);

pub(crate) struct Styles(pub(crate) Vec<Value>);

impl Styles {
    pub(crate) fn intern(&mut self, text_node: &Value) -> usize {
        let mut style = text_node.clone();
        if let Some(object) = style.as_object_mut() {
            object.remove("text");
        }
        if let Some(index) = self.0.iter().position(|existing| *existing == style) {
            return index;
        }
        self.0.push(style);
        self.0.len() - 1
    }
}

pub(crate) fn is_inline_container(node: &Value) -> bool {
    let node_type = node.get("type").and_then(Value::as_str).unwrap_or("");
    if !matches!(node_type, "paragraph" | "heading" | "codeBlock") {
        return false;
    }
    let Some(children) = node.get("content").and_then(Value::as_array) else {
        return true;
    };
    children.iter().all(|child| {
        child.get("type").and_then(Value::as_str) == Some("text") && child.get("content").is_none()
    })
}

/// Flatten an inline container into styled scalars.
pub(crate) fn scalars(node: &Value, styles: &mut Styles) -> Vec<Styled> {
    let mut out = Vec::new();
    if let Some(children) = node.get("content").and_then(Value::as_array) {
        for child in children {
            let style = styles.intern(child);
            if let Some(text) = child.get("text").and_then(Value::as_str) {
                out.extend(text.chars().map(|ch| (ch, style)));
            }
        }
    }
    out
}

/// Rebuild text nodes from styled scalars, coalescing runs whose style values
/// are equal (two indices may name the same style).
pub(crate) fn children(scalars: &[Styled], styles: &[Value]) -> Result<Vec<Value>, PatchError> {
    let mut out = Vec::new();
    let mut run = String::new();
    let mut current: Option<&Value> = None;
    let flush =
        |run: &mut String, style: Option<&Value>, out: &mut Vec<Value>| -> Result<(), PatchError> {
            if run.is_empty() {
                return Ok(());
            }
            let Some(style) = style else {
                return Ok(());
            };
            let mut node = style.clone();
            let object = node
                .as_object_mut()
                .ok_or(PatchError("style is not an object"))?;
            if object.get("type").and_then(Value::as_str) != Some("text")
                || object.contains_key("content")
            {
                return Err(PatchError("style is not a text node"));
            }
            object.insert("text".into(), Value::String(std::mem::take(run)));
            out.push(node);
            Ok(())
        };
    for (ch, style) in scalars {
        let value = styles
            .get(*style)
            .ok_or(PatchError("unknown style index"))?;
        if current != Some(value) {
            flush(&mut run, current, &mut out)?;
            current = Some(value);
        }
        run.push(*ch);
    }
    flush(&mut run, current, &mut out)?;
    Ok(out)
}

pub(crate) fn props_of(node: &Value) -> Value {
    let mut props = node.clone();
    if let Some(object) = props.as_object_mut() {
        object.remove("content");
    }
    props
}

// ───────────────────────── Myers diff ─────────────────────────

/// Deterministic Myers diff over two scalar sequences, compared by character.
/// Returns coalesced ops with styles taken from `after`.
fn diff_scalars(before: &[Styled], after: &[Styled]) -> Vec<TextOp> {
    let n = before.len();
    let m = after.len();
    // Edit script as (op, index) pairs: 0 = keep, 1 = delete, 2 = insert.
    let script = myers(before, after);
    let mut ops: Vec<TextOp> = Vec::new();
    let mut i = 0;
    let mut j = 0;
    let push = |ops: &mut Vec<TextOp>, op: TextOp| match (ops.last_mut(), op) {
        (
            Some(TextOp::Retain {
                count,
                style: existing,
            }),
            TextOp::Retain { count: more, style },
        ) if *existing == style => *count += more,
        (Some(TextOp::Delete { count }), TextOp::Delete { count: more }) => *count += more,
        (
            Some(TextOp::Insert {
                text,
                style: existing,
            }),
            TextOp::Insert { text: more, style },
        ) if *existing == style => text.push_str(&more),
        (_, op) => ops.push(op),
    };
    for step in script {
        match step {
            Step::Keep => {
                let style = if before[i].1 == after[j].1 {
                    None
                } else {
                    Some(after[j].1)
                };
                push(&mut ops, TextOp::Retain { count: 1, style });
                i += 1;
                j += 1;
            }
            Step::Delete => {
                push(&mut ops, TextOp::Delete { count: 1 });
                i += 1;
            }
            Step::Insert => {
                push(
                    &mut ops,
                    TextOp::Insert {
                        text: after[j].0.to_string(),
                        style: after[j].1,
                    },
                );
                j += 1;
            }
        }
    }
    debug_assert_eq!(i, n);
    debug_assert_eq!(j, m);
    // Order deletes before inserts within a changed region, as Dynodoc does.
    normalise(ops)
}

/// A changed region of a base sequence and what one side put in its place.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Hunk {
    pub(crate) base_start: usize,
    pub(crate) base_end: usize,
    pub(crate) replacement: Vec<Styled>,
}

/// Align `side` against `base`: the changed regions and, for every kept base
/// scalar, the index of its counterpart in `side`.
pub(crate) fn align(base: &[Styled], side: &[Styled]) -> (Vec<Hunk>, Vec<Option<usize>>) {
    let steps = myers(base, side);
    let mut hunks: Vec<Hunk> = Vec::new();
    let mut kept: Vec<Option<usize>> = vec![None; base.len()];
    let mut i = 0usize;
    let mut j = 0usize;
    let mut open: Option<Hunk> = None;
    for step in steps {
        match step {
            Step::Keep => {
                if let Some(hunk) = open.take() {
                    hunks.push(hunk);
                }
                kept[i] = Some(j);
                i += 1;
                j += 1;
            }
            Step::Delete => {
                let hunk = open.get_or_insert_with(|| Hunk {
                    base_start: i,
                    base_end: i,
                    replacement: Vec::new(),
                });
                hunk.base_end = i + 1;
                i += 1;
            }
            Step::Insert => {
                let hunk = open.get_or_insert_with(|| Hunk {
                    base_start: i,
                    base_end: i,
                    replacement: Vec::new(),
                });
                hunk.replacement.push(side[j]);
                j += 1;
            }
        }
    }
    if let Some(hunk) = open.take() {
        hunks.push(hunk);
    }
    (hunks, kept)
}

#[derive(Clone, Copy)]
enum Step {
    Keep,
    Delete,
    Insert,
}

/// The largest edit distance the minimal diff will search. Beyond it the
/// block is treated as rewritten (delete all, insert all), which keeps memory
/// at O(D²) rather than O((N+M)·D) and time at O((N+M)·D).
const MAX_EDIT_DISTANCE: usize = 1_500;

/// Myers with a trace, after trimming the common prefix and suffix. Produces
/// a minimal edit script up to [`MAX_EDIT_DISTANCE`], else a whole rewrite.
fn myers(a: &[Styled], b: &[Styled]) -> Vec<Step> {
    let prefix = a
        .iter()
        .zip(b.iter())
        .take_while(|(x, y)| x.0 == y.0)
        .count();
    let suffix = a[prefix..]
        .iter()
        .rev()
        .zip(b[prefix..].iter().rev())
        .take_while(|(x, y)| x.0 == y.0)
        .count();
    let core_a = &a[prefix..a.len() - suffix];
    let core_b = &b[prefix..b.len() - suffix];
    let mut steps = vec![Step::Keep; prefix];
    steps.extend(myers_core(core_a, core_b));
    steps.extend(std::iter::repeat_n(Step::Keep, suffix));
    steps
}

fn myers_core(a: &[Styled], b: &[Styled]) -> Vec<Step> {
    let n = a.len() as isize;
    let m = b.len() as isize;
    if n == 0 {
        return vec![Step::Insert; m as usize];
    }
    if m == 0 {
        return vec![Step::Delete; n as usize];
    }
    let max = (n + m).min(MAX_EDIT_DISTANCE as isize);
    // `v` holds the furthest x for each diagonal k in [-max, max].
    let offset = max;
    let width = (2 * max + 1) as usize;
    let mut v: Vec<isize> = vec![0; width];
    // The trace keeps only the live slice [-d, d] of each round: O(D²) total.
    let mut trace: Vec<Vec<isize>> = Vec::new();
    let mut found = false;
    'outer: for d in 0..=max {
        let lo = (offset - d) as usize;
        let hi = (offset + d) as usize;
        trace.push(v[lo..=hi].to_vec());
        let mut k = -d;
        while k <= d {
            let index = (k + offset) as usize;
            let mut x = if k == -d || (k != d && v[index - 1] < v[index + 1]) {
                v[index + 1]
            } else {
                v[index - 1] + 1
            };
            let mut y = x - k;
            while x < n && y < m && a[x as usize].0 == b[y as usize].0 {
                x += 1;
                y += 1;
            }
            v[index] = x;
            if x >= n && y >= m {
                found = true;
                break 'outer;
            }
            k += 2;
        }
    }
    if !found {
        // Too different to be worth a minimal script: a rewrite.
        let mut steps = vec![Step::Delete; n as usize];
        steps.extend(std::iter::repeat_n(Step::Insert, m as usize));
        return steps;
    }
    // Backtrack through the live slices.
    let mut steps = Vec::new();
    let mut x = n;
    let mut y = m;
    for (d, slice) in trace.iter().enumerate().rev() {
        let d = d as isize;
        if d == 0 {
            // Round zero is the opening snake along diagonal 0: all keeps.
            debug_assert_eq!(x, y);
            steps.extend(std::iter::repeat_n(Step::Keep, x.max(0) as usize));
            break;
        }
        let at = |k: isize| slice[(k + d) as usize];
        let k = x - y;
        let prev_k = if k == -d || (k != d && at(k - 1) < at(k + 1)) {
            k + 1
        } else {
            k - 1
        };
        let prev_x = at(prev_k);
        let prev_y = prev_x - prev_k;
        while x > prev_x && y > prev_y {
            steps.push(Step::Keep);
            x -= 1;
            y -= 1;
        }
        if d > 0 {
            if x == prev_x {
                steps.push(Step::Insert);
            } else {
                steps.push(Step::Delete);
            }
        }
        x = prev_x;
        y = prev_y;
    }
    steps.reverse();
    steps
}

/// Within each run of non-retain ops, put deletes first, then inserts, then coalesce.
fn normalise(ops: Vec<TextOp>) -> Vec<TextOp> {
    let mut out: Vec<TextOp> = Vec::new();
    let mut deletes = 0usize;
    let mut inserts: Vec<TextOp> = Vec::new();
    let flush = |out: &mut Vec<TextOp>, deletes: &mut usize, inserts: &mut Vec<TextOp>| {
        if *deletes > 0 {
            out.push(TextOp::Delete { count: *deletes });
            *deletes = 0;
        }
        for insert in inserts.drain(..) {
            match (out.last_mut(), insert) {
                (
                    Some(TextOp::Insert {
                        text,
                        style: existing,
                    }),
                    TextOp::Insert { text: more, style },
                ) if *existing == style => text.push_str(&more),
                (_, insert) => out.push(insert),
            }
        }
    };
    for op in ops {
        match op {
            TextOp::Retain { .. } => {
                flush(&mut out, &mut deletes, &mut inserts);
                match (out.last_mut(), &op) {
                    (
                        Some(TextOp::Retain {
                            count,
                            style: existing,
                        }),
                        TextOp::Retain { count: more, style },
                    ) if existing == style => *count += more,
                    _ => out.push(op),
                }
            }
            TextOp::Delete { count } => deletes += count,
            insert @ TextOp::Insert { .. } => inserts.push(insert),
        }
    }
    flush(&mut out, &mut deletes, &mut inserts);
    out
}

// ───────────────────────── make and apply ─────────────────────────

/// Build a patch from `before` to `after`, or `None` when no patch can express
/// the change (the caller then falls back to replacing the block).
#[must_use]
pub fn make_patch(before: &Value, after: &Value) -> Option<RichTextPatch> {
    if before == after {
        return None;
    }
    let mut styles = Styles(Vec::new());
    let mut edits = Vec::new();
    build(before, after, &mut Vec::new(), &mut styles, &mut edits);
    if edits.is_empty() || edits.len() > MAX_EDITS {
        return None;
    }
    let styles = compact_styles(styles.0, &mut edits);
    let patch = RichTextPatch {
        version: 1,
        before: fingerprint(before),
        after: fingerprint(after),
        styles,
        edits,
    };
    // Self-verifying: the patch must reproduce `after` exactly.
    match apply_patch(before, &patch) {
        Ok(result) if result == *after => Some(patch),
        _ => None,
    }
}

/// Keep only the styles the ops reference, renumbering them in first-use order.
fn compact_styles(table: Vec<Value>, edits: &mut [RichTextEdit]) -> Vec<Value> {
    let mut used: Vec<usize> = Vec::new();
    let mut remap = |style: &mut usize| {
        let position = match used.iter().position(|existing| existing == style) {
            Some(position) => position,
            None => {
                used.push(*style);
                used.len() - 1
            }
        };
        *style = position;
    };
    for edit in edits.iter_mut() {
        if let RichTextEdit::Text { ops, .. } = edit {
            for op in ops {
                match op {
                    TextOp::Retain {
                        style: Some(style), ..
                    }
                    | TextOp::Insert { style, .. } => remap(style),
                    _ => {}
                }
            }
        }
    }
    used.into_iter().map(|index| table[index].clone()).collect()
}

fn build(
    a: &Value,
    b: &Value,
    path: &mut Vec<usize>,
    styles: &mut Styles,
    edits: &mut Vec<RichTextEdit>,
) {
    if a == b {
        return;
    }
    let same_type = a.get("type") == b.get("type");
    if same_type && is_inline_container(a) && is_inline_container(b) {
        let before = scalars(a, styles);
        let after = scalars(b, styles);
        if before.len() > MAX_SCALARS || after.len() > MAX_SCALARS {
            edits.push(RichTextEdit::Replace {
                path: path.clone(),
                value: b.clone(),
            });
            return;
        }
        if props_of(a) != props_of(b) {
            edits.push(RichTextEdit::Props {
                path: path.clone(),
                value: props_of(b),
            });
        }
        if before != after {
            edits.push(RichTextEdit::Text {
                path: path.clone(),
                ops: diff_scalars(&before, &after),
            });
        }
        return;
    }
    let children_a = a.get("content").and_then(Value::as_array);
    let children_b = b.get("content").and_then(Value::as_array);
    match (children_a, children_b) {
        (Some(ca), Some(cb)) if same_type && ca.len() == cb.len() => {
            if props_of(a) != props_of(b) {
                edits.push(RichTextEdit::Props {
                    path: path.clone(),
                    value: props_of(b),
                });
            }
            for (index, (child_a, child_b)) in ca.iter().zip(cb).enumerate() {
                path.push(index);
                build(child_a, child_b, path, styles, edits);
                path.pop();
            }
        }
        _ => edits.push(RichTextEdit::Replace {
            path: path.clone(),
            value: b.clone(),
        }),
    }
}

fn node_at_mut<'a>(root: &'a mut Value, path: &[usize]) -> Result<&'a mut Value, PatchError> {
    let mut node = root;
    for index in path {
        node = node
            .get_mut("content")
            .and_then(Value::as_array_mut)
            .and_then(|children| children.get_mut(*index))
            .ok_or(PatchError("path does not exist"))?;
    }
    Ok(node)
}

/// Apply a patch to its exact base.
pub fn apply_patch(before: &Value, patch: &RichTextPatch) -> Result<Value, PatchError> {
    if patch.version != 1 {
        return Err(PatchError("unsupported version"));
    }
    if fingerprint(before) != patch.before {
        return Err(PatchError("base fingerprint mismatch"));
    }
    if patch.edits.len() > MAX_EDITS || patch.styles.len() > MAX_EDITS {
        return Err(PatchError("too many edits"));
    }
    let mut out = before.clone();
    for edit in &patch.edits {
        match edit {
            RichTextEdit::Replace { path, value } => {
                if path.len() > 64 {
                    return Err(PatchError("path too deep"));
                }
                *node_at_mut(&mut out, path)? = value.clone();
            }
            RichTextEdit::Props { path, value } => {
                let node = node_at_mut(&mut out, path)?;
                let props = value
                    .as_object()
                    .ok_or(PatchError("props must be an object"))?;
                if props.contains_key("content") {
                    return Err(PatchError("props must not carry content"));
                }
                let content = node.get("content").cloned();
                let mut next = serde_json::Map::new();
                for (key, prop) in props {
                    next.insert(key.clone(), prop.clone());
                }
                if let Some(content) = content {
                    next.insert("content".into(), content);
                }
                *node = Value::Object(next);
            }
            RichTextEdit::Text { path, ops } => {
                let node = node_at_mut(&mut out, path)?;
                if !is_inline_container(node) {
                    return Err(PatchError("text edit on a non-inline node"));
                }
                let mut local = Styles(Vec::new());
                let base = scalars(node, &mut local);
                // Interned base styles are appended after the patch's own table.
                let mut table = patch.styles.clone();
                let base_offset = table.len();
                table.extend(local.0);
                let mut result: Vec<Styled> = Vec::with_capacity(base.len());
                let mut cursor = 0usize;
                for op in ops {
                    match op {
                        TextOp::Retain { count, style } => {
                            let end = cursor
                                .checked_add(*count)
                                .ok_or(PatchError("retain overflow"))?;
                            if end > base.len() {
                                return Err(PatchError("retain past end"));
                            }
                            for (ch, base_style) in &base[cursor..end] {
                                let style = match style {
                                    Some(style) => *style,
                                    None => base_style + base_offset,
                                };
                                result.push((*ch, style));
                            }
                            cursor = end;
                        }
                        TextOp::Delete { count } => {
                            let end = cursor
                                .checked_add(*count)
                                .ok_or(PatchError("delete overflow"))?;
                            if end > base.len() {
                                return Err(PatchError("delete past end"));
                            }
                            cursor = end;
                        }
                        TextOp::Insert { text, style } => {
                            if text.len() > MAX_SCALARS * 4 {
                                return Err(PatchError("insert too large"));
                            }
                            result.extend(text.chars().map(|ch| (ch, *style)));
                        }
                    }
                }
                if cursor != base.len() {
                    return Err(PatchError("incomplete consumption"));
                }
                if result.len() > MAX_SCALARS {
                    return Err(PatchError("too many scalars"));
                }
                let rebuilt = children(&result, &table)?;
                if let Some(object) = node.as_object_mut() {
                    if rebuilt.is_empty() {
                        object.remove("content");
                    } else {
                        object.insert("content".into(), Value::Array(rebuilt));
                    }
                }
            }
        }
    }
    if fingerprint(&out) != patch.after {
        return Err(PatchError("result fingerprint mismatch"));
    }
    Ok(out)
}

// ───────────────────────── position mapping ─────────────────────────

/// Which way an anchor boundary leans when text is inserted exactly at it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Bias {
    /// Stay before inserted text (anchor starts).
    Left,
    /// Move after inserted text (anchor ends), so text typed inside a span stays inside it.
    Right,
}

/// Map a scalar position through text ops. Returns `None` when the position
/// was inside deleted text.
#[must_use]
pub fn map_position(ops: &[TextOp], position: u32, bias: Bias) -> Option<u32> {
    let mut old = 0u32;
    let mut new = 0u32;
    for op in ops {
        match op {
            TextOp::Retain { count, .. } => {
                let count = *count as u32;
                if position < old + count || (position == old + count && bias == Bias::Left) {
                    return Some(new + (position - old));
                }
                old += count;
                new += count;
            }
            TextOp::Delete { count } => {
                let count = *count as u32;
                if position == old {
                    // The position sits just before the deleted text.
                    return Some(new);
                }
                if position < old + count {
                    return None;
                }
                old += count;
                if position == old && bias == Bias::Left {
                    return Some(new);
                }
            }
            TextOp::Insert { text, .. } => {
                let count = text.chars().count() as u32;
                if position == old && bias == Bias::Left {
                    return Some(new);
                }
                new += count;
            }
        }
    }
    Some(new + position.saturating_sub(old))
}

/// The text ops of a patch for the block root, if the patch is a root text edit.
#[must_use]
pub fn root_text_ops(patch: &RichTextPatch) -> Option<&[TextOp]> {
    patch.edits.iter().find_map(|edit| match edit {
        RichTextEdit::Text { path, ops } if path.is_empty() => Some(ops.as_slice()),
        _ => None,
    })
}

/// Whether the patch replaces the whole block (anchors must be re-attached).
#[must_use]
pub fn replaces_root(patch: &RichTextPatch) -> bool {
    patch
        .edits
        .iter()
        .any(|edit| matches!(edit, RichTextEdit::Replace { path, .. } if path.is_empty()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn paragraph(text: &str) -> Value {
        json!({"type":"paragraph","attrs":{"id":"p"},"content":[{"type":"text","text":text}]})
    }

    #[test]
    fn patches_round_trip_in_scalars() {
        let before = paragraph("Mira pushed her auburn hair back.");
        let after = paragraph("Mira pushed her black hair back, slowly.");
        let patch = make_patch(&before, &after).expect("patch");
        assert_eq!(patch.version, 1);
        assert_eq!(apply_patch(&before, &patch).unwrap(), after);
        let ops = root_text_ops(&patch).unwrap();
        assert!(matches!(ops[0], TextOp::Retain { count: 16, .. }));
    }

    #[test]
    fn counts_unicode_scalars_not_bytes() {
        let before = paragraph("মিরা তার চুল");
        let after = paragraph("মিরা তার কালো চুল");
        let patch = make_patch(&before, &after).unwrap();
        let ops = root_text_ops(&patch).unwrap();
        let total: usize = ops
            .iter()
            .map(|op| match op {
                TextOp::Retain { count, .. } | TextOp::Delete { count } => *count,
                TextOp::Insert { .. } => 0,
            })
            .sum();
        assert_eq!(total, "মিরা তার চুল".chars().count());
        assert_eq!(apply_patch(&before, &patch).unwrap(), after);
    }

    #[test]
    fn preserves_marks_and_props() {
        let before = json!({"type":"paragraph","attrs":{"id":"p"},"content":[
            {"type":"text","text":"Mira "},
            {"type":"text","text":"waited","marks":[{"type":"italic"}]},
            {"type":"text","text":"."}
        ]});
        let after = json!({"type":"paragraph","attrs":{"id":"p","align":"right"},"content":[
            {"type":"text","text":"Mira "},
            {"type":"text","text":"waited long","marks":[{"type":"italic"}]},
            {"type":"text","text":"."}
        ]});
        let patch = make_patch(&before, &after).unwrap();
        assert!(
            patch
                .edits
                .iter()
                .any(|edit| matches!(edit, RichTextEdit::Props { .. }))
        );
        assert_eq!(apply_patch(&before, &patch).unwrap(), after);
    }

    #[test]
    fn rejects_wrong_base() {
        let before = paragraph("a");
        let after = paragraph("ab");
        let patch = make_patch(&before, &after).unwrap();
        assert_eq!(
            apply_patch(&paragraph("x"), &patch),
            Err(PatchError("base fingerprint mismatch"))
        );
    }

    #[test]
    fn maps_positions_through_edits() {
        // "Mira pushed her auburn hair" -> "Mira pushed her black hair"
        let before = paragraph("Mira pushed her auburn hair");
        let after = paragraph("Mira pushed her black hair");
        let patch = make_patch(&before, &after).unwrap();
        let ops = root_text_ops(&patch).unwrap();
        // "hair" starts at 23 before and 22 after.
        assert_eq!(map_position(ops, 23, Bias::Left), Some(22));
        // Inside the replaced word: deleted.
        assert_eq!(map_position(ops, 17, Bias::Left), None);
        // Start of the sentence is unaffected.
        assert_eq!(map_position(ops, 0, Bias::Left), Some(0));
    }

    #[test]
    fn insertion_at_boundary_respects_bias() {
        let ops = vec![
            TextOp::Retain {
                count: 4,
                style: None,
            },
            TextOp::Insert {
                text: "xx".into(),
                style: 0,
            },
            TextOp::Retain {
                count: 4,
                style: None,
            },
        ];
        assert_eq!(map_position(&ops, 4, Bias::Left), Some(4));
        assert_eq!(map_position(&ops, 4, Bias::Right), Some(6));
    }

    #[test]
    fn structural_changes_fall_back_to_replace() {
        let before = json!({"type":"bulletList","attrs":{"id":"l"},"content":[{"type":"listItem","content":[paragraph("one")]}]});
        let after = json!({"type":"bulletList","attrs":{"id":"l"},"content":[{"type":"listItem","content":[paragraph("one")]},{"type":"listItem","content":[paragraph("two")]}]});
        let patch = make_patch(&before, &after).unwrap();
        assert!(replaces_root(&patch));
        assert_eq!(apply_patch(&before, &patch).unwrap(), after);
    }

    #[test]
    fn myers_handles_empty_and_identical() {
        let before = paragraph("");
        let after = paragraph("new text");
        let patch = make_patch(&before, &after).unwrap();
        assert_eq!(apply_patch(&before, &patch).unwrap(), after);
        assert!(make_patch(&after, &after).is_none());
    }
}
