//! The incremental canon engine (§5, §7).
//!
//! Inputs are the project's blocks, canon and settings at a revision. The
//! engine perceives each block once per (fingerprint, cast, lexicon, context)
//! and caches the result, so an edit to one paragraph re-perceives one block.
//! Perception becomes the Story IR; the analyses in `asten-logic` run over it;
//! `asten-diagnostics` turns violations into marks with computed severity,
//! suppressions and baselines. The output is a snapshot and a diff against the
//! previous revision, which clients receive as a change set, not the full list.

use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;
use std::time::Instant;

use asten_diagnostics::{
    Diagnostic, Location, Pairing, Repair, Severity, Statement, StatementKind, Suppression, Tier,
    Violation, apply_baseline, apply_suppressions, catalog, diff_marks, finish,
};
use asten_ir::lattice::{ConceptLattice, color_lattice, lifecycle_lattice, material_lattice};
use asten_ir::{
    Anchor, BlockId, Cardinality, DocId, Entity, Event, Fact, Invariant, Mutability, PredicateId,
    PredicateSig, RelationProps, Route, Scene, Settings, Sort, Status, StoryIr, TemporalConstraint,
    ValueDomain,
};
use asten_ls::{DiagnosticsDelta, DiagnosticsSnapshot, LanguageTier, Obligation, Timings};
use asten_perceive::{BlockPerception, Lexicon, Matcher, cache_key, perceive_block};
use serde::{Deserialize, Serialize};

pub mod queries;

pub use queries::*;

pub const ENGINE_VERSION: &str = env!("CARGO_PKG_VERSION");

/// One block of prose with its stable id and content fingerprint.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct BlockInput {
    pub document: DocId,
    pub block: BlockId,
    pub text: String,
    pub fingerprint: String,
    pub reading_pos: i64,
}

/// A craft hint from a method pack: a word to notice, never a canon error.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct CraftHint {
    pub code: String,
    pub word: String,
    pub message: String,
    pub pack: String,
}

/// Everything the engine reads at one revision.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct ProjectInputs {
    pub revision: String,
    pub scenes: Vec<Scene>,
    pub blocks: Vec<BlockInput>,
    pub entities: Vec<Entity>,
    pub predicates: Vec<PredicateSig>,
    #[serde(default)]
    pub lattices: BTreeMap<String, ConceptLattice>,
    pub facts: Vec<Fact>,
    pub events: Vec<Event>,
    #[serde(default)]
    pub constraints: Vec<TemporalConstraint>,
    #[serde(default)]
    pub invariants: Vec<Invariant>,
    #[serde(default)]
    pub routes: Vec<Route>,
    pub settings: Settings,
    #[serde(default)]
    pub suppressions: Vec<Suppression>,
    #[serde(default)]
    pub baseline: Vec<String>,
    /// Packs whose rules run. "scribe" is always on.
    #[serde(default)]
    pub enabled_packs: Vec<String>,
    #[serde(default)]
    pub craft_hints: Vec<CraftHint>,
}

impl ProjectInputs {
    /// Inputs with the core predicates and lattices filled in.
    #[must_use]
    pub fn with_core(mut self) -> Self {
        let mut predicates = core_predicates();
        predicates.retain(|core| !self.predicates.iter().any(|sig| sig.id == core.id));
        self.predicates.extend(predicates);
        for lattice in [color_lattice(), lifecycle_lattice(), material_lattice()] {
            self.lattices.entry(lattice.name.clone()).or_insert(lattice);
        }
        if self.enabled_packs.is_empty() {
            self.enabled_packs = default_packs();
        }
        self
    }
}

/// Packs enabled when a project says nothing.
#[must_use]
pub fn default_packs() -> Vec<String> {
    [
        "scribe",
        "lifecycle",
        "clocks",
        "relations",
        "knowledge",
        "atlas",
        "invariants",
    ]
    .iter()
    .map(|pack| (*pack).to_owned())
    .collect()
}

/// The predicate signatures every project starts with.
#[must_use]
pub fn core_predicates() -> Vec<PredicateSig> {
    fn concept(
        id: &str,
        label: &str,
        lattice: &str,
        sorts: &[Sort],
        cues: &[&str],
    ) -> PredicateSig {
        PredicateSig {
            id: PredicateId::new(id),
            label: label.into(),
            subject_sorts: sorts.to_vec(),
            domain: ValueDomain::Concept {
                lattice: lattice.into(),
            },
            cardinality: Cardinality::Functional,
            mutability: Mutability::Mutable,
            relation: RelationProps::default(),
            cues: cues.iter().map(|cue| (*cue).to_owned()).collect(),
            pack: "scribe".into(),
        }
    }
    fn relation(
        id: &str,
        label: &str,
        props: RelationProps,
        cardinality: Cardinality,
    ) -> PredicateSig {
        PredicateSig {
            id: PredicateId::new(id),
            label: label.into(),
            subject_sorts: vec![Sort::Character],
            domain: ValueDomain::Entity {
                sort: Some(Sort::Character),
            },
            cardinality,
            mutability: Mutability::Mutable,
            relation: props,
            cues: Vec::new(),
            pack: "relations".into(),
        }
    }
    vec![
        concept(
            "hair_color",
            "hair",
            "color",
            &[Sort::Character],
            &[
                "hair",
                "locks",
                "braid",
                "braids",
                "curls",
                "mane",
                "চুল",
                "cabello",
                "pelo",
                "cheveux",
                "haar",
                "髪",
                "头发",
                "شعر",
                "बाल",
            ],
        ),
        concept(
            "eye_color",
            "eyes",
            "color",
            &[Sort::Character],
            &[
                "eye",
                "eyes",
                "gaze",
                "irises",
                "চোখ",
                "ojos",
                "yeux",
                "augen",
                "目",
                "眼睛",
                "عيون",
                "आँखें",
            ],
        ),
        concept("lifecycle", "status", "lifecycle", &[], &[]),
        concept(
            "material",
            "material",
            "material",
            &[Sort::Item, Sort::Place],
            &["made", "forged", "carved", "built"],
        ),
        PredicateSig {
            id: PredicateId::new("location"),
            label: "location".into(),
            subject_sorts: vec![Sort::Character, Sort::Item, Sort::Group],
            domain: ValueDomain::Entity {
                sort: Some(Sort::Place),
            },
            cardinality: Cardinality::Functional,
            mutability: Mutability::Mutable,
            relation: RelationProps::default(),
            cues: Vec::new(),
            pack: "atlas".into(),
        },
        PredicateSig {
            id: PredicateId::new("birthplace"),
            label: "birthplace".into(),
            subject_sorts: vec![Sort::Character],
            domain: ValueDomain::Entity {
                sort: Some(Sort::Place),
            },
            cardinality: Cardinality::Functional,
            mutability: Mutability::Immutable,
            relation: RelationProps::default(),
            cues: Vec::new(),
            pack: "scribe".into(),
        },
        PredicateSig {
            id: PredicateId::new("age"),
            label: "age".into(),
            subject_sorts: vec![Sort::Character],
            domain: ValueDomain::Quantity {
                dimension: asten_ir::Dimension::Age,
            },
            cardinality: Cardinality::Functional,
            mutability: Mutability::Mutable,
            relation: RelationProps::default(),
            cues: vec!["years old".into(), "aged".into()],
            pack: "scribe".into(),
        },
        PredicateSig {
            id: PredicateId::new("height"),
            label: "height".into(),
            subject_sorts: vec![Sort::Character],
            domain: ValueDomain::Quantity {
                dimension: asten_ir::Dimension::Length,
            },
            cardinality: Cardinality::Functional,
            mutability: Mutability::Mutable,
            relation: RelationProps::default(),
            cues: vec!["tall".into(), "height".into()],
            pack: "scribe".into(),
        },
        relation(
            "parent_of",
            "parent",
            RelationProps {
                acyclic: true,
                inverse: Some(PredicateId::new("child_of")),
                ..RelationProps::default()
            },
            Cardinality::Multi,
        ),
        relation(
            "child_of",
            "child",
            RelationProps {
                acyclic: true,
                inverse: Some(PredicateId::new("parent_of")),
                ..RelationProps::default()
            },
            Cardinality::Multi,
        ),
        relation(
            "married_to",
            "spouse",
            RelationProps {
                symmetric: true,
                exclusive: true,
                ..RelationProps::default()
            },
            Cardinality::Multi,
        ),
        relation(
            "sibling_of",
            "sibling",
            RelationProps {
                symmetric: true,
                ..RelationProps::default()
            },
            Cardinality::Multi,
        ),
        relation(
            "ally_of",
            "ally",
            RelationProps {
                symmetric: true,
                ..RelationProps::default()
            },
            Cardinality::Multi,
        ),
        relation(
            "enemy_of",
            "enemy",
            RelationProps {
                symmetric: true,
                ..RelationProps::default()
            },
            Cardinality::Multi,
        ),
    ]
}

/// The result of one analysis pass.
#[derive(Clone, Debug)]
pub struct Outcome {
    pub snapshot: DiagnosticsSnapshot,
    /// Shared with the engine's `last_ir`, so a pass does not copy the IR.
    pub ir: Arc<StoryIr>,
    pub ledger: Vec<asten_logic::invariants::Obligation>,
    pub delta: Option<DiagnosticsDelta>,
}

/// The engine: a perception cache and the previous revision's marks.
#[derive(Default)]
pub struct Engine {
    cache: HashMap<String, BlockPerception>,
    previous: Option<(String, Vec<Diagnostic>)>,
    last_ir: Option<Arc<StoryIr>>,
    last_perceptions: Vec<BlockPerception>,
    /// The lexicon fingerprint of the last pass and what it was computed
    /// from, so an unchanged lexicon is not serialised again.
    lexicon: Option<LexiconMemo>,
    pub max_cache_entries: usize,
}

struct LexiconMemo {
    predicates: Vec<PredicateSig>,
    lattices: BTreeMap<String, ConceptLattice>,
    fingerprint: String,
}

impl Engine {
    #[must_use]
    pub fn new() -> Self {
        Self {
            max_cache_entries: 50_000,
            ..Self::default()
        }
    }

    /// The IR of the last analysis, for queries.
    #[must_use]
    pub fn last_ir(&self) -> Option<&StoryIr> {
        self.last_ir.as_deref()
    }

    #[must_use]
    pub fn last_perceptions(&self) -> &[BlockPerception] {
        &self.last_perceptions
    }

    #[must_use]
    pub fn last_diagnostics(&self) -> &[Diagnostic] {
        self.previous
            .as_ref()
            .map(|(_, diagnostics)| diagnostics.as_slice())
            .unwrap_or(&[])
    }

    /// Forget the previous revision, so the next analysis has no delta.
    pub fn reset_delta(&mut self) {
        self.previous = None;
    }

    /// Analyse a project revision.
    pub fn analyze(&mut self, inputs: &ProjectInputs) -> Outcome {
        self.analyze_owned(inputs.clone())
    }

    /// Analyse a project revision, taking the inputs by value. A caller that
    /// built the inputs for this pass alone saves the copy of every block's
    /// text that `analyze` makes.
    pub fn analyze_owned(&mut self, inputs: ProjectInputs) -> Outcome {
        let started = Instant::now();
        let inputs = inputs.with_core();
        let (perceptions, cached) = self.perceive(&inputs);
        let perceive_ms = started.elapsed().as_millis() as u64;

        let ir = build_ir(&inputs, &perceptions);
        let report = asten_logic::analyze(&ir);
        let mut diagnostics: Vec<Diagnostic> = report
            .violations
            .iter()
            .filter(|violation| pack_enabled(&inputs, &violation.code))
            .cloned()
            .filter_map(|violation| finish(violation, inputs.settings.strictness, &inputs.revision))
            .collect();
        diagnostics.extend(craft_hints(&inputs));
        let suppressed = apply_suppressions(diagnostics, &inputs.suppressions);
        let mut diagnostics = suppressed.diagnostics;
        apply_baseline(&mut diagnostics, &inputs.baseline);
        diagnostics.sort_by(|a, b| {
            a.severity
                .cmp(&b.severity)
                .then_with(|| {
                    b.probability
                        .partial_cmp(&a.probability)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .then_with(|| a.id.cmp(&b.id))
        });
        let analyze_ms = started.elapsed().as_millis() as u64 - perceive_ms;

        let delta = self
            .previous
            .as_ref()
            .map(|(from, previous)| DiagnosticsDelta {
                from_revision: from.clone(),
                revision: inputs.revision.clone(),
                diff: diff_marks(previous, &diagnostics),
            });
        self.previous = Some((inputs.revision.clone(), diagnostics.clone()));

        let tiers = language_tiers(&perceptions);
        let ledger: Vec<Obligation> = report
            .ledger
            .iter()
            .map(|obligation| Obligation {
                invariant: obligation.invariant.clone(),
                label: obligation.label.clone(),
                since: obligation.since.clone(),
                since_title: obligation.since_title.clone(),
            })
            .collect();
        let snapshot = DiagnosticsSnapshot {
            revision: inputs.revision.clone(),
            diagnostics,
            ledger,
            tiers,
            stale_suppressions: suppressed.stale,
            timings_ms: Timings {
                perceive: perceive_ms,
                analyze: analyze_ms,
                total: started.elapsed().as_millis() as u64,
            },
            cached_blocks: cached,
            total_blocks: inputs.blocks.len(),
        };
        let ir = Arc::new(ir);
        self.last_ir = Some(ir.clone());
        self.last_perceptions = perceptions;
        Outcome {
            snapshot,
            ir,
            ledger: report.ledger,
            delta,
        }
    }

    /// The lexicon fingerprint, serialised only when predicates or lattices
    /// differ from the last pass.
    fn lexicon_fingerprint(&mut self, inputs: &ProjectInputs) -> String {
        if let Some(memo) = &self.lexicon {
            if memo.predicates == inputs.predicates && memo.lattices == inputs.lattices {
                return memo.fingerprint.clone();
            }
        }
        let fingerprint = lexicon_fingerprint(inputs);
        self.lexicon = Some(LexiconMemo {
            predicates: inputs.predicates.clone(),
            lattices: inputs.lattices.clone(),
            fingerprint: fingerprint.clone(),
        });
        fingerprint
    }

    /// Perceive every block, from the cache where the key matches.
    fn perceive(&mut self, inputs: &ProjectInputs) -> (Vec<BlockPerception>, usize) {
        let matcher = Matcher::new(&inputs.entities);
        let lexicon = Lexicon::new(&inputs.lattices, &inputs.predicates);
        let lexicon_fingerprint = self.lexicon_fingerprint(inputs);
        let scene_context: HashMap<&DocId, &asten_ir::Context> = inputs
            .scenes
            .iter()
            .map(|scene| (&scene.document, &scene.context))
            .collect();
        let mut out = Vec::with_capacity(inputs.blocks.len());
        let mut cached = 0;
        let mut touched: Vec<String> = Vec::with_capacity(inputs.blocks.len());
        for block in &inputs.blocks {
            let context = scene_context
                .get(&block.document)
                .copied()
                .cloned()
                .unwrap_or_default();
            let key = cache_key(
                &block.fingerprint,
                &matcher.cast_fingerprint,
                &lexicon_fingerprint,
                &context,
            );
            let key = format!(
                "{key}|{}|{}|{}",
                block.document.0, block.block.0, block.reading_pos
            );
            if let Some(perception) = self.cache.get(&key) {
                cached += 1;
                out.push(perception.clone());
                touched.push(key);
                continue;
            }
            let perception = perceive_block(
                &block.text,
                &block.fingerprint,
                &block.document.0,
                &block.block.0,
                block.reading_pos,
                &context,
                &matcher,
                &lexicon,
                &inputs.entities,
                &inputs.predicates,
                &inputs.lattices,
            );
            self.cache.insert(key.clone(), perception.clone());
            touched.push(key);
            out.push(perception);
        }
        if self.cache.len() > self.max_cache_entries {
            // Keep only what this revision used.
            let keep: std::collections::HashSet<String> = touched.into_iter().collect();
            self.cache.retain(|key, _| keep.contains(key));
        }
        (out, cached)
    }
}

fn lexicon_fingerprint(inputs: &ProjectInputs) -> String {
    let predicates = serde_json::to_string(&inputs.predicates).unwrap_or_default();
    let lattices = serde_json::to_string(&inputs.lattices).unwrap_or_default();
    asten_ir::sha256_hex(format!("{predicates}\u{1f}{lattices}").as_bytes())
}

fn pack_enabled(inputs: &ProjectInputs, code: &str) -> bool {
    let Some(info) = catalog::info(code) else {
        return true;
    };
    info.pack == "scribe" || inputs.enabled_packs.iter().any(|pack| pack == info.pack)
}

/// Assemble the IR from inputs and per-block perception.
#[must_use]
pub fn build_ir(inputs: &ProjectInputs, perceptions: &[BlockPerception]) -> StoryIr {
    let mut ir = StoryIr {
        entities: inputs.entities.clone(),
        predicates: inputs.predicates.clone(),
        scenes: inputs.scenes.clone(),
        facts: inputs.facts.clone(),
        observations: Vec::new(),
        events: inputs.events.clone(),
        mentions: Vec::new(),
        constraints: inputs.constraints.clone(),
        invariants: inputs.invariants.clone(),
        routes: inputs.routes.clone(),
        unknown_names: Vec::new(),
        settings: inputs.settings.clone(),
        lattices: inputs.lattices.clone(),
    };
    let mut unknown: Vec<(Anchor, String, i64)> = Vec::new();
    for perception in perceptions {
        ir.mentions.extend(perception.mentions.iter().cloned());
        ir.observations
            .extend(perception.observations.iter().cloned());
        unknown.extend(perception.unknown_names.iter().cloned());
    }
    // Unknown names are shown when they recur or are multi-word, to keep the
    // single capitalised common noun from nagging.
    let mut counts: HashMap<String, usize> = HashMap::new();
    for (_, name, _) in &unknown {
        *counts.entry(name.to_lowercase()).or_default() += 1;
    }
    ir.unknown_names = unknown
        .into_iter()
        .filter(|(_, name, _)| {
            name.contains(' ') || counts.get(&name.to_lowercase()).copied().unwrap_or(0) >= 2
        })
        .collect();
    // Scenes know their blocks: grouped once, not one scan per scene.
    let mut blocks_by_document: HashMap<&DocId, Vec<BlockId>> = HashMap::new();
    for block in &inputs.blocks {
        blocks_by_document
            .entry(&block.document)
            .or_default()
            .push(block.block.clone());
    }
    for scene in &mut ir.scenes {
        scene.blocks = blocks_by_document
            .get(&scene.document)
            .cloned()
            .unwrap_or_default();
    }
    ir
}

fn language_tiers(perceptions: &[BlockPerception]) -> Vec<LanguageTier> {
    let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
    for perception in perceptions {
        if perception.script == "Zyyy" {
            continue;
        }
        *counts.entry(perception.script.as_str()).or_default() += 1;
    }
    counts
        .into_iter()
        .map(|(script, blocks)| LanguageTier {
            script: script.to_owned(),
            tier: format!("{:?}", asten_perceive::tier_for(script)),
            blocks,
        })
        .collect()
}

/// Craft hints from packs: observations, never canon errors. Each block is
/// folded once and searched for every enabled hint; the final order of the
/// marks is settled by the sort in `analyze`, not by this loop.
fn craft_hints(inputs: &ProjectInputs) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    let hints: Vec<(&CraftHint, Vec<char>)> = inputs
        .craft_hints
        .iter()
        .filter(|hint| inputs.enabled_packs.contains(&hint.pack))
        .filter_map(|hint| {
            let needle = asten_perceive::matching_form(&hint.word);
            (!needle.is_empty()).then(|| (hint, needle.chars().collect()))
        })
        .collect();
    if hints.is_empty() {
        return out;
    }
    for block in &inputs.blocks {
        let folded = asten_perceive::Folded::new(&block.text);
        let chars = &folded.chars;
        for (hint, needle_chars) in &hints {
            if chars.len() < needle_chars.len() {
                continue;
            }
            for start in 0..=chars.len() - needle_chars.len() {
                if chars[start..start + needle_chars.len()] != needle_chars[..] {
                    continue;
                }
                let end = start + needle_chars.len();
                let before_ok = start == 0 || !chars[start - 1].is_alphanumeric();
                let after_ok = end >= chars.len() || !chars[end].is_alphanumeric();
                if !before_ok || !after_ok {
                    continue;
                }
                let s = folded.origin[start] as u32;
                let e = folded.origin[end - 1] as u32 + 1;
                let anchor = Anchor::new(block.document.clone(), block.block.clone(), s, e)
                    .with_text(&block.text);
                let violation = Violation {
                    code: hint.code.clone(),
                    title: hint.message.clone(),
                    primary: Some(Location::from_anchor(&anchor)),
                    related: Vec::new(),
                    conflict: vec![
                        Statement::new(
                            StatementKind::Rule,
                            format!("hint:{}:{}:{}", hint.code, block.block.0, s),
                            format!("\u{201c}{}\u{201d}: {}", anchor.quote, hint.message),
                            Status::Settled,
                        )
                        .at(Some(Location::from_anchor(&anchor)))
                        .with_prior(0.5),
                    ],
                    tier: Tier::May,
                    pairing: Pairing::SettledSettled,
                    confidence: 1.0,
                    repairs: vec![Repair::suppress(), Repair::keep()],
                    entity: None,
                    details: format!("Craft hint from the {} pack. Not a canon error.", hint.pack),
                    severity_override: Some(Severity::Hint),
                };
                if let Some(diagnostic) =
                    finish(violation, inputs.settings.strictness, &inputs.revision)
                {
                    out.push(diagnostic);
                }
                break; // one hint per word per block
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use asten_ir::{
        Alias, ConceptId, Context, EntityId, FactId, Fluent, TimeBounds, TimeRef, Value,
    };

    fn scene(document: &str, title: &str, pos: i64, tick: i64) -> Scene {
        Scene {
            document: DocId::new(document),
            title: title.into(),
            reading_pos: pos,
            at: TimeRef::Tick { tick },
            time_label: format!("day {tick}"),
            context: Context::Narration,
            continuity_run: None,
            language: None,
            blocks: Vec::new(),
        }
    }

    fn block(document: &str, id: &str, pos: i64, text: &str) -> BlockInput {
        BlockInput {
            document: DocId::new(document),
            block: BlockId::new(id),
            text: text.into(),
            fingerprint: asten_ir::sha256_hex(text.as_bytes()),
            reading_pos: pos,
        }
    }

    fn inputs() -> ProjectInputs {
        ProjectInputs {
            revision: "r1".into(),
            scenes: vec![
                scene("ch2", "Chapter 2", 2, 3),
                scene("ch19", "Chapter 19", 19, 22),
            ],
            blocks: vec![
                block(
                    "ch2",
                    "b1",
                    2,
                    "Mira tied back her auburn hair and waited for Oren.",
                ),
                block(
                    "ch19",
                    "b2",
                    19,
                    "Mira pushed her black hair out of her eyes. Lord Aric watched. Lord Aric said nothing.",
                ),
            ],
            entities: vec![
                Entity {
                    id: EntityId::new("mira"),
                    sort: Sort::Character,
                    name: "Mira".into(),
                    aliases: Vec::new(),
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
            ],
            facts: vec![Fact {
                id: FactId::new("f1"),
                fluent: Fluent {
                    predicate: PredicateId::new("hair_color"),
                    subject: EntityId::new("mira"),
                    value: Value::Concept {
                        id: ConceptId::new("auburn"),
                        label: "auburn".into(),
                    },
                },
                context: Context::World,
                valid: TimeBounds::from(3),
                status: Status::Settled,
                provenance: asten_ir::Provenance::Author {
                    action: "settle".into(),
                },
                confidence: 1.0,
                anchor: Some(
                    Anchor::new("ch2", "b1", 19, 25)
                        .with_text("Mira tied back her auburn hair and waited for Oren."),
                ),
                settled_at: None,
            }],
            ..ProjectInputs::default()
        }
    }

    #[test]
    fn end_to_end_mark_with_explanation_and_cache() {
        let mut engine = Engine::new();
        let outcome = engine.analyze(&inputs());
        let marks = &outcome.snapshot.diagnostics;
        let e013 = marks
            .iter()
            .find(|mark| mark.code == "CANON-E013")
            .expect("an attribute mark");
        assert_eq!(e013.severity, Severity::Warning);
        assert_eq!(e013.tier, Tier::Must);
        assert_eq!(e013.primary.as_ref().unwrap().document, DocId::new("ch19"));
        assert_eq!(e013.primary.as_ref().unwrap().quote, "black");
        assert!(e013.message.contains("Chapter 2"));
        assert!(
            e013.repairs
                .iter()
                .any(|repair| repair.label == "Add an event")
        );
        assert!(
            marks
                .iter()
                .any(|mark| mark.code == "NAME-W001" && mark.title.contains("Lord Aric"))
        );
        assert_eq!(outcome.snapshot.cached_blocks, 0);
        assert_eq!(outcome.snapshot.tiers[0].script, "Latn");

        // Same inputs again: everything cached, no delta changes.
        let mut again = inputs();
        again.revision = "r2".into();
        let outcome = engine.analyze(&again);
        assert_eq!(outcome.snapshot.cached_blocks, 2);
        let delta = outcome.delta.unwrap();
        assert!(delta.diff.added.is_empty() && delta.diff.removed.is_empty());

        // Edit the second block to agree with canon: the mark goes away, one block re-perceived.
        let mut fixed = inputs();
        fixed.revision = "r3".into();
        fixed.blocks[1] = block(
            "ch19",
            "b2",
            19,
            "Mira pushed her auburn hair out of her eyes.",
        );
        let outcome = engine.analyze(&fixed);
        assert_eq!(outcome.snapshot.cached_blocks, 1);
        assert!(
            outcome
                .snapshot
                .diagnostics
                .iter()
                .all(|mark| mark.code != "CANON-E013")
        );
        assert!(outcome.delta.unwrap().diff.removed.contains(&e013.id));
    }

    #[test]
    fn strictness_quiet_hides_observed_conflicts() {
        let mut engine = Engine::new();
        let mut quiet = inputs();
        quiet.settings.strictness = asten_ir::Strictness::Quiet;
        let outcome = engine.analyze(&quiet);
        assert!(
            outcome
                .snapshot
                .diagnostics
                .iter()
                .all(|mark| mark.code != "CANON-E013")
        );
    }

    /// 250k words: 50 chapters of 50 blocks, each block ~100 words.
    fn large_fixture() -> ProjectInputs {
        let mut big = inputs();
        big.blocks.clear();
        big.scenes.clear();
        let sentence = "Mira walked along the harbor wall while Oren counted the boats and the gulls argued over the nets. ";
        for chapter in 0..50 {
            let document = format!("c{chapter}");
            big.scenes.push(scene(
                &document,
                &format!("Chapter {chapter}"),
                chapter,
                chapter * 2,
            ));
            for index in 0..50 {
                let text = sentence.repeat(5);
                big.blocks.push(block(
                    &document,
                    &format!("{document}-{index}"),
                    chapter,
                    &text,
                ));
            }
        }
        big
    }

    /// `cargo test -p asten-engine -- --ignored --nocapture bench` prints the
    /// perceive and analyze timings for a cold pass and a one-paragraph edit.
    #[test]
    #[ignore = "benchmark; run by hand with --nocapture"]
    fn bench_large_project() {
        let mut big = large_fixture();
        let mut engine = Engine::new();
        let cold = engine.analyze(&big);
        println!(
            "cold: perceive {} ms, analyze {} ms, total {} ms ({} blocks, {} marks)",
            cold.snapshot.timings_ms.perceive,
            cold.snapshot.timings_ms.analyze,
            cold.snapshot.timings_ms.total,
            cold.snapshot.total_blocks,
            cold.snapshot.diagnostics.len()
        );
        for pass in 0..3 {
            big.revision = format!("edit{pass}");
            big.blocks[1234].text.push_str(" Mira smiled.");
            big.blocks[1234].fingerprint = asten_ir::sha256_hex(big.blocks[1234].text.as_bytes());
            let warm = engine.analyze(&big);
            println!(
                "edit {pass}: perceive {} ms, analyze {} ms, total {} ms ({} cached)",
                warm.snapshot.timings_ms.perceive,
                warm.snapshot.timings_ms.analyze,
                warm.snapshot.timings_ms.total,
                warm.snapshot.cached_blocks
            );
        }
    }

    #[test]
    fn large_project_is_fast() {
        let mut big = large_fixture();
        let mut engine = Engine::new();
        let first = engine.analyze(&big);
        assert_eq!(first.snapshot.total_blocks, 2500);
        // One paragraph edit: only one block re-perceived, and quickly.
        big.revision = "r2".into();
        big.blocks[1234].text.push_str(" Mira smiled.");
        big.blocks[1234].fingerprint = asten_ir::sha256_hex(big.blocks[1234].text.as_bytes());
        let second = engine.analyze(&big);
        assert_eq!(second.snapshot.cached_blocks, 2499);
        assert!(
            // Generous for a debug build on a shared CI runner; locally this is
            // well under a second.
            second.snapshot.timings_ms.total < 8_000,
            "{:?}",
            second.snapshot.timings_ms
        );
    }
}
