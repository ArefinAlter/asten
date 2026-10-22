//! Property tests for the deterministic frontend: folded offsets map back to
//! the original scalars, the cast matcher finds whole-word names (in any case
//! or normalisation form) and nothing inside longer words, and the segmenters
//! agree with their naive forms.

use asten_ir::{Context, Entity, EntityId, Sort};
use asten_perceive::{Folded, Matcher, Segments, matching_form, quoted_spans, segment};
use proptest::prelude::*;
use unicode_normalization::UnicodeNormalization;
use unicode_segmentation::UnicodeSegmentation;

const CASES: u32 = 256;

// ───────────────────────── folding ─────────────────────────

/// Case-fold one scalar: lowercase, with Greek final sigma folded to sigma as
/// Unicode case folding does.
fn fold_char(ch: char) -> impl Iterator<Item = char> {
    ch.to_lowercase().map(|folded| {
        if folded == '\u{03C2}' {
            '\u{03C3}'
        } else {
            folded
        }
    })
}

/// The fold of a whole string: NFKC, then per-char case folding. Patterns are
/// folded this way, so the folded text must agree with it.
fn fold_str(text: &str) -> String {
    text.nfkc().flat_map(fold_char).collect()
}

/// Scalars that fold one-to-one and ones that expand or compose: ligatures,
/// circled digits, halfwidth kana, the Ohm sign, Turkish capital I, capital
/// sharp S, a long Arabic ligature, precomposed Latin letters, Hangul jamo,
/// plus combining marks, Bengali, emoji and ZWJ.
fn weird_char() -> impl Strategy<Value = char> {
    prop_oneof![
        5 => prop::char::range('a', 'z'),
        1 => prop::char::range('A', 'Z'),
        2 => Just(' '),
        2 => prop::sample::select(vec![
            '\u{FB01}', '\u{FB03}', '\u{2460}', '\u{FF76}', '\u{339E}', '\u{0130}', '\u{1E9E}',
            '\u{03A3}', '\u{00DF}', '\u{2126}', '\u{00B5}', '\u{FDFA}', '\u{2167}', '\u{00E9}',
            '\u{00C5}', '\u{212B}', '\u{09DF}', '\u{1100}', '\u{1161}', '\u{11A8}',
        ]),
        1 => prop::char::range('\u{0300}', '\u{036F}'),
        1 => prop::char::range('\u{0980}', '\u{09FF}'),
        1 => prop::char::range('\u{1F600}', '\u{1F64F}'),
        1 => Just('\u{200D}'),
    ]
}

fn weird_text() -> impl Strategy<Value = String> {
    prop::collection::vec(weird_char(), 0..40).prop_map(|chars| chars.into_iter().collect())
}

/// The specification of `Folded`: each grapheme cluster folds on its own and
/// every folded char remembers the cluster's scalar range.
struct Spec {
    chars: Vec<char>,
    origin: Vec<usize>,
    end: Vec<usize>,
}

fn spec(text: &str) -> Spec {
    let mut spec = Spec {
        chars: Vec::new(),
        origin: Vec::new(),
        end: Vec::new(),
    };
    let mut scalar = 0usize;
    for cluster in text.graphemes(true) {
        let start = scalar;
        scalar += cluster.chars().count();
        for folded in cluster.nfkc().flat_map(fold_char) {
            spec.chars.push(folded);
            spec.origin.push(start);
            spec.end.push(scalar);
        }
    }
    spec
}

fn cluster_boundaries(text: &str) -> Vec<usize> {
    let mut boundaries = vec![0];
    let mut scalar = 0;
    for cluster in text.graphemes(true) {
        scalar += cluster.chars().count();
        boundaries.push(scalar);
    }
    boundaries
}

proptest! {
    #![proptest_config(ProptestConfig { cases: CASES, .. ProptestConfig::default() })]

    /// Folded chars are the per-cluster folds in order and equal the fold of
    /// the whole text (so patterns and text agree); `origin` is monotone and
    /// a valid original index; byte offsets map back to folded indices.
    #[test]
    fn folding_keeps_origins(text in weird_text()) {
        let folded = Folded::new(&text);
        let want = spec(&text);
        prop_assert_eq!(&folded.chars, &want.chars);
        prop_assert_eq!(&folded.origin, &want.origin);
        prop_assert_eq!(&folded.text, &fold_str(&text), "folded text differs from the whole-string fold");
        prop_assert!(folded.origin.windows(2).all(|pair| pair[0] <= pair[1]));
        let originals = text.chars().count();
        prop_assert!(folded.origin.iter().all(|origin| *origin < originals));
        prop_assert_eq!(matching_form(&text), fold_str(text.trim()));

        for (index, (byte, _)) in folded.text.char_indices().enumerate() {
            prop_assert_eq!(folded.char_index(byte), index);
        }
        prop_assert_eq!(folded.char_index(folded.text.len()), folded.chars.len());
    }

    /// The original span of a folded range is the range of the grapheme
    /// clusters it came from: its fold contains the matched text, never
    /// splits a cluster, and equals the matched text when the range is
    /// cluster-aligned.
    #[test]
    fn original_spans_cover_the_match(text in weird_text(), raw_start in 0..64usize, raw_len in 0..8usize) {
        let folded = Folded::new(&text);
        let n = folded.chars.len();
        if n == 0 {
            prop_assert_eq!(folded.original_span(0, 0), (0, 0));
            return Ok(());
        }
        let want = spec(&text);
        let originals: Vec<char> = text.chars().collect();
        let start = raw_start % n;
        let end = (start + 1 + raw_len).min(n);
        let (s, e) = folded.original_span(start, end);
        let (s, e) = (s as usize, e as usize);
        prop_assert_eq!(s, want.origin[start]);
        prop_assert_eq!(e, want.end[end - 1]);
        prop_assert!(s < e && e <= originals.len());
        let covered = fold_str(&originals[s..e].iter().collect::<String>());
        let matched: String = folded.chars[start..end].iter().collect();
        prop_assert!(covered.contains(&matched), "{covered:?} does not contain {matched:?}");
        let aligned_start = start == 0 || folded.origin[start - 1] != folded.origin[start];
        let aligned_end = end == n || folded.origin[end] != folded.origin[end - 1];
        if aligned_start && aligned_end {
            prop_assert_eq!(covered, matched);
        }
        let boundaries = cluster_boundaries(&text);
        prop_assert!(boundaries.contains(&s) && boundaries.contains(&e), "[{s}, {e}) splits a cluster");
    }
}

// ───────────────────────── the cast matcher ─────────────────────────

/// Letters whose case round-trips scalar for scalar: ASCII, Greek (minus
/// final sigma), Cyrillic, Bengali consonants, and precomposed Latin letters
/// that NFD takes apart.
fn name_char() -> impl Strategy<Value = char> {
    prop_oneof![
        4 => prop::char::range('a', 'z'),
        1 => prop::char::range('\u{03B1}', '\u{03C9}').prop_filter("final sigma", |ch| *ch != '\u{03C2}'),
        1 => prop::char::range('\u{0430}', '\u{044F}'),
        1 => prop::char::range('\u{0995}', '\u{09A8}'),
        1 => prop::sample::select(vec!['\u{00E9}', '\u{00F1}', '\u{00F6}', '\u{017E}', '\u{00E5}']),
    ]
}

fn word(len: std::ops::Range<usize>) -> impl Strategy<Value = String> {
    prop::collection::vec(name_char(), len).prop_map(|chars| chars.into_iter().collect())
}

#[derive(Clone, Debug)]
enum Token {
    Exact(usize),
    Suffixed(usize, String),
    Prefixed(String, usize),
    Noise(String),
}

#[derive(Clone, Copy, Debug)]
enum Case {
    AsIs,
    Capitalised,
    Upper,
}

#[derive(Clone, Copy, Debug)]
enum Norm {
    AsIs,
    Nfc,
    Nfd,
}

fn with_case(name: &str, case: Case) -> String {
    match case {
        Case::AsIs => name.to_owned(),
        Case::Upper => name.to_uppercase(),
        Case::Capitalised => {
            let mut chars = name.chars();
            let first: String = chars
                .next()
                .map(|ch| ch.to_uppercase().collect())
                .unwrap_or_default();
            first + chars.as_str()
        }
    }
}

fn with_norm(text: &str, norm: Norm) -> String {
    match norm {
        Norm::AsIs => text.to_owned(),
        Norm::Nfc => text.nfc().collect(),
        Norm::Nfd => text.nfd().collect(),
    }
}

const SEPARATORS: [&str; 6] = [" ", ", ", ". ", " \u{201c}", "\u{2014}", "\n"];

/// Every `(name index, folded position)` at which a cast name starts inside
/// `token`, compared in matching form.
fn occurrences(names: &[String], token: &str) -> Vec<(usize, usize)> {
    let chars: Vec<char> = matching_form(token).chars().collect();
    let mut found = Vec::new();
    for (index, name) in names.iter().enumerate() {
        let name: Vec<char> = matching_form(name).chars().collect();
        for position in 0..chars.len() {
            if chars[position..].starts_with(&name) {
                found.push((index, position));
            }
        }
    }
    found
}

type Laid = (Token, Case, Norm, &'static str);

fn cast_and_tokens() -> impl Strategy<Value = (Vec<String>, Vec<Norm>, Vec<Laid>)> {
    prop::collection::hash_set(word(2..6), 1..=3)
        .prop_map(|set| set.into_iter().collect::<Vec<String>>())
        .prop_filter("no name inside another", |names| {
            names.iter().enumerate().all(|(i, a)| {
                names
                    .iter()
                    .enumerate()
                    .all(|(j, b)| i == j || !matching_form(b).contains(&matching_form(a)))
            })
        })
        .prop_flat_map(|names| {
            let count = names.len();
            let norm = prop::sample::select(vec![Norm::AsIs, Norm::Nfc, Norm::Nfd]);
            let token = (
                0..4u8,
                0..count,
                word(1..7),
                prop::sample::select(vec![Case::AsIs, Case::Capitalised, Case::Upper]),
                norm.clone(),
                prop::sample::select(SEPARATORS.to_vec()),
            );
            (
                Just(names),
                prop::collection::vec(norm, count),
                prop::collection::vec(token, 0..8),
            )
        })
        .prop_map(|(names, stored, raw)| {
            let tokens = raw
                .into_iter()
                .filter_map(|(kind, index, extra, case, norm, separator)| {
                    let token = match kind {
                        0 => Token::Exact(index),
                        1 => Token::Suffixed(index, extra),
                        2 => {
                            let token = format!("{extra}{}", names[index]);
                            let only_at_end = occurrences(&names, &token)
                                == vec![(index, matching_form(&extra).chars().count())];
                            if !only_at_end {
                                return None;
                            }
                            Token::Prefixed(extra, index)
                        }
                        _ => {
                            if !occurrences(&names, &extra).is_empty() {
                                return None;
                            }
                            Token::Noise(extra)
                        }
                    };
                    Some((token, case, norm, separator))
                })
                .collect();
            (names, stored, tokens)
        })
}

struct Expected {
    entity: usize,
    start: u32,
    end: u32,
    confidence: f32,
}

/// Lay the tokens out as text and predict the mentions: a name on its own or
/// with up to four trailing letters is a mention spanning the name's scalars
/// as written; a name after letters, or with five or more trailing letters,
/// is part of another word.
fn lay_out(names: &[String], tokens: &[Laid]) -> (String, Vec<Expected>) {
    let mut text = String::new();
    let mut offset = 0u32;
    let mut expected = Vec::new();
    for (token, case, norm, separator) in tokens {
        text.push_str(separator);
        offset += separator.chars().count() as u32;
        let (name_part, rest, hit) = match token {
            Token::Exact(index) => (
                with_norm(&with_case(&names[*index], *case), *norm),
                String::new(),
                Some((*index, 0.97f32)),
            ),
            Token::Suffixed(index, suffix) => {
                let tolerated = suffix.chars().count() <= 4;
                let mut confidence: f32 = 0.97;
                confidence *= 0.75;
                (
                    with_norm(&with_case(&names[*index], *case), *norm),
                    with_norm(suffix, *norm),
                    tolerated.then_some((*index, confidence)),
                )
            }
            Token::Prefixed(prefix, index) => (
                with_norm(
                    &format!("{prefix}{}", with_case(&names[*index], *case)),
                    *norm,
                ),
                String::new(),
                None,
            ),
            Token::Noise(word) => (
                with_norm(&with_case(word, *case), *norm),
                String::new(),
                None,
            ),
        };
        if let Some((index, confidence)) = hit {
            expected.push(Expected {
                entity: index,
                start: offset,
                end: offset + name_part.chars().count() as u32,
                confidence,
            });
        }
        offset += (name_part.chars().count() + rest.chars().count()) as u32;
        text.push_str(&name_part);
        text.push_str(&rest);
    }
    (text, expected)
}

fn cast(names: &[String], stored: &[Norm]) -> Vec<Entity> {
    names
        .iter()
        .zip(stored)
        .enumerate()
        .map(|(index, (name, norm))| Entity {
            id: EntityId::new(format!("e{index}")),
            sort: Sort::Character,
            name: with_norm(name, *norm),
            aliases: Vec::new(),
            introduced_at: None,
        })
        .collect()
}

proptest! {
    #![proptest_config(ProptestConfig { cases: CASES, .. ProptestConfig::default() })]

    /// The matcher finds every whole-word occurrence (in any case and either
    /// normalisation form), tolerates up to four trailing letters at reduced
    /// confidence, never matches a name preceded by letters, and every anchor
    /// spans exactly the name as written.
    #[test]
    fn matcher_finds_whole_words_only((names, stored, tokens) in cast_and_tokens()) {
        let entities = cast(&names, &stored);
        let matcher = Matcher::new(&entities);
        let (text, expected) = lay_out(&names, &tokens);
        let folded = Folded::new(&text);
        let mentions = matcher.mentions(&folded, "doc", "block", 1, &Context::Narration, &[], "Latn");
        prop_assert_eq!(
            mentions.len(),
            expected.len(),
            "text {:?} names {:?}: got {:?}",
            text,
            names,
            mentions.iter().map(|m| (m.entity.0.clone(), m.anchor.start, m.anchor.end)).collect::<Vec<_>>()
        );
        let originals: Vec<char> = text.chars().collect();
        for (mention, want) in mentions.iter().zip(&expected) {
            prop_assert_eq!(&mention.entity, &EntityId::new(format!("e{}", want.entity)));
            prop_assert_eq!((mention.anchor.start, mention.anchor.end), (want.start, want.end), "{:?}", text);
            prop_assert_eq!(mention.confidence, want.confidence);
            prop_assert_eq!(&mention.surface, &matching_form(&names[want.entity]));
            prop_assert_eq!(&mention.context, &Context::Narration);
            // The anchored text folds to the name: never shorter than it.
            let anchored: String = originals[want.start as usize..want.end as usize].iter().collect();
            prop_assert_eq!(matching_form(&anchored), matching_form(&names[want.entity]));
        }
    }

    /// Matching is a function of the folded text: the same entities, with the
    /// same confidences, come from the text in any case or normalisation form.
    #[test]
    fn matching_ignores_case_and_form((names, stored, tokens) in cast_and_tokens()) {
        let entities = cast(&names, &stored);
        let matcher = Matcher::new(&entities);
        let (text, _) = lay_out(&names, &tokens);
        let entities_of = |text: &str| {
            matcher
                .mentions(&Folded::new(text), "d", "b", 1, &Context::Narration, &[], "Latn")
                .into_iter()
                .map(|m| (m.entity, m.confidence.to_bits()))
                .collect::<Vec<_>>()
        };
        let base = entities_of(&text);
        prop_assert_eq!(&base, &entities_of(&text.to_uppercase()));
        prop_assert_eq!(&base, &entities_of(&text.to_lowercase()));
        prop_assert_eq!(&base, &entities_of(&text.nfc().collect::<String>()));
        prop_assert_eq!(&base, &entities_of(&text.nfd().collect::<String>()));
    }
}

// ───────────────────────── segmentation oracles ─────────────────────────

/// The direct form of `segment`: scalar offsets recounted from the start for
/// every boundary. The library keeps one running cursor instead.
fn naive_segment(text: &str) -> Segments {
    let byte_to_scalar = |byte: usize| text[..byte.min(text.len())].chars().count() as u32;
    let mut sentences = Vec::new();
    let mut offset = 0usize;
    for sentence in text.unicode_sentences() {
        let start = text[offset..]
            .find(sentence)
            .map(|found| offset + found)
            .unwrap_or(offset);
        let end = start + sentence.len();
        sentences.push((byte_to_scalar(start), byte_to_scalar(end)));
        offset = end;
    }
    let words = text
        .unicode_word_indices()
        .map(|(byte, word)| (byte_to_scalar(byte), byte_to_scalar(byte + word.len())))
        .collect();
    Segments { sentences, words }
}

/// The direct form of `quoted_spans`, with `nth` lookups for the neighbours
/// of a closing single quote.
fn naive_quoted_spans(text: &str) -> Vec<(u32, u32)> {
    let mut spans = Vec::new();
    let mut open: Option<(char, u32)> = None;
    for (index, ch) in text.chars().enumerate() {
        let index = index as u32;
        match open {
            None => {
                let closer = match ch {
                    '\u{201c}' => Some('\u{201d}'),
                    '"' => Some('"'),
                    '\u{ab}' => Some('\u{bb}'),
                    '\u{201e}' => Some('\u{201c}'),
                    '\u{300c}' => Some('\u{300d}'),
                    '\u{300e}' => Some('\u{300f}'),
                    '\u{2018}' => Some('\u{2019}'),
                    _ => None,
                };
                if let Some(closer) = closer {
                    open = Some((closer, index + 1));
                }
            }
            Some((closer, start)) => {
                if ch == closer {
                    if closer == '\u{2019}' {
                        let next_is_letter = text
                            .chars()
                            .nth(index as usize + 1)
                            .is_some_and(char::is_alphanumeric);
                        let prev_is_letter = index > 0
                            && text
                                .chars()
                                .nth(index as usize - 1)
                                .is_some_and(char::is_alphanumeric);
                        if next_is_letter && prev_is_letter {
                            continue;
                        }
                    }
                    spans.push((start, index));
                    open = None;
                }
            }
        }
    }
    if let Some((_, start)) = open {
        spans.push((start, text.chars().count() as u32));
    }
    spans
}

fn prose_piece() -> impl Strategy<Value = String> {
    prop_oneof![
        6 => "[a-z]{1,7}",
        2 => Just(" ".to_owned()),
        1 => prop::sample::select(vec![". ", "! ", "? ", ", ", "\n", "; "]).prop_map(str::to_owned),
        2 => prop::sample::select(vec![
            "\u{201c}", "\u{201d}", "\"", "\u{2018}", "\u{2019}", "\u{ab}", "\u{bb}", "\u{201e}",
            "\u{300c}", "\u{300d}", "'",
        ]).prop_map(str::to_owned),
        1 => Just("Mira\u{2019}s".to_owned()),
        1 => Just("\u{09AE}\u{09BF}\u{09B0}\u{09BE} \u{0995}\u{09BE}\u{09B2}\u{09CB}\u{0964}".to_owned()),
        1 => Just("\u{1F600}\u{200D}\u{1F4A5}".to_owned()),
    ]
}

fn prose() -> impl Strategy<Value = String> {
    prop::collection::vec(prose_piece(), 0..30).prop_map(|pieces| pieces.concat())
}

proptest! {
    #![proptest_config(ProptestConfig { cases: CASES, .. ProptestConfig::default() })]

    #[test]
    fn segment_matches_its_naive_form(text in prose()) {
        let segments = segment(&text);
        prop_assert_eq!(&segments, &naive_segment(&text));
        let len = text.chars().count() as u32;
        prop_assert!(segments.words.iter().all(|(lo, hi)| lo < hi && *hi <= len));
        prop_assert!(segments.words.windows(2).all(|pair| pair[0].1 <= pair[1].0));
        prop_assert!(segments.sentences.windows(2).all(|pair| pair[0].1 <= pair[1].0));
    }

    #[test]
    fn quoted_spans_match_their_naive_form(text in prose()) {
        let spans = quoted_spans(&text);
        prop_assert_eq!(&spans, &naive_quoted_spans(&text));
        let len = text.chars().count() as u32;
        prop_assert!(spans.iter().all(|(lo, hi)| lo <= hi && *hi <= len));
        prop_assert!(spans.windows(2).all(|pair| pair[0].1 < pair[1].0));
    }
}
