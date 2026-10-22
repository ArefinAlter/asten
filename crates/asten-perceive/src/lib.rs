//! Perception without models: the tier-B frontend (§8, §13.2).
//!
//! Everything here is deterministic and works on what the author has entered:
//! cast names and aliases become an Aho–Corasick automaton; concept lattices
//! become a lexicon automaton; quotation marks become dialogue contexts;
//! `[[entity.predicate=value]]` becomes an explicit observation. Every output is
//! an evidence atom with a confidence and provenance, and nothing becomes canon
//! here. Offsets are Unicode scalars, as the anchors count them.

use std::collections::BTreeMap;

use aho_corasick::{AhoCorasick, AhoCorasickBuilder, MatchKind};
use asten_ir::lattice::ConceptLattice;
use asten_ir::{
    Anchor, Context, Entity, EntityId, Mention, Observation, ObservationId, PredicateId,
    PredicateSig, Provenance, Sort, TimeRef, Value, ValueDomain,
};
use serde::{Deserialize, Serialize};
use unicode_normalization::UnicodeNormalization;
use unicode_segmentation::UnicodeSegmentation;

pub const PERCEIVE_VERSION: &str = "tier-b/1";

// ───────────────────────── folding and offsets ─────────────────────────

/// A text folded for matching, with a map from folded chars back to the
/// original scalars. Each grapheme cluster is folded on its own (NFKC, then
/// per-character lowercasing): NFKC composes and reorders marks only inside
/// a cluster, so the result equals the fold of the whole text, which is how
/// patterns are folded, while every folded char still knows the scalars it
/// came from.
#[derive(Clone, Debug)]
pub struct Folded {
    pub chars: Vec<char>,
    /// `origin[i]` is the first original scalar of the cluster folded char
    /// `i` came from (the scalar itself, for a character that folds alone).
    pub origin: Vec<usize>,
    pub text: String,
    /// One past the last original scalar of that cluster.
    origin_end: Vec<usize>,
    /// Byte offset of each folded char in `text`, plus the end.
    byte_starts: Vec<usize>,
}

impl Folded {
    #[must_use]
    pub fn new(original: &str) -> Self {
        let mut chars = Vec::with_capacity(original.len());
        let mut origin = Vec::with_capacity(original.len());
        let mut origin_end = Vec::with_capacity(original.len());
        let mut scalar = 0usize;
        for cluster in original.graphemes(true) {
            let start = scalar;
            scalar += cluster.chars().count();
            for folded in cluster.nfkc().flat_map(fold_char) {
                chars.push(folded);
                origin.push(start);
                origin_end.push(scalar);
            }
        }
        let text: String = chars.iter().collect();
        let mut byte_starts = Vec::with_capacity(chars.len() + 1);
        let mut byte = 0;
        for ch in &chars {
            byte_starts.push(byte);
            byte += ch.len_utf8();
        }
        byte_starts.push(byte);
        Self {
            chars,
            origin,
            text,
            origin_end,
            byte_starts,
        }
    }

    /// Folded char index for a byte offset in `text` (the end offset maps to
    /// `chars.len()`).
    #[must_use]
    pub fn char_index(&self, byte: usize) -> usize {
        match self.byte_starts.binary_search(&byte) {
            Ok(index) => index,
            Err(index) => index.saturating_sub(1),
        }
    }

    /// Original scalar span `[start, end)` for a folded char span: the
    /// scalars of the grapheme clusters the span was folded from, so a span
    /// never cuts a cluster. An empty span gives `(0, 0)`.
    #[must_use]
    pub fn original_span(&self, start: usize, end: usize) -> (u32, u32) {
        if self.origin.is_empty() || start >= end {
            return (0, 0);
        }
        let s = self.origin[start.min(self.origin.len() - 1)];
        let e = self.origin_end[(end - 1).min(self.origin_end.len() - 1)];
        (s as u32, e as u32)
    }
}

/// The matching form of a surface string.
#[must_use]
pub fn matching_form(text: &str) -> String {
    text.trim().nfkc().flat_map(fold_char).collect()
}

/// Case-fold one scalar: lowercase, with Greek final sigma folded to sigma as
/// Unicode case folding does, so a name matches however the word was cased.
fn fold_char(ch: char) -> impl Iterator<Item = char> {
    ch.to_lowercase().map(|folded| {
        if folded == '\u{03C2}' {
            '\u{03C3}'
        } else {
            folded
        }
    })
}

fn is_word_char(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '\u{200d}' || ch == '\'' || ch == '\u{2019}' || is_combining(ch)
}

fn is_combining(ch: char) -> bool {
    matches!(u32::from(ch), 0x0300..=0x036F | 0x0980..=0x09FF | 0x0900..=0x097F | 0x0E31..=0x0E3A | 0x0E47..=0x0E4E | 0x0D80..=0x0DFF)
        && !ch.is_alphabetic()
}

// ───────────────────────── scripts ─────────────────────────

/// ISO 15924 script code of the majority of a text's letters.
#[must_use]
pub fn detect_script(text: &str) -> &'static str {
    let mut counts: BTreeMap<&'static str, usize> = BTreeMap::new();
    for ch in text.chars().filter(|ch| ch.is_alphabetic()) {
        let script = match u32::from(ch) {
            0x0041..=0x024F | 0x1E00..=0x1EFF => "Latn",
            0x0370..=0x03FF => "Grek",
            0x0400..=0x052F => "Cyrl",
            0x0530..=0x058F => "Armn",
            0x0590..=0x05FF => "Hebr",
            0x0600..=0x06FF | 0x0750..=0x077F => "Arab",
            0x0900..=0x097F => "Deva",
            0x0980..=0x09FF => "Beng",
            0x0A00..=0x0A7F => "Guru",
            0x0A80..=0x0AFF => "Gujr",
            0x0B00..=0x0B7F => "Orya",
            0x0B80..=0x0BFF => "Taml",
            0x0C00..=0x0C7F => "Telu",
            0x0C80..=0x0CFF => "Knda",
            0x0D00..=0x0D7F => "Mlym",
            0x0D80..=0x0DFF => "Sinh",
            0x0E00..=0x0E7F => "Thai",
            0x0E80..=0x0EFF => "Laoo",
            0x0F00..=0x0FFF => "Tibt",
            0x1000..=0x109F => "Mymr",
            0x10A0..=0x10FF => "Geor",
            0x1100..=0x11FF | 0xAC00..=0xD7AF => "Hang",
            0x1780..=0x17FF => "Khmr",
            0x3040..=0x30FF => "Jpan",
            0x3400..=0x4DBF | 0x4E00..=0x9FFF | 0x20000..=0x2A6DF => "Hani",
            0x11100..=0x1114F => "Cakm",
            _ => "Zyyy",
        };
        *counts.entry(script).or_default() += 1;
    }
    counts
        .into_iter()
        .max_by_key(|(_, count)| *count)
        .map(|(script, _)| script)
        .unwrap_or("Zyyy")
}

/// Scripts written without spaces between words, where boundary checks must be skipped.
#[must_use]
pub fn script_has_word_spaces(script: &str) -> bool {
    !matches!(
        script,
        "Hani" | "Jpan" | "Thai" | "Laoo" | "Khmr" | "Mymr" | "Tibt"
    )
}

/// Scripts with letter case, where capitalisation is a usable name cue.
#[must_use]
pub fn script_has_case(script: &str) -> bool {
    matches!(script, "Latn" | "Grek" | "Cyrl" | "Armn" | "Geor")
}

/// Support tier for a script in this build (§13.2).
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum SupportTier {
    /// Canon checks between settled facts only.
    C,
    /// Plus deterministic mentions and lexicon observations.
    B,
    /// Plus trained perception models.
    A,
}

#[must_use]
pub fn tier_for(_script: &str) -> SupportTier {
    SupportTier::B
}

// ───────────────────────── segmentation ─────────────────────────

/// Sentence and word spans in scalar offsets.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Segments {
    pub sentences: Vec<(u32, u32)>,
    pub words: Vec<(u32, u32)>,
}

#[must_use]
pub fn segment(text: &str) -> Segments {
    // Both iterators yield byte offsets in increasing order, so one cursor
    // converts them to scalar offsets in a single pass instead of recounting
    // the text from the start at every boundary.
    let mut cursor = ScalarCursor::new(text);
    let mut sentences = Vec::new();
    let mut offset = 0usize;
    for sentence in text.unicode_sentences() {
        let start = text[offset..]
            .find(sentence)
            .map(|found| offset + found)
            .unwrap_or(offset);
        let end = start + sentence.len();
        sentences.push((cursor.scalar(start), cursor.scalar(end)));
        offset = end;
    }
    let mut cursor = ScalarCursor::new(text);
    let words = text
        .unicode_word_indices()
        .map(|(byte, word)| (cursor.scalar(byte), cursor.scalar(byte + word.len())))
        .collect();
    Segments { sentences, words }
}

/// Converts byte offsets to scalar offsets, counting only the text between
/// successive queries when they arrive in increasing order.
struct ScalarCursor<'a> {
    text: &'a str,
    byte: usize,
    scalar: u32,
}

impl<'a> ScalarCursor<'a> {
    fn new(text: &'a str) -> Self {
        Self {
            text,
            byte: 0,
            scalar: 0,
        }
    }

    fn scalar(&mut self, byte: usize) -> u32 {
        let byte = byte.min(self.text.len());
        if byte < self.byte {
            self.byte = 0;
            self.scalar = 0;
        }
        self.scalar += self.text[self.byte..byte].chars().count() as u32;
        self.byte = byte;
        self.scalar
    }
}

/// Spans inside quotation marks, as scalar offsets. Handles “ ”, " ", « », „ “, 「 」 and ‘ ’.
#[must_use]
pub fn quoted_spans(text: &str) -> Vec<(u32, u32)> {
    let mut spans = Vec::new();
    let mut open: Option<(char, u32)> = None;
    let chars: Vec<char> = text.chars().collect();
    for (index, ch) in chars.iter().copied().enumerate() {
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
                    // An apostrophe inside a word is not a closing single quote.
                    if closer == '\u{2019}' {
                        let next_is_letter = chars
                            .get(index as usize + 1)
                            .is_some_and(|next| next.is_alphanumeric());
                        let prev_is_letter =
                            index > 0 && chars[index as usize - 1].is_alphanumeric();
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
        // Unclosed quote runs to the end of the block.
        spans.push((start, chars.len() as u32));
    }
    spans
}

fn in_spans(spans: &[(u32, u32)], start: u32, end: u32) -> bool {
    spans.iter().any(|(lo, hi)| start >= *lo && end <= *hi)
}

// ───────────────────────── mentions ─────────────────────────

struct Pattern {
    entity: EntityId,
    surface: String,
    is_name: bool,
}

/// An Aho–Corasick matcher over every cast name and alias.
pub struct Matcher {
    automaton: Option<AhoCorasick>,
    patterns: Vec<Pattern>,
    /// Fingerprint of the cast it was built from, for cache keys.
    pub cast_fingerprint: String,
}

impl Matcher {
    #[must_use]
    pub fn new(entities: &[Entity]) -> Self {
        let mut patterns = Vec::new();
        let mut parts: Vec<String> = Vec::new();
        for entity in entities {
            for (index, form) in entity.surface_forms().enumerate() {
                let folded = matching_form(form);
                if folded.chars().count() < 2 {
                    continue;
                }
                parts.push(format!("{}={}", entity.id.0, folded));
                patterns.push(Pattern {
                    entity: entity.id.clone(),
                    surface: folded,
                    is_name: index == 0,
                });
            }
        }
        parts.sort();
        let cast_fingerprint = asten_ir::sha256_hex(parts.join("\u{1f}").as_bytes());
        let automaton = if patterns.is_empty() {
            None
        } else {
            AhoCorasickBuilder::new()
                .match_kind(MatchKind::LeftmostLongest)
                .build(patterns.iter().map(|pattern| pattern.surface.as_str()))
                .ok()
        };
        Self {
            automaton,
            patterns,
            cast_fingerprint,
        }
    }

    /// Find mentions in a block.
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn mentions(
        &self,
        folded: &Folded,
        document: &str,
        block: &str,
        reading_pos: i64,
        scene_context: &Context,
        quotes: &[(u32, u32)],
        script: &str,
    ) -> Vec<Mention> {
        let Some(automaton) = &self.automaton else {
            return Vec::new();
        };
        let boundaries = script_has_word_spaces(script);
        let mut mentions = Vec::new();
        for found in automaton.find_iter(&folded.text) {
            let start = folded.char_index(found.start());
            let end = folded.char_index(found.end());
            let pattern = &self.patterns[found.pattern().as_usize()];
            let mut confidence: f32 = if pattern.is_name { 0.97 } else { 0.9 };
            if boundaries {
                let before_ok = start == 0 || !is_word_char(folded.chars[start - 1]);
                if !before_ok {
                    continue;
                }
                // Letters right after the name: an inflection ("Mira's", "Miran") up to four scalars.
                let mut suffix = 0;
                let mut cursor = end;
                while cursor < folded.chars.len() && is_word_char(folded.chars[cursor]) {
                    suffix += 1;
                    cursor += 1;
                }
                if suffix > 4 {
                    continue;
                }
                if suffix > 0 {
                    confidence *= 0.75;
                }
            }
            let (s, e) = folded.original_span(start, end);
            let context = if in_spans(quotes, s, e) {
                Context::Says { speaker: None }
            } else {
                scene_context.clone()
            };
            mentions.push(Mention {
                anchor: Anchor::new(document, block, s, e),
                entity: pattern.entity.clone(),
                surface: pattern.surface.clone(),
                confidence,
                reading_pos,
                context,
            });
        }
        mentions
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.patterns.is_empty()
    }
}

// ───────────────────────── lexicon observations ─────────────────────────

struct LexiconEntry {
    lattice: String,
    concept: asten_ir::ConceptId,
    form: String,
}

/// Concept lexicon over every lattice, with the predicates that use each lattice.
pub struct Lexicon {
    automaton: Option<AhoCorasick>,
    entries: Vec<LexiconEntry>,
    predicates: Vec<PredicateSig>,
    cues: Vec<(PredicateId, Vec<String>)>,
}

impl Lexicon {
    #[must_use]
    pub fn new(lattices: &BTreeMap<String, ConceptLattice>, predicates: &[PredicateSig]) -> Self {
        let mut entries = Vec::new();
        for (name, lattice) in lattices {
            let used = predicates.iter().any(
                |sig| matches!(&sig.domain, ValueDomain::Concept { lattice } if lattice == name),
            );
            if !used {
                continue;
            }
            for (form, concept) in lattice.forms() {
                if form.chars().count() < 2 {
                    continue;
                }
                entries.push(LexiconEntry {
                    lattice: name.clone(),
                    concept: concept.clone(),
                    form: form.to_owned(),
                });
            }
        }
        let automaton = if entries.is_empty() {
            None
        } else {
            AhoCorasickBuilder::new()
                .match_kind(MatchKind::LeftmostLongest)
                .build(entries.iter().map(|entry| entry.form.as_str()))
                .ok()
        };
        let cues = predicates
            .iter()
            .map(|sig| {
                (
                    sig.id.clone(),
                    sig.cues.iter().map(|cue| matching_form(cue)).collect(),
                )
            })
            .collect();
        Self {
            automaton,
            entries,
            predicates: predicates.to_vec(),
            cues,
        }
    }

    /// Observations in a block, attributed to the nearest mention in the same sentence.
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn observe(
        &self,
        folded: &Folded,
        segments: &Segments,
        mentions: &[Mention],
        entities: &[Entity],
        document: &str,
        block: &str,
        reading_pos: i64,
        scene_context: &Context,
        quotes: &[(u32, u32)],
        script: &str,
    ) -> Vec<Observation> {
        let Some(automaton) = &self.automaton else {
            return Vec::new();
        };
        if mentions.is_empty() {
            return Vec::new();
        }
        let boundaries = script_has_word_spaces(script);
        let mut observations = Vec::new();
        for found in automaton.find_iter(&folded.text) {
            let start = folded.char_index(found.start());
            let end = folded.char_index(found.end());
            if boundaries {
                let before_ok = start == 0 || !is_word_char(folded.chars[start - 1]);
                let after_ok = end >= folded.chars.len() || !is_word_char(folded.chars[end]);
                if !before_ok || !after_ok {
                    continue;
                }
            }
            let (value_start, value_end) = folded.original_span(start, end);
            let entry = &self.entries[found.pattern().as_usize()];
            let Some(sentence) = segments
                .sentences
                .iter()
                .find(|(lo, hi)| value_start >= *lo && value_start < *hi)
                .copied()
            else {
                continue;
            };
            // Nearest mention in the sentence, preferring one before the value.
            let Some(mention) = mentions
                .iter()
                .filter(|mention| {
                    mention.anchor.start >= sentence.0 && mention.anchor.end <= sentence.1
                })
                .min_by_key(|mention| {
                    if mention.anchor.end <= value_start {
                        (0, value_start - mention.anchor.end)
                    } else {
                        (1, mention.anchor.start.saturating_sub(value_end))
                    }
                })
            else {
                continue;
            };
            let sort = entities
                .iter()
                .find(|entity| entity.id == mention.entity)
                .map(|entity| entity.sort.clone())
                .unwrap_or(Sort::Other);
            let candidates: Vec<&PredicateSig> = self
                .predicates
                .iter()
                .filter(|sig| matches!(&sig.domain, ValueDomain::Concept { lattice } if *lattice == entry.lattice))
                .filter(|sig| sig.subject_sorts.is_empty() || sig.subject_sorts.contains(&sort))
                .collect();
            if candidates.is_empty() {
                continue;
            }
            // Distance from the value to the nearest cue word of each candidate predicate.
            let sentence_word_spans: Vec<((u32, u32), &str)> = segments
                .words
                .iter()
                .filter(|(lo, hi)| *lo >= sentence.0 && *hi <= sentence.1)
                .map(|(lo, hi)| ((*lo, *hi), folded_slice(folded, *lo, *hi)))
                .collect();
            let mut cued: Vec<(&PredicateSig, u32)> = candidates
                .iter()
                .copied()
                .filter_map(|sig| {
                    let cues = &self.cues.iter().find(|(id, _)| *id == sig.id)?.1;
                    sentence_word_spans
                        .iter()
                        .filter(|(_, word)| cues.iter().any(|cue| cue == word))
                        .map(|((lo, hi), _)| {
                            if *hi <= value_start {
                                value_start - *hi
                            } else {
                                lo.saturating_sub(value_end)
                            }
                        })
                        .min()
                        .map(|distance| (sig, distance))
                })
                .collect();
            cued.sort_by_key(|(_, distance)| *distance);
            let (predicate, mut confidence) = match (cued.len(), candidates.len()) {
                (1, _) => (cued[0].0, 0.82),
                (0, 1) => (candidates[0], 0.45),
                (n, _) if n >= 2 && cued[0].1 < cued[1].1 => (cued[0].0, 0.72),
                _ => continue,
            };
            // Negation within three words before the value.
            let preceding: Vec<&str> = segments
                .words
                .iter()
                .filter(|(_, hi)| *hi <= value_start && *hi + 24 >= value_start)
                .rev()
                .take(3)
                .map(|(lo, hi)| folded_slice(folded, *lo, *hi))
                .collect();
            let negated = preceding.iter().any(|word| {
                matches!(
                    *word,
                    "not"
                        | "no"
                        | "never"
                        | "n't"
                        | "isn't"
                        | "wasn't"
                        | "weren't"
                        | "hadn't"
                        | "nor"
                        | "non"
                        | "নয়"
                        | "না"
                        | "nicht"
                        | "kein"
                        | "pas"
                        | "ne"
                )
            });
            let context = if in_spans(quotes, value_start, value_end) {
                Context::Says { speaker: None }
            } else {
                scene_context.clone()
            };
            if matches!(context, Context::Says { .. }) {
                confidence *= 0.7;
            }
            confidence *= mention.confidence;
            let anchor = Anchor::new(document, block, value_start, value_end);
            let id = ObservationId::new(format!(
                "obs:{block}:{value_start}:{}:{}",
                predicate.id.0, entry.concept.0
            ));
            observations.push(Observation {
                id,
                fluent: asten_ir::Fluent {
                    predicate: predicate.id.clone(),
                    subject: mention.entity.clone(),
                    value: Value::Concept {
                        id: entry.concept.clone(),
                        label: entry.form.clone(),
                    },
                },
                context,
                at: TimeRef::Unknown,
                anchor,
                confidence,
                negated,
                provenance: Provenance::Text {
                    anchor: Anchor::new(document, block, value_start, value_end),
                },
                reading_pos,
            });
        }
        observations
    }
}

fn folded_slice(folded: &Folded, lo: u32, hi: u32) -> &str {
    // Map original scalar offsets to folded char indices (first folded char with that origin).
    let start = folded
        .origin
        .partition_point(|origin| (*origin as u32) < lo);
    let end = folded
        .origin
        .partition_point(|origin| (*origin as u32) < hi);
    let byte_start = folded
        .byte_starts
        .get(start)
        .copied()
        .unwrap_or(folded.text.len());
    let byte_end = folded
        .byte_starts
        .get(end)
        .copied()
        .unwrap_or(folded.text.len());
    &folded.text[byte_start.min(byte_end)..byte_end]
}

// ───────────────────────── values ─────────────────────────

/// Resolve author-typed text to a value in a predicate's domain. Unknown
/// concepts become text values the author can map later.
#[must_use]
pub fn resolve_value(
    sig: Option<&PredicateSig>,
    lattices: &BTreeMap<String, ConceptLattice>,
    text: &str,
) -> Value {
    let label = text.trim().to_owned();
    match sig.map(|sig| &sig.domain) {
        Some(ValueDomain::Concept { lattice }) => {
            if let Some(concept) = lattices
                .get(lattice)
                .and_then(|lattice| lattice.resolve(&label))
            {
                return Value::Concept { id: concept, label };
            }
            Value::Text {
                norm: matching_form(&label),
                label,
            }
        }
        Some(ValueDomain::Quantity { .. }) => parse_quantity(&label).unwrap_or(Value::Text {
            norm: matching_form(&label),
            label,
        }),
        Some(ValueDomain::Entity { .. }) => Value::Entity {
            id: EntityId::new(entity_key(&label)),
        },
        Some(ValueDomain::Boolean) => Value::Bool {
            value: matches!(
                matching_form(&label).as_str(),
                "true" | "yes" | "1" | "alive"
            ),
        },
        Some(ValueDomain::Text) | None => {
            // Without a signature, try every lattice so colors still become concepts.
            for lattice in lattices.values() {
                if let Some(concept) = lattice.resolve(&label) {
                    return Value::Concept { id: concept, label };
                }
            }
            parse_quantity(&label).unwrap_or(Value::Text {
                norm: matching_form(&label),
                label,
            })
        }
    }
}

/// Entity key from a display name: folded, spaces to underscores.
#[must_use]
pub fn entity_key(name: &str) -> String {
    matching_form(name)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("_")
}

/// "about thirty" → [27, 33]; "32" → [32, 32]; "six feet" → centimetres; "1.8 m".
#[must_use]
pub fn parse_quantity(text: &str) -> Option<Value> {
    let folded = matching_form(text);
    let fuzzy = folded.starts_with("about ")
        || folded.starts_with("around ")
        || folded.starts_with("roughly ")
        || folded.starts_with("nearly ")
        || folded.starts_with("almost ")
        || folded.starts_with("some ")
        || folded.starts_with('~');
    let body = folded
        .trim_start_matches("about ")
        .trim_start_matches("around ")
        .trim_start_matches("roughly ")
        .trim_start_matches("nearly ")
        .trim_start_matches("almost ")
        .trim_start_matches("some ")
        .trim_start_matches('~')
        .trim()
        .to_owned();
    let mut parts = body.split_whitespace();
    let first = parts.next()?;
    let number: f64 = match first.parse::<f64>() {
        Ok(number) => number,
        Err(_) => f64::from(number_word(first)?),
    };
    let unit = parts.next().unwrap_or("");
    let (factor, _) = unit_factor(unit);
    let value = number * factor;
    let (lo, hi) = if fuzzy {
        (value * 0.9, value * 1.1)
    } else {
        (value, value)
    };
    Some(Value::Quantity {
        lo,
        hi,
        label: text.trim().to_owned(),
    })
}

fn number_word(word: &str) -> Option<u32> {
    Some(match word {
        "zero" => 0,
        "one" | "a" | "an" => 1,
        "two" => 2,
        "three" => 3,
        "four" => 4,
        "five" => 5,
        "six" => 6,
        "seven" => 7,
        "eight" => 8,
        "nine" => 9,
        "ten" => 10,
        "eleven" => 11,
        "twelve" => 12,
        "thirteen" => 13,
        "fourteen" => 14,
        "fifteen" => 15,
        "sixteen" => 16,
        "seventeen" => 17,
        "eighteen" => 18,
        "nineteen" => 19,
        "twenty" => 20,
        "thirty" => 30,
        "forty" => 40,
        "fifty" => 50,
        "sixty" => 60,
        "seventy" => 70,
        "eighty" => 80,
        "ninety" => 90,
        "hundred" => 100,
        "thousand" => 1000,
        _ => return None,
    })
}

fn unit_factor(unit: &str) -> (f64, &'static str) {
    let unit = unit.trim_end_matches('.');
    match unit {
        "cm" | "centimetre" | "centimetres" | "centimeter" | "centimeters" => (1.0, "cm"),
        "m" | "metre" | "metres" | "meter" | "meters" => (100.0, "cm"),
        "km" | "kilometre" | "kilometres" | "kilometer" | "kilometers" => (100_000.0, "cm"),
        "mm" => (0.1, "cm"),
        "in" | "inch" | "inches" | "\"" => (2.54, "cm"),
        "ft" | "foot" | "feet" | "'" => (30.48, "cm"),
        "yd" | "yard" | "yards" => (91.44, "cm"),
        "mi" | "mile" | "miles" => (160_934.4, "cm"),
        "kg" | "kilogram" | "kilograms" => (1000.0, "g"),
        "g" | "gram" | "grams" => (1.0, "g"),
        "lb" | "lbs" | "pound" | "pounds" => (453.592, "g"),
        "st" | "stone" => (6350.29, "g"),
        "min" | "minute" | "minutes" => (1.0, "min"),
        "h" | "hr" | "hour" | "hours" => (60.0, "min"),
        "d" | "day" | "days" => (1440.0, "min"),
        _ => (1.0, ""),
    }
}

// ───────────────────────── inline assertions ─────────────────────────

/// `[[mira.hair_color=black]]` typed in the prose: an explicit observation.
#[must_use]
pub fn inline_assertions(
    text: &str,
    document: &str,
    block: &str,
    reading_pos: i64,
    context: &Context,
    predicates: &[PredicateSig],
    lattices: &BTreeMap<String, ConceptLattice>,
) -> Vec<Observation> {
    let chars: Vec<char> = text.chars().collect();
    let mut observations = Vec::new();
    let mut index = 0;
    while index + 1 < chars.len() {
        if chars[index] != '[' || chars[index + 1] != '[' {
            index += 1;
            continue;
        }
        let start = index;
        let Some(close) = (start + 2..chars.len().saturating_sub(1))
            .find(|&i| chars[i] == ']' && chars[i + 1] == ']')
        else {
            break;
        };
        let inner: String = chars[start + 2..close].iter().collect();
        index = close + 2;
        let Some((lhs, value)) = inner.split_once('=') else {
            continue;
        };
        let Some((entity, predicate)) = lhs.split_once('.') else {
            continue;
        };
        let predicate_id = PredicateId::new(entity_key(predicate));
        let sig = predicates.iter().find(|sig| sig.id == predicate_id);
        let value = resolve_value(sig, lattices, value);
        let anchor = Anchor::new(document, block, start as u32, index as u32).with_text(text);
        observations.push(Observation {
            id: ObservationId::new(format!("inline:{block}:{start}")),
            fluent: asten_ir::Fluent {
                predicate: predicate_id,
                subject: EntityId::new(entity_key(entity)),
                value,
            },
            context: context.clone(),
            at: TimeRef::Unknown,
            anchor: anchor.clone(),
            confidence: 1.0,
            negated: false,
            provenance: Provenance::Text { anchor },
            reading_pos,
        });
    }
    observations
}

// ───────────────────────── name candidates ─────────────────────────

const STOP: &[&str] = &[
    "i", "the", "a", "an", "and", "but", "or", "then", "when", "while", "after", "before", "she",
    "he", "they", "it", "we", "you", "his", "her", "their", "its", "our", "my", "your", "this",
    "that", "these", "those", "there", "here", "what", "who", "why", "how", "where", "yes", "no",
    "not", "if", "so", "as", "at", "by", "for", "from", "in", "into", "of", "on", "to", "with",
    "chapter", "part", "once", "now", "later", "still", "yet", "all", "every", "some", "none",
    "one", "two", "three", "oh", "ah", "well", "god",
];

/// Titles that name someone only together with the word after them.
const HONORIFICS: &[&str] = &[
    "mr",
    "mrs",
    "ms",
    "dr",
    "sir",
    "lady",
    "lord",
    "captain",
    "king",
    "queen",
    "prince",
    "princess",
    "master",
    "mistress",
    "saint",
    "st",
    "professor",
    "general",
    "sergeant",
    "doctor",
    "father",
    "mother",
    "brother",
    "sister",
    "aunt",
    "uncle",
];

/// Capitalised words that are not cast members, as unknown-name candidates.
/// Only in scripts with case; sentence-initial words count only when the same
/// word also appears capitalised elsewhere in the block.
#[must_use]
pub fn name_candidates(
    text: &str,
    segments: &Segments,
    mentions: &[Mention],
    document: &str,
    block: &str,
    reading_pos: i64,
    script: &str,
) -> Vec<(Anchor, String, i64)> {
    if !script_has_case(script) {
        return Vec::new();
    }
    let chars: Vec<char> = text.chars().collect();
    let mut candidates: Vec<(u32, u32, String, bool)> = Vec::new();
    for (lo, hi) in &segments.words {
        let word: String = chars[*lo as usize..*hi as usize].iter().collect();
        let mut letters = word.chars();
        let Some(first) = letters.next() else {
            continue;
        };
        if !first.is_uppercase()
            || !letters.all(|c| c.is_lowercase() || c == '\'' || c == '\u{2019}')
            || word.chars().count() < 2
        {
            continue;
        }
        let lower = word.to_lowercase();
        if STOP.contains(&lower.as_str()) {
            continue;
        }
        if mentions
            .iter()
            .any(|mention| mention.anchor.start <= *lo && *hi <= mention.anchor.end)
        {
            continue;
        }
        let sentence_initial = segments.sentences.iter().any(|(start, _)| {
            // First word of a sentence: nothing but whitespace or quotes before it since the sentence start.
            *start <= *lo
                && chars[*start as usize..*lo as usize].iter().all(|c| {
                    c.is_whitespace()
                        || matches!(
                            c,
                            '"' | '\u{201c}'
                                | '\u{2018}'
                                | '\u{ab}'
                                | '\u{300c}'
                                | '('
                                | '\u{2014}'
                        )
                })
        });
        candidates.push((*lo, *hi, word, sentence_initial));
    }
    // Merge adjacent capitalised words ("Lord Aric" after the stoplist drops "Lord": keep "Aric").
    let mut out = Vec::new();
    let mut index = 0;
    while index < candidates.len() {
        let (lo, mut hi, mut word, initial) = candidates[index].clone();
        let mut next = index + 1;
        while next < candidates.len()
            && candidates[next].0 <= hi + 1
            && chars[hi as usize..candidates[next].0 as usize]
                .iter()
                .all(|c| *c == ' ')
        {
            hi = candidates[next].1;
            word.push(' ');
            word.push_str(&candidates[next].2);
            next += 1;
        }
        if HONORIFICS.contains(&word.to_lowercase().as_str()) {
            // A title on its own names nobody.
            index = next;
            continue;
        }
        let repeated_elsewhere = candidates
            .iter()
            .filter(|(_, _, other, init)| *other == word && !*init)
            .count()
            > 0;
        if !initial || repeated_elsewhere || word.contains(' ') {
            out.push((
                Anchor::new(document, block, lo, hi).with_text(text),
                word,
                reading_pos,
            ));
        }
        index = next;
    }
    out
}

// ───────────────────────── per-block perception ─────────────────────────

/// Everything perceived in one block.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct BlockPerception {
    pub fingerprint: String,
    pub script: String,
    pub mentions: Vec<Mention>,
    pub observations: Vec<Observation>,
    pub unknown_names: Vec<(Anchor, String, i64)>,
    pub quoted: Vec<(u32, u32)>,
}

/// Cache key for a block's perception.
#[must_use]
pub fn cache_key(
    fingerprint: &str,
    cast_fingerprint: &str,
    lexicon_fingerprint: &str,
    context: &Context,
) -> String {
    format!(
        "{fingerprint}|{cast_fingerprint}|{lexicon_fingerprint}|{}|{PERCEIVE_VERSION}",
        context.label()
    )
}

/// Perceive one block deterministically.
#[must_use]
#[allow(clippy::too_many_arguments)]
pub fn perceive_block(
    text: &str,
    fingerprint: &str,
    document: &str,
    block: &str,
    reading_pos: i64,
    scene_context: &Context,
    matcher: &Matcher,
    lexicon: &Lexicon,
    entities: &[Entity],
    predicates: &[PredicateSig],
    lattices: &BTreeMap<String, ConceptLattice>,
) -> BlockPerception {
    let folded = Folded::new(text);
    let script = detect_script(text);
    let segments = segment(text);
    let quoted = quoted_spans(text);
    let mut mentions = matcher.mentions(
        &folded,
        document,
        block,
        reading_pos,
        scene_context,
        &quoted,
        script,
    );
    for mention in &mut mentions {
        mention.anchor = std::mem::take(&mut mention.anchor).with_text(text);
    }
    let mut observations = lexicon.observe(
        &folded,
        &segments,
        &mentions,
        entities,
        document,
        block,
        reading_pos,
        scene_context,
        &quoted,
        script,
    );
    for observation in &mut observations {
        observation.anchor = std::mem::take(&mut observation.anchor).with_text(text);
        if let Provenance::Text { anchor } = &mut observation.provenance {
            *anchor = observation.anchor.clone();
        }
    }
    observations.extend(inline_assertions(
        text,
        document,
        block,
        reading_pos,
        scene_context,
        predicates,
        lattices,
    ));
    let unknown_names = name_candidates(
        text,
        &segments,
        &mentions,
        document,
        block,
        reading_pos,
        script,
    );
    BlockPerception {
        fingerprint: fingerprint.to_owned(),
        script: script.to_owned(),
        mentions,
        observations,
        unknown_names,
        quoted,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use asten_ir::lattice::color_lattice;
    use asten_ir::{Alias, Cardinality, Mutability};

    fn cast() -> Vec<Entity> {
        vec![
            Entity {
                id: EntityId::new("mira"),
                sort: Sort::Character,
                name: "Mira".into(),
                aliases: vec![
                    Alias {
                        form: "the dyer's girl".into(),
                        from_tick: None,
                        used_by: None,
                    },
                    Alias {
                        form: "মিরা".into(),
                        from_tick: None,
                        used_by: None,
                    },
                ],
                introduced_at: None,
            },
            Entity {
                id: EntityId::new("oren"),
                sort: Sort::Character,
                name: "Captain Oren".into(),
                aliases: vec![Alias {
                    form: "Oren".into(),
                    from_tick: None,
                    used_by: None,
                }],
                introduced_at: None,
            },
            Entity {
                id: EntityId::new("veld"),
                sort: Sort::Place,
                name: "Veld".into(),
                aliases: vec![],
                introduced_at: None,
            },
        ]
    }

    fn predicates() -> Vec<PredicateSig> {
        vec![
            PredicateSig {
                id: PredicateId::new("hair_color"),
                label: "hair".into(),
                subject_sorts: vec![Sort::Character],
                domain: ValueDomain::Concept {
                    lattice: "color".into(),
                },
                cardinality: Cardinality::Functional,
                mutability: Mutability::Mutable,
                relation: Default::default(),
                cues: vec!["hair".into(), "locks".into(), "braid".into(), "চুল".into()],
                pack: "scribe".into(),
            },
            PredicateSig {
                id: PredicateId::new("eye_color"),
                label: "eyes".into(),
                subject_sorts: vec![Sort::Character],
                domain: ValueDomain::Concept {
                    lattice: "color".into(),
                },
                cardinality: Cardinality::Functional,
                mutability: Mutability::Mutable,
                relation: Default::default(),
                cues: vec!["eyes".into(), "eye".into(), "gaze".into()],
                pack: "scribe".into(),
            },
        ]
    }

    fn lattices() -> BTreeMap<String, ConceptLattice> {
        let mut map = BTreeMap::new();
        map.insert("color".into(), color_lattice());
        map
    }

    #[test]
    fn finds_mentions_with_suffixes_and_dialogue_context() {
        let entities = cast();
        let matcher = Matcher::new(&entities);
        let text = "Mira's hand shook. \u{201c}Oren is dead,\u{201d} said the dyer's girl.";
        let folded = Folded::new(text);
        let quotes = quoted_spans(text);
        let mentions = matcher.mentions(&folded, "d", "b", 1, &Context::Narration, &quotes, "Latn");
        let surfaces: Vec<(&str, &Context)> = mentions
            .iter()
            .map(|m| (m.anchor.quote_or(text), &m.context))
            .collect();
        assert_eq!(mentions.len(), 3, "{surfaces:?}");
        assert_eq!(mentions[0].entity, EntityId::new("mira"));
        assert!(mentions[0].confidence < 0.9, "suffix lowers confidence");
        assert_eq!(mentions[1].entity, EntityId::new("oren"));
        assert_eq!(mentions[1].context, Context::Says { speaker: None });
        assert_eq!(mentions[2].entity, EntityId::new("mira"));
        assert_eq!(mentions[2].context, Context::Narration);
    }

    trait QuoteOr {
        fn quote_or<'a>(&self, text: &'a str) -> &'a str;
    }

    impl QuoteOr for Anchor {
        fn quote_or<'a>(&self, text: &'a str) -> &'a str {
            let start = text
                .char_indices()
                .nth(self.start as usize)
                .map(|(b, _)| b)
                .unwrap_or(text.len());
            let end = text
                .char_indices()
                .nth(self.end as usize)
                .map(|(b, _)| b)
                .unwrap_or(text.len());
            &text[start..end]
        }
    }

    #[test]
    fn observes_colors_with_cues_in_two_languages() {
        let entities = cast();
        let predicates = predicates();
        let lattices = lattices();
        let matcher = Matcher::new(&entities);
        let lexicon = Lexicon::new(&lattices, &predicates);
        let perception = perceive_block(
            "Mira pushed her black hair out of her eyes. Oren watched with green eyes.",
            "fp",
            "d",
            "b",
            1,
            &Context::Narration,
            &matcher,
            &lexicon,
            &entities,
            &predicates,
            &lattices,
        );
        assert_eq!(
            perception.observations.len(),
            2,
            "{:?}",
            perception.observations
        );
        let hair = &perception.observations[0];
        assert_eq!(hair.fluent.predicate.0, "hair_color");
        assert_eq!(hair.fluent.subject.0, "mira");
        assert_eq!(hair.fluent.value.key(), "c:black");
        assert_eq!(hair.anchor.start, 16);
        let eyes = &perception.observations[1];
        assert_eq!(eyes.fluent.predicate.0, "eye_color");
        assert_eq!(eyes.fluent.subject.0, "oren");

        let bengali = perceive_block(
            "মিরা তার কালো চুল সরিয়ে দিল।",
            "fp",
            "d",
            "b",
            1,
            &Context::Narration,
            &matcher,
            &lexicon,
            &entities,
            &predicates,
            &lattices,
        );
        assert_eq!(bengali.script, "Beng");
        assert_eq!(bengali.observations.len(), 1);
        assert_eq!(bengali.observations[0].fluent.value.key(), "c:black");
    }

    #[test]
    fn negation_and_dialogue_are_marked() {
        let entities = cast();
        let predicates = predicates();
        let lattices = lattices();
        let matcher = Matcher::new(&entities);
        let lexicon = Lexicon::new(&lattices, &predicates);
        let perception = perceive_block(
            "Mira's hair was not black. \u{201c}Oren has blue eyes,\u{201d} she lied.",
            "fp",
            "d",
            "b",
            1,
            &Context::Narration,
            &matcher,
            &lexicon,
            &entities,
            &predicates,
            &lattices,
        );
        assert!(perception.observations[0].negated);
        assert_eq!(
            perception.observations[1].context,
            Context::Says { speaker: None }
        );
        assert!(perception.observations[1].confidence < 0.7);
    }

    #[test]
    fn inline_assertions_and_name_candidates() {
        let entities = cast();
        let predicates = predicates();
        let lattices = lattices();
        let matcher = Matcher::new(&entities);
        let lexicon = Lexicon::new(&lattices, &predicates);
        let text = "Lord Aric bowed to Mira. [[mira.hair_color=auburn]] Then Aric left for Veld.";
        let perception = perceive_block(
            text,
            "fp",
            "d",
            "b",
            4,
            &Context::Narration,
            &matcher,
            &lexicon,
            &entities,
            &predicates,
            &lattices,
        );
        assert_eq!(perception.observations.len(), 1);
        assert_eq!(perception.observations[0].fluent.value.key(), "c:auburn");
        assert_eq!(perception.observations[0].confidence, 1.0);
        let names: Vec<&str> = perception
            .unknown_names
            .iter()
            .map(|(_, name, _)| name.as_str())
            .collect();
        assert!(names.contains(&"Aric"), "{names:?}");
        assert!(!names.contains(&"Then"));
    }

    #[test]
    fn values_resolve_through_lattices_and_units() {
        let lattices = lattices();
        let sig = predicates().remove(0);
        assert_eq!(
            resolve_value(Some(&sig), &lattices, "Auburn").key(),
            "c:auburn"
        );
        assert!(matches!(
            resolve_value(Some(&sig), &lattices, "storm-grey"),
            Value::Text { .. }
        ));
        let Value::Quantity { lo, hi, .. } = parse_quantity("about six feet").unwrap() else {
            panic!()
        };
        assert!((lo - 164.6).abs() < 0.1 && (hi - 201.2).abs() < 0.1);
        assert_eq!(entity_key("Captain  Oren"), "captain_oren");
    }

    #[test]
    fn scripts_are_detected() {
        assert_eq!(detect_script("Mira waited."), "Latn");
        assert_eq!(detect_script("মিরা অপেক্ষা করল"), "Beng");
        assert_eq!(detect_script("ميرا انتظرت"), "Arab");
        assert_eq!(detect_script("美拉等待"), "Hani");
        assert!(!script_has_word_spaces("Hani"));
    }
}
