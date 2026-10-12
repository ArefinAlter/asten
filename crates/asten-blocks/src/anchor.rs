//! Anchors survive editing by position mapping through patches and, when that
//! fails, by re-attaching the anchored quote in the block's new text.

use crate::patch::{Bias, TextOp, map_position};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Mapped {
    /// The span moved (or shrank) and is still valid.
    Moved { start: u32, end: u32 },
    /// The span was deleted or collapsed; re-attach it from its quote.
    Orphaned,
}

/// Map a `[start, end)` span through text ops. The start stays left of text
/// inserted at it; the end moves right, so typing inside a span keeps it inside.
#[must_use]
pub fn map_span(ops: &[TextOp], start: u32, end: u32) -> Mapped {
    if retained_in(ops, start, end) == 0 {
        // Nothing of the original text survives: the span must be re-attached.
        return Mapped::Orphaned;
    }
    let new_start = map_position(ops, start, Bias::Left);
    let new_end = map_position(ops, end, Bias::Right);
    match (new_start, new_end) {
        (Some(s), Some(e)) if e > s => Mapped::Moved { start: s, end: e },
        (Some(s), None) => {
            // The end fell inside a deletion: shrink to the deletion's start.
            let shrunk = map_position(ops, end, Bias::Left).or_else(|| {
                // Walk back to the last surviving position.
                (start..end)
                    .rev()
                    .find_map(|p| map_position(ops, p, Bias::Right))
            });
            match shrunk {
                Some(e) if e > s => Mapped::Moved { start: s, end: e },
                _ => Mapped::Orphaned,
            }
        }
        (None, Some(e)) => {
            let grown = (start..end).find_map(|p| map_position(ops, p, Bias::Left));
            match grown {
                Some(s) if e > s => Mapped::Moved { start: s, end: e },
                _ => Mapped::Orphaned,
            }
        }
        _ => Mapped::Orphaned,
    }
}

/// How many scalars of the original `[start, end)` the ops keep.
fn retained_in(ops: &[TextOp], start: u32, end: u32) -> u32 {
    let mut old = 0u32;
    let mut kept = 0u32;
    for op in ops {
        match op {
            TextOp::Retain { count, .. } => {
                let count = *count as u32;
                let lo = old.max(start);
                let hi = (old + count).min(end);
                if hi > lo {
                    kept += hi - lo;
                }
                old += count;
            }
            TextOp::Delete { count } => old += *count as u32,
            TextOp::Insert { .. } => {}
        }
    }
    // Anything past the last op is retained implicitly.
    if end > old.max(start) {
        kept += end - old.max(start);
    }
    kept
}

/// Find `quote` in `text`, preferring the occurrence whose surroundings agree
/// best with `before` and `after`. Falls back to a fuzzy match when the quote
/// itself changed slightly. Offsets are in scalars. Returns `None` when nothing
/// close enough exists.
#[must_use]
pub fn reattach(quote: &str, before: &str, after: &str, text: &str) -> Option<(u32, u32)> {
    let quote_chars: Vec<char> = quote.chars().collect();
    if quote_chars.is_empty() {
        return None;
    }
    let text_chars: Vec<char> = text.chars().collect();
    let before_chars: Vec<char> = before.chars().collect();
    let after_chars: Vec<char> = after.chars().collect();

    // Exact occurrences, scored by context agreement.
    let mut best: Option<(usize, usize, f64)> = None;
    if text_chars.len() >= quote_chars.len() {
        for start in 0..=text_chars.len() - quote_chars.len() {
            if text_chars[start..start + quote_chars.len()] != quote_chars[..] {
                continue;
            }
            let end = start + quote_chars.len();
            let score = context_agreement(&text_chars, start, end, &before_chars, &after_chars);
            if best.is_none_or(|(_, _, existing)| score > existing) {
                best = Some((start, end, score));
            }
        }
    }
    if let Some((start, end, _)) = best {
        return Some((start as u32, end as u32));
    }

    // Fuzzy: windows within ±25% of the quote length, normalised edit distance ≤ 0.25.
    // Only the head of a long quote is matched (the cost is cubic in the quote
    // length), and very long blocks are not searched at all.
    const FUZZY_HEAD: usize = 64;
    const FUZZY_MAX_TEXT: usize = 20_000;
    if text_chars.is_empty() || text_chars.len() > FUZZY_MAX_TEXT {
        return None;
    }
    let head: Vec<char> = quote_chars.iter().take(FUZZY_HEAD).copied().collect();
    let tail_len = quote_chars.len() - head.len();
    let min_len = (head.len() * 3 / 4).max(1);
    let max_len = head.len() * 5 / 4 + 1;
    let mut fuzzy: Option<(usize, usize, f64)> = None;
    for start in 0..text_chars.len() {
        for len in min_len..=max_len {
            let end = start + len;
            if end > text_chars.len() {
                break;
            }
            let distance = levenshtein(&text_chars[start..end], &head, head.len() / 4 + 1);
            let Some(distance) = distance else { continue };
            let normalised = distance as f64 / head.len().max(1) as f64;
            if normalised > 0.25 {
                continue;
            }
            let end = (end + tail_len).min(text_chars.len());
            let score = (1.0 - normalised)
                + 0.5 * context_agreement(&text_chars, start, end, &before_chars, &after_chars);
            if fuzzy.is_none_or(|(_, _, existing)| score > existing) {
                fuzzy = Some((start, end, score));
            }
        }
    }
    fuzzy.map(|(start, end, _)| (start as u32, end as u32))
}

/// How much of the remembered context matches around a candidate span, in `[0, 1]`.
fn context_agreement(
    text: &[char],
    start: usize,
    end: usize,
    before: &[char],
    after: &[char],
) -> f64 {
    let mut matched = 0usize;
    let mut total = 0usize;
    for (offset, expected) in before.iter().rev().enumerate() {
        total += 1;
        if start > offset && text[start - 1 - offset] == *expected {
            matched += 1;
        }
    }
    for (offset, expected) in after.iter().enumerate() {
        total += 1;
        if end + offset < text.len() && text[end + offset] == *expected {
            matched += 1;
        }
    }
    if total == 0 {
        0.5
    } else {
        matched as f64 / total as f64
    }
}

/// Levenshtein distance with an early exit when it exceeds `limit`.
fn levenshtein(a: &[char], b: &[char], limit: usize) -> Option<usize> {
    if a.len().abs_diff(b.len()) > limit {
        return None;
    }
    let mut previous: Vec<usize> = (0..=b.len()).collect();
    let mut current = vec![0; b.len() + 1];
    for (i, ca) in a.iter().enumerate() {
        current[0] = i + 1;
        let mut row_min = current[0];
        for (j, cb) in b.iter().enumerate() {
            let cost = usize::from(ca != cb);
            current[j + 1] = (previous[j] + cost)
                .min(previous[j + 1] + 1)
                .min(current[j] + 1);
            row_min = row_min.min(current[j + 1]);
        }
        if row_min > limit {
            return None;
        }
        std::mem::swap(&mut previous, &mut current);
    }
    let distance = previous[b.len()];
    (distance <= limit).then_some(distance)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_spans_and_shrinks_at_deletions() {
        // retain 5, delete 3, insert "xy", retain 5
        let ops = vec![
            TextOp::Retain {
                count: 5,
                style: None,
            },
            TextOp::Delete { count: 3 },
            TextOp::Insert {
                text: "xy".into(),
                style: 0,
            },
            TextOp::Retain {
                count: 5,
                style: None,
            },
        ];
        assert_eq!(map_span(&ops, 0, 4), Mapped::Moved { start: 0, end: 4 });
        assert_eq!(map_span(&ops, 9, 13), Mapped::Moved { start: 8, end: 12 });
        assert_eq!(map_span(&ops, 5, 8), Mapped::Orphaned);
        assert_eq!(map_span(&ops, 3, 7), Mapped::Moved { start: 3, end: 5 });
    }

    #[test]
    fn reattaches_exact_then_fuzzy() {
        let text = "Later, Mira pushed her black hair out of her eyes and smiled.";
        assert_eq!(
            reattach("black hair", "pushed her ", " out", text),
            Some((23, 33))
        );
        assert_eq!(
            reattach("blak hair", "pushed her ", " out", text),
            Some((23, 33))
        );
        assert_eq!(reattach("auburn braid", "", "", text), None);
    }

    #[test]
    fn context_disambiguates_repeats() {
        let text = "the ring, the ring again";
        assert_eq!(reattach("ring", "the ", " again", text), Some((14, 18)));
    }
}
