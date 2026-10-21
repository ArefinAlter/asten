//! Property tests for rich-text patches: round trips, determinism, the
//! edit-distance cap, and position mapping for anchors.
//!
//! Paragraphs are generated in canonical form (no empty runs, adjacent runs
//! with equal marks merged, no `content` key when empty), which is the form
//! ProseMirror emits and the only form a patch can reproduce exactly: the
//! applier rebuilds text nodes by coalescing equal styles. For arbitrary
//! (non-canonical) input the weaker guarantee holds: `make_patch` returns
//! either an exact patch or `None`, never a wrong one.

use asten_blocks::anchor::{Mapped, map_span};
use asten_blocks::patch::{Bias, RichTextEdit, TextOp, apply_patch, make_patch, map_position};
use proptest::prelude::*;
use serde_json::{Value, json};

const CASES: u32 = 256;

// ───────────────────────── generators ─────────────────────────

/// Scalars from several planes and categories: ASCII, Bengali (with combining
/// vowel signs), combining diacriticals, ZWJ, emoji and Linear B (non-BMP).
fn scalar() -> impl Strategy<Value = char> {
    prop_oneof![
        5 => prop::char::range('a', 'e'),
        2 => prop::char::range(' ', '~'),
        1 => prop::char::range('\u{0980}', '\u{09FF}'),
        1 => prop::char::range('\u{0300}', '\u{036F}'),
        1 => Just('\u{200D}'),
        1 => prop::char::range('\u{1F600}', '\u{1F64F}'),
        1 => prop::char::range('\u{10000}', '\u{1003F}'),
    ]
}

/// A style index: 0 plain, 1 bold, 2 italic, 3 bold+italic.
fn style() -> impl Strategy<Value = u8> {
    0..4u8
}

fn marks(style: u8) -> Vec<Value> {
    let mut marks = Vec::new();
    if style & 1 != 0 {
        marks.push(json!({"type": "bold"}));
    }
    if style & 2 != 0 {
        marks.push(json!({"type": "italic"}));
    }
    marks
}

/// Styled scalars, the flat form every generator works in.
type Runs = Vec<(char, u8)>;

fn runs() -> impl Strategy<Value = Runs> {
    prop::collection::vec((prop::collection::vec(scalar(), 0..8), style()), 0..6).prop_map(
        |pieces| {
            pieces
                .into_iter()
                .flat_map(|(text, style)| text.into_iter().map(move |ch| (ch, style)))
                .collect()
        },
    )
}

/// Build a canonical paragraph (or heading) from styled scalars.
fn paragraph(node_type: &str, align: Option<&str>, runs: &Runs) -> Value {
    let mut content: Vec<Value> = Vec::new();
    let mut current: Option<(String, u8)> = None;
    for (ch, style) in runs {
        match &mut current {
            Some((text, existing)) if existing == style => text.push(*ch),
            _ => {
                if let Some((text, style)) = current.take() {
                    content.push(text_node(&text, style));
                }
                current = Some((ch.to_string(), *style));
            }
        }
    }
    if let Some((text, style)) = current.take() {
        content.push(text_node(&text, style));
    }
    let mut attrs = json!({"id": "p"});
    if let Some(align) = align {
        attrs["align"] = Value::String(align.to_owned());
    }
    let mut node = json!({"type": node_type, "attrs": attrs});
    if !content.is_empty() {
        node["content"] = Value::Array(content);
    }
    node
}

fn text_node(text: &str, style: u8) -> Value {
    let marks = marks(style);
    if marks.is_empty() {
        json!({"type": "text", "text": text})
    } else {
        json!({"type": "text", "text": text, "marks": marks})
    }
}

#[derive(Clone, Debug)]
enum Edit {
    Insert { at: usize, text: Vec<(char, u8)> },
    Delete { at: usize, len: usize },
    Restyle { at: usize, len: usize, style: u8 },
}

fn edit() -> impl Strategy<Value = Edit> {
    prop_oneof![
        (0..64usize, prop::collection::vec((scalar(), style()), 1..6))
            .prop_map(|(at, text)| Edit::Insert { at, text }),
        (0..64usize, 1..8usize).prop_map(|(at, len)| Edit::Delete { at, len }),
        (0..64usize, 1..8usize, style()).prop_map(|(at, len, style)| Edit::Restyle {
            at,
            len,
            style
        }),
    ]
}

fn apply_edits(mut runs: Runs, edits: &[Edit]) -> Runs {
    for edit in edits {
        match edit {
            Edit::Insert { at, text } => {
                let at = (*at).min(runs.len());
                runs.splice(at..at, text.iter().copied());
            }
            Edit::Delete { at, len } => {
                let at = (*at).min(runs.len());
                let end = (at + len).min(runs.len());
                runs.drain(at..end);
            }
            Edit::Restyle { at, len, style } => {
                let at = (*at).min(runs.len());
                let end = (at + len).min(runs.len());
                for scalar in &mut runs[at..end] {
                    scalar.1 = *style;
                }
            }
        }
    }
    runs
}

fn align() -> impl Strategy<Value = Option<&'static str>> {
    prop_oneof![
        3 => Just(None),
        1 => Just(Some("right")),
        1 => Just(Some("center")),
    ]
}

fn node_type() -> impl Strategy<Value = &'static str> {
    prop_oneof![4 => Just("paragraph"), 1 => Just("heading")]
}

/// A before/after pair where `after` is `before` under a few random edits,
/// so the diff is exercised on related inputs as well as unrelated ones.
fn related_pair() -> impl Strategy<Value = (Value, Value, Runs, Runs)> {
    (
        runs(),
        prop::collection::vec(edit(), 0..5),
        node_type(),
        align(),
        align(),
    )
        .prop_map(|(before, edits, node_type, align_a, align_b)| {
            let after = apply_edits(before.clone(), &edits);
            (
                paragraph(node_type, align_a, &before),
                paragraph(node_type, align_b, &after),
                before,
                after,
            )
        })
}

fn unrelated_pair() -> impl Strategy<Value = (Value, Value, Runs, Runs)> {
    (runs(), runs(), node_type(), node_type()).prop_map(|(before, after, type_a, type_b)| {
        (
            paragraph(type_a, None, &before),
            paragraph(type_b, None, &after),
            before,
            after,
        )
    })
}

/// A paragraph that may be non-canonical: empty runs, adjacent equal styles,
/// an empty `content` array.
fn loose_paragraph() -> impl Strategy<Value = Value> {
    prop::collection::vec((prop::collection::vec(scalar(), 0..5), style()), 0..5).prop_map(
        |pieces| {
            let content: Vec<Value> = pieces
                .iter()
                .map(|(text, style)| text_node(&text.iter().collect::<String>(), *style))
                .collect();
            json!({"type": "paragraph", "attrs": {"id": "p"}, "content": content})
        },
    )
}

// ───────────────────────── helpers ─────────────────────────

fn root_text_ops(patch: &asten_blocks::patch::RichTextPatch) -> Option<&[TextOp]> {
    asten_blocks::patch::root_text_ops(patch)
}

/// Where every original scalar lands after the ops: `Some(new index)` when
/// retained, `None` when deleted. Computed independently of `map_position`.
fn survivors(ops: &[TextOp], old_len: usize) -> Vec<Option<usize>> {
    let mut out = Vec::with_capacity(old_len);
    let mut new = 0usize;
    for op in ops {
        match op {
            TextOp::Retain { count, .. } => {
                for _ in 0..*count {
                    out.push(Some(new));
                    new += 1;
                }
            }
            TextOp::Delete { count } => out.extend(std::iter::repeat_n(None, *count)),
            TextOp::Insert { text, .. } => new += text.chars().count(),
        }
    }
    assert_eq!(out.len(), old_len, "ops must consume the whole base");
    out
}

fn new_len(ops: &[TextOp]) -> usize {
    ops.iter()
        .map(|op| match op {
            TextOp::Retain { count, .. } => *count,
            TextOp::Delete { .. } => 0,
            TextOp::Insert { text, .. } => text.chars().count(),
        })
        .sum()
}

// ───────────────────────── properties ─────────────────────────

proptest! {
    #![proptest_config(ProptestConfig { cases: CASES, .. ProptestConfig::default() })]

    /// `apply_patch(before, make_patch(before, after)) == after` for canonical
    /// paragraphs, and a patch always exists when the two differ.
    #[test]
    fn related_edits_round_trip((before, after, _, _) in related_pair()) {
        if before == after {
            prop_assert!(make_patch(&before, &after).is_none());
            return Ok(());
        }
        let patch = make_patch(&before, &after);
        prop_assert!(patch.is_some(), "no patch for\n{before}\n->\n{after}");
        let patch = patch.unwrap();
        prop_assert_eq!(apply_patch(&before, &patch), Ok(after.clone()));
    }

    #[test]
    fn unrelated_edits_round_trip((before, after, _, _) in unrelated_pair()) {
        if before == after {
            return Ok(());
        }
        let patch = make_patch(&before, &after);
        prop_assert!(patch.is_some(), "no patch for\n{before}\n->\n{after}");
        prop_assert_eq!(apply_patch(&before, &patch.unwrap()), Ok(after.clone()));
    }

    /// Non-canonical input never yields a wrong patch: the result is exact or `None`.
    #[test]
    fn loose_input_is_exact_or_refused(before in loose_paragraph(), after in loose_paragraph()) {
        if let Some(patch) = make_patch(&before, &after) {
            prop_assert_eq!(apply_patch(&before, &patch), Ok(after.clone()));
        }
    }

    /// The same inputs always produce the same ops and style table.
    #[test]
    fn patches_are_deterministic((before, after, _, _) in related_pair()) {
        let first = make_patch(&before, &after);
        let second = make_patch(&before, &after);
        prop_assert_eq!(first, second);
    }

    /// Text ops exactly consume the base and produce the new length, and every
    /// retained scalar keeps its character.
    #[test]
    fn ops_are_well_formed((before, after, old, new) in related_pair()) {
        let Some(patch) = make_patch(&before, &after) else { return Ok(()) };
        let Some(ops) = root_text_ops(&patch) else { return Ok(()) };
        let map = survivors(ops, old.len());
        prop_assert_eq!(new_len(ops), new.len());
        for (index, slot) in map.iter().enumerate() {
            if let Some(target) = slot {
                prop_assert_eq!(old[index].0, new[*target].0, "retained scalar changed");
            }
        }
    }

    /// `map_position` is monotone non-decreasing for both biases, never maps
    /// past the new length, and the left bias never lands right of the right one.
    #[test]
    fn map_position_is_monotone_and_bounded((before, after, old, _) in related_pair()) {
        let Some(patch) = make_patch(&before, &after) else { return Ok(()) };
        let Some(ops) = root_text_ops(&patch) else { return Ok(()) };
        let limit = new_len(ops) as u32;
        for bias in [Bias::Left, Bias::Right] {
            let mut last = 0u32;
            for position in 0..=old.len() as u32 {
                if let Some(mapped) = map_position(ops, position, bias) {
                    prop_assert!(mapped >= last, "{bias:?}: {position} -> {mapped} after {last}");
                    prop_assert!(mapped <= limit, "{bias:?}: {position} -> {mapped} > {limit}");
                    last = mapped;
                }
            }
        }
        for position in 0..=old.len() as u32 {
            if let (Some(left), Some(right)) = (
                map_position(ops, position, Bias::Left),
                map_position(ops, position, Bias::Right),
            ) {
                prop_assert!(left <= right, "{position}: left {left} > right {right}");
            }
        }
        // A retained scalar's boundary maps, with the right bias, exactly to
        // where the scalar went; with the left bias it stays before any text
        // inserted at that boundary, so it lands at or before the scalar.
        for (index, slot) in survivors(ops, old.len()).iter().enumerate() {
            if let Some(target) = slot {
                let target = *target as u32;
                prop_assert_eq!(map_position(ops, index as u32, Bias::Right), Some(target));
                let left = map_position(ops, index as u32, Bias::Left);
                prop_assert!(left.is_some_and(|left| left <= target), "{index}: {left:?} vs {target}");
            }
        }
    }

    /// `map_span` is `Orphaned` exactly when no scalar of the span survived, and
    /// a moved span still covers every surviving scalar.
    #[test]
    fn map_span_orphans_only_dead_spans(
        (before, after, old, _) in related_pair(),
        raw_start in 0..64u32,
        raw_len in 1..16u32,
    ) {
        let Some(patch) = make_patch(&before, &after) else { return Ok(()) };
        let Some(ops) = root_text_ops(&patch) else { return Ok(()) };
        if old.is_empty() {
            return Ok(());
        }
        let start = raw_start % old.len() as u32;
        let end = (start + raw_len).min(old.len() as u32);
        let map = survivors(ops, old.len());
        let alive: Vec<usize> = (start..end)
            .filter_map(|p| map[p as usize])
            .collect();
        match map_span(ops, start, end) {
            Mapped::Orphaned => {
                prop_assert!(alive.is_empty(), "orphaned [{start}, {end}) although {alive:?} survived");
            }
            Mapped::Moved { start: s, end: e } => {
                prop_assert!(!alive.is_empty(), "moved although nothing survived");
                prop_assert!(s < e, "empty moved span {s}..{e}");
                prop_assert!(e as usize <= new_len(ops));
                let first = *alive.first().unwrap() as u32;
                let last = *alive.last().unwrap() as u32;
                prop_assert!(s <= first && last < e, "[{s}, {e}) does not cover {first}..={last}");
            }
        }
    }
}

/// Two long, unrelated texts exceed `MAX_EDIT_DISTANCE`; the patch degrades to a
/// rewrite (delete all, insert all) that still round-trips.
#[test]
fn edit_distance_cap_falls_back_to_a_rewrite() {
    let before_runs: Runs = (0..800u32)
        .map(|i| (char::from_u32(0x4E00 + i).unwrap(), 0))
        .collect();
    let mut after_runs = before_runs.clone();
    after_runs.reverse();
    let before = paragraph("paragraph", None, &before_runs);
    let after = paragraph("paragraph", None, &after_runs);
    let patch = make_patch(&before, &after).expect("a rewrite patch");
    let ops = root_text_ops(&patch).expect("root text ops");
    assert!(
        matches!(ops, [TextOp::Delete { count: 800 }, TextOp::Insert { text, .. }] if text.chars().count() == 800),
        "expected a whole rewrite, got {} ops",
        ops.len()
    );
    assert_eq!(apply_patch(&before, &patch), Ok(after));
}

proptest! {
    // Each case diffs ~1k scalars up to the cap; keep the count small.
    #![proptest_config(ProptestConfig { cases: 8, .. ProptestConfig::default() })]

    /// Long random texts round-trip whether or not the minimal diff is found.
    #[test]
    fn long_texts_round_trip(
        before in prop::collection::vec(prop::char::range('\u{4E00}', '\u{9FFF}'), 600..1000),
        after in prop::collection::vec(prop::char::range('\u{4E00}', '\u{9FFF}'), 600..1000),
    ) {
        let before: Runs = before.into_iter().map(|ch| (ch, 0)).collect();
        let after: Runs = after.into_iter().map(|ch| (ch, 0)).collect();
        let a = paragraph("paragraph", None, &before);
        let b = paragraph("paragraph", None, &after);
        let patch = make_patch(&a, &b).expect("patch");
        prop_assert_eq!(apply_patch(&a, &patch), Ok(b));
    }
}

/// Structural changes that are not inline edits still round-trip via `Replace`.
#[test]
fn type_changes_replace_the_root() {
    let runs: Runs = "hello".chars().map(|ch| (ch, 0)).collect();
    let before = paragraph("paragraph", None, &runs);
    let after = paragraph("heading", None, &runs);
    let patch = make_patch(&before, &after).unwrap();
    assert!(
        patch
            .edits
            .iter()
            .any(|edit| matches!(edit, RichTextEdit::Replace { path, .. } if path.is_empty()))
    );
    assert_eq!(apply_patch(&before, &patch), Ok(after));
}
