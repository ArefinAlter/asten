//! Diagnostics: the records the desk shows, the codes that never change, the
//! severity table, explanations as minimal conflict sets, ranked culprits,
//! abductive repairs, and suppressions that know when to come back.
//! See `docs/ENGINE_DESIGN.md` §12.

use std::collections::BTreeMap;

use asten_ir::{
    Anchor, BlockId, Context, DocId, EntityId, Provenance, Status, Strictness,
    evidence_fingerprint, sha256_hex,
};
use serde::{Deserialize, Serialize};

pub mod catalog;

pub use catalog::{Area, CodeInfo, catalog, info};

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[cfg_attr(
    feature = "ts",
    derive(ts_rs::TS),
    ts(export, export_to = "diagnostics/")
)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Error,
    Warning,
    Information,
    Hint,
}

/// Whether the conflict is forced by the constraints or merely possible.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[cfg_attr(
    feature = "ts",
    derive(ts_rs::TS),
    ts(export, export_to = "diagnostics/")
)]
#[serde(rename_all = "snake_case")]
pub enum Tier {
    Must,
    #[default]
    May,
}

/// Which kinds of statements disagree. Drives severity (§12.2).
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[cfg_attr(
    feature = "ts",
    derive(ts_rs::TS),
    ts(export, export_to = "diagnostics/")
)]
#[serde(rename_all = "snake_case")]
pub enum Pairing {
    SettledSettled,
    SettledObserved,
    ProposedProposed,
}

/// Where a mark lands. Block-relative when the statement has an anchor; the
/// whole document otherwise.
#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[cfg_attr(
    feature = "ts",
    derive(ts_rs::TS),
    ts(export, export_to = "diagnostics/")
)]
pub struct Location {
    pub document: DocId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub block: Option<BlockId>,
    pub start: u32,
    pub end: u32,
    /// What the span says, for display.
    #[serde(default)]
    pub quote: String,
    /// A human label for the place ("chapter 2").
    #[serde(default)]
    pub label: String,
}

impl Location {
    #[must_use]
    pub fn from_anchor(anchor: &Anchor) -> Self {
        Self {
            document: anchor.document.clone(),
            block: Some(anchor.block.clone()),
            start: anchor.start,
            end: anchor.end,
            quote: anchor.quote.clone(),
            label: String::new(),
        }
    }

    #[must_use]
    pub fn document(document: DocId, label: impl Into<String>) -> Self {
        Self {
            document,
            label: label.into(),
            ..Self::default()
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[cfg_attr(
    feature = "ts",
    derive(ts_rs::TS),
    ts(export, export_to = "diagnostics/")
)]
#[serde(rename_all = "snake_case")]
pub enum StatementKind {
    Fact,
    Observation,
    Event,
    Scene,
    Constraint,
    Mention,
    Invariant,
    Rule,
}

/// One statement in a minimal conflict set.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[cfg_attr(
    feature = "ts",
    derive(ts_rs::TS),
    ts(export, export_to = "diagnostics/")
)]
pub struct Statement {
    pub kind: StatementKind,
    /// The id of the fact, observation, event, scene or constraint.
    pub id: String,
    /// Rendered for the explanation chain.
    pub text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<Location>,
    pub status: Status,
    /// Prior probability that this statement is the one at fault (§10.6).
    pub fault_prior: f32,
    #[serde(default = "one")]
    pub confidence: f32,
}

fn one() -> f32 {
    1.0
}

impl Statement {
    #[must_use]
    pub fn new(
        kind: StatementKind,
        id: impl Into<String>,
        text: impl Into<String>,
        status: Status,
    ) -> Self {
        let fault_prior = match status {
            Status::Settled => 0.01,
            Status::Proposed => 0.15,
            Status::Deprecated | Status::Rejected => 0.5,
        };
        Self {
            kind,
            id: id.into(),
            text: text.into(),
            location: None,
            status,
            fault_prior,
            confidence: 1.0,
        }
    }

    #[must_use]
    pub fn at(mut self, location: Option<Location>) -> Self {
        self.location = location;
        self
    }

    #[must_use]
    pub fn with_prior(mut self, prior: f32) -> Self {
        self.fault_prior = prior.clamp(0.001, 0.999);
        self
    }

    #[must_use]
    pub fn with_confidence(mut self, confidence: f32) -> Self {
        self.confidence = confidence.clamp(0.0, 1.0);
        self
    }

    /// Prior from provenance, status and context.
    #[must_use]
    pub fn prior_from(
        provenance: &Provenance,
        status: Status,
        context: &Context,
        confidence: f32,
    ) -> f32 {
        provenance.fault_prior(status, context, confidence)
    }
}

/// A repair the author may apply. Repairs change canon, contexts or metadata,
/// never prose (§10.7).
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[cfg_attr(
    feature = "ts",
    derive(ts_rs::TS),
    ts(export, export_to = "diagnostics/")
)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum RepairKind {
    /// Record that a fluent changed between two times.
    AddChangeEvent {
        entity: EntityId,
        predicate: String,
        to_value: serde_json::Value,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        after_tick: Option<i64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        before_tick: Option<i64>,
    },
    /// Reclassify a passage's context (dream, memory, a character's claim).
    MarkContext { document: DocId, context: Context },
    /// Bound a fact's validity in story time.
    SetValidity {
        fact: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        from_tick: Option<i64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        to_tick: Option<i64>,
    },
    /// Move a scene in story time.
    PlaceScene {
        document: DocId,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        lo: Option<i64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        hi: Option<i64>,
    },
    /// Move an event in story time.
    MoveEvent {
        event: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        lo: Option<i64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        hi: Option<i64>,
    },
    /// Retire a settled fact.
    DeprecateFact { fact: String },
    /// Settle a proposed statement as canon.
    SettleFact { fact: String },
    /// Add an unknown name to the cast.
    AddEntity { name: String, sort: asten_ir::Sort },
    /// Link an alias to an existing entity.
    AddAlias { entity: EntityId, alias: String },
    /// Declare that a run of scenes is not continuous after all.
    ClearContinuity { document: DocId },
    /// Silence this mark with a reason. Always available, never auto-chosen.
    Suppress,
    /// Keep the sentence as written; nothing changes.
    Keep,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[cfg_attr(
    feature = "ts",
    derive(ts_rs::TS),
    ts(export, export_to = "diagnostics/")
)]
pub struct Repair {
    pub kind: RepairKind,
    /// A verb the author owns: "Add an event", "Mark as a dream", "Keep my sentence".
    pub label: String,
    /// Tropical-semiring cost: lower is more likely to be what the author meant.
    pub cost: f32,
}

impl Repair {
    #[must_use]
    pub fn new(kind: RepairKind, label: impl Into<String>, cost: f32) -> Self {
        Self {
            kind,
            label: label.into(),
            cost,
        }
    }

    #[must_use]
    pub fn keep() -> Self {
        Self::new(RepairKind::Keep, "Keep my sentence", 100.0)
    }

    #[must_use]
    pub fn suppress() -> Self {
        Self::new(RepairKind::Suppress, "Silence this mark", 99.0)
    }
}

/// What an analysis emits: a conflict with its evidence and tier, before
/// severity, calibration, suppressions and baselines are applied.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[cfg_attr(
    feature = "ts",
    derive(ts_rs::TS),
    ts(export, export_to = "diagnostics/")
)]
pub struct Violation {
    pub code: String,
    /// One line: what disagrees. "Mira's hair was settled as auburn."
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub primary: Option<Location>,
    /// Other sides of the conflict with their roles.
    #[serde(default)]
    pub related: Vec<(Location, String)>,
    /// The minimal conflict set, most authoritative statement first.
    pub conflict: Vec<Statement>,
    pub tier: Tier,
    pub pairing: Pairing,
    /// Product of the evidence confidences behind the conflict.
    pub confidence: f32,
    #[serde(default)]
    pub repairs: Vec<Repair>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entity: Option<EntityId>,
    /// A longer note for the details view.
    #[serde(default)]
    pub details: String,
    /// Codes that are notes rather than conflicts (names, the ledger) fix
    /// their severity instead of deriving it from the pairing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub severity_override: Option<Severity>,
}

impl Violation {
    /// The stable evidence fingerprint: statement ids and anchor blocks.
    #[must_use]
    pub fn evidence_fingerprint(&self) -> String {
        let mut parts: Vec<String> = self
            .conflict
            .iter()
            .map(|statement| statement.id.clone())
            .collect();
        for statement in &self.conflict {
            if let Some(block) = statement
                .location
                .as_ref()
                .and_then(|location| location.block.as_ref())
            {
                parts.push(format!("block:{}", block.0));
            }
        }
        evidence_fingerprint(parts.iter().map(String::as_str))
    }

    /// Stable id: code plus evidence identities, never offsets.
    #[must_use]
    pub fn stable_id(&self) -> String {
        sha256_hex(format!("{}|{}", self.code, self.evidence_fingerprint()).as_bytes())[..24]
            .to_owned()
    }
}

/// A ranked single-fault diagnosis.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[cfg_attr(
    feature = "ts",
    derive(ts_rs::TS),
    ts(export, export_to = "diagnostics/")
)]
pub struct Diagnosis {
    pub statement_id: String,
    pub probability: f32,
}

/// Rank which statement in a conflict set is most likely wrong, assuming one
/// fault: `P(Δ = {c}) ∝ p_c ∏_{d≠c} (1 − p_d)`.
#[must_use]
pub fn diagnose(conflict: &[Statement]) -> Vec<Diagnosis> {
    if conflict.is_empty() {
        return Vec::new();
    }
    let weights: Vec<f64> = conflict
        .iter()
        .map(|candidate| {
            conflict
                .iter()
                .map(|other| {
                    let p = f64::from(other.fault_prior.clamp(0.001, 0.999));
                    if std::ptr::eq(candidate, other) {
                        p
                    } else {
                        1.0 - p
                    }
                })
                .product::<f64>()
        })
        .collect();
    let total: f64 = weights.iter().sum::<f64>().max(f64::MIN_POSITIVE);
    let mut diagnoses: Vec<Diagnosis> = conflict
        .iter()
        .zip(weights)
        .map(|(statement, weight)| Diagnosis {
            statement_id: statement.id.clone(),
            probability: (weight / total) as f32,
        })
        .collect();
    diagnoses.sort_by(|a, b| {
        b.probability
            .partial_cmp(&a.probability)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    diagnoses
}

/// Severity from the pairing, the tier and the project strictness. `None`
/// means the mark is not shown at this strictness.
#[must_use]
pub fn severity_for(pairing: Pairing, tier: Tier, strictness: Strictness) -> Option<Severity> {
    match (pairing, tier) {
        (Pairing::SettledSettled, Tier::Must) => Some(Severity::Error),
        (Pairing::SettledSettled, Tier::May) => Some(Severity::Warning),
        (Pairing::SettledObserved, Tier::Must) => match strictness {
            Strictness::Quiet => None,
            Strictness::Standard | Strictness::Thorough => Some(Severity::Warning),
        },
        (Pairing::SettledObserved, Tier::May) => match strictness {
            Strictness::Quiet => None,
            Strictness::Standard | Strictness::Thorough => Some(Severity::Information),
        },
        (Pairing::ProposedProposed, Tier::Must) => match strictness {
            Strictness::Thorough => Some(Severity::Information),
            _ => None,
        },
        (Pairing::ProposedProposed, Tier::May) => None,
    }
}

/// Confidence threshold below which observed conflicts are not shown, per
/// strictness. Calibrated thresholds replace these once models exist (§10.8).
#[must_use]
pub fn confidence_threshold(strictness: Strictness) -> f32 {
    match strictness {
        Strictness::Quiet => 1.0,
        Strictness::Standard => 0.55,
        Strictness::Thorough => 0.25,
    }
}

/// The record the desk shows.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[cfg_attr(
    feature = "ts",
    derive(ts_rs::TS),
    ts(export, export_to = "diagnostics/")
)]
pub struct Diagnostic {
    /// Stable across re-analysis: hash of code and evidence identities.
    pub id: String,
    pub code: String,
    pub severity: Severity,
    pub tier: Tier,
    /// Calibrated probability that this is a real problem.
    pub probability: f32,
    pub title: String,
    /// The explanation chain rendered as text.
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub primary: Option<Location>,
    #[serde(default)]
    pub related: Vec<(Location, String)>,
    pub explanation: Vec<Statement>,
    pub culprit: Vec<Diagnosis>,
    pub repairs: Vec<Repair>,
    pub provenance: DiagnosticProvenance,
    #[serde(default)]
    pub stale: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub suppressed_by: Option<String>,
    #[serde(default)]
    pub new_since_baseline: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entity: Option<EntityId>,
    #[serde(default)]
    pub details: String,
    pub evidence_fingerprint: String,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[cfg_attr(
    feature = "ts",
    derive(ts_rs::TS),
    ts(export, export_to = "diagnostics/")
)]
pub struct DiagnosticProvenance {
    pub rules: Vec<String>,
    #[serde(default)]
    pub packs: Vec<String>,
    /// The project revision the diagnostic was computed at.
    pub revision: String,
    #[serde(default)]
    pub engine_version: String,
}

/// Render the explanation chain in the voice of a careful editor.
#[must_use]
pub fn render_message(violation: &Violation, culprit: &[Diagnosis]) -> String {
    let marks = ['①', '②', '③', '④', '⑤', '⑥', '⑦', '⑧', '⑨'];
    let mut lines = Vec::new();
    for (index, statement) in violation.conflict.iter().enumerate() {
        let mark = marks.get(index).copied().unwrap_or('•');
        let place = statement
            .location
            .as_ref()
            .map(|location| {
                if location.label.is_empty() {
                    String::new()
                } else {
                    format!(" ({})", location.label)
                }
            })
            .unwrap_or_default();
        lines.push(format!("{mark} {}{place}", statement.text));
    }
    if let Some(best) = culprit.first() {
        if violation.conflict.len() > 1 && best.probability > 0.5 {
            if let Some(index) = violation
                .conflict
                .iter()
                .position(|statement| statement.id == best.statement_id)
            {
                let mark = marks.get(index).copied().unwrap_or('•');
                lines.push(format!(
                    "Most likely slip: {mark} ({}%)",
                    (best.probability * 100.0).round() as i32
                ));
            }
        }
    }
    if !violation.repairs.is_empty() {
        let labels: Vec<&str> = violation
            .repairs
            .iter()
            .map(|repair| repair.label.as_str())
            .collect();
        lines.push(format!("Fixes: {}", labels.join(" · ")));
    }
    lines.join("\n")
}

/// Turn a violation into a diagnostic at a strictness, or `None` when the
/// strictness hides it.
#[must_use]
pub fn finish(violation: Violation, strictness: Strictness, revision: &str) -> Option<Diagnostic> {
    let severity = match violation.severity_override {
        Some(severity) => severity,
        None => {
            let severity = severity_for(violation.pairing, violation.tier, strictness)?;
            if violation.pairing != Pairing::SettledSettled
                && violation.confidence < confidence_threshold(strictness)
            {
                return None;
            }
            severity
        }
    };
    let culprit = diagnose(&violation.conflict);
    let mut repairs = violation.repairs.clone();
    repairs.sort_by(|a, b| {
        a.cost
            .partial_cmp(&b.cost)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let probability = match violation.pairing {
        Pairing::SettledSettled => 1.0,
        _ => violation.confidence.clamp(0.0, 1.0),
    };
    let evidence = violation.evidence_fingerprint();
    let id = violation.stable_id();
    let message = render_message(&violation, &culprit);
    Some(Diagnostic {
        id,
        code: violation.code.clone(),
        severity,
        tier: violation.tier,
        probability,
        title: violation.title,
        message,
        primary: violation.primary,
        related: violation.related,
        explanation: violation.conflict,
        culprit,
        repairs,
        provenance: DiagnosticProvenance {
            rules: vec![violation.code],
            packs: Vec::new(),
            revision: revision.to_owned(),
            engine_version: env!("CARGO_PKG_VERSION").to_owned(),
        },
        stale: false,
        suppressed_by: None,
        new_since_baseline: false,
        entity: violation.entity,
        details: violation.details,
        evidence_fingerprint: evidence,
    })
}

// ───────────────────────── suppressions 2.0 ─────────────────────────

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[cfg_attr(
    feature = "ts",
    derive(ts_rs::TS),
    ts(export, export_to = "diagnostics/")
)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum SuppressionScope {
    /// This one mark, bound to its evidence.
    Occurrence { diagnostic_id: String },
    /// This code for this entity.
    Entity { entity: EntityId },
    /// This code in this document.
    Document { document: DocId },
    /// This code everywhere in the project.
    Project,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[cfg_attr(
    feature = "ts",
    derive(ts_rs::TS),
    ts(export, export_to = "diagnostics/")
)]
pub struct Suppression {
    pub id: String,
    pub code: String,
    pub scope: SuppressionScope,
    /// Fingerprint of the evidence when the author silenced it. When the
    /// evidence changes the suppression goes stale and the mark returns.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence_fingerprint: Option<String>,
    #[serde(default)]
    pub reason: String,
}

/// The result of applying suppressions to a set of diagnostics.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Suppressed {
    pub diagnostics: Vec<Diagnostic>,
    /// Suppressions whose evidence no longer exists as it did: the mark came back.
    pub stale: Vec<String>,
}

/// Mark suppressed diagnostics and report stale suppressions. Suppressed marks
/// stay in the list with `suppressed_by` set so the desk can show them on request.
#[must_use]
pub fn apply_suppressions(
    diagnostics: Vec<Diagnostic>,
    suppressions: &[Suppression],
) -> Suppressed {
    let mut stale = Vec::new();
    let mut out = Vec::with_capacity(diagnostics.len());
    let mut matched: BTreeMap<&str, bool> = BTreeMap::new();
    for mut diagnostic in diagnostics {
        for suppression in suppressions {
            if suppression.code != diagnostic.code {
                continue;
            }
            let in_scope = match &suppression.scope {
                SuppressionScope::Occurrence { diagnostic_id } => *diagnostic_id == diagnostic.id,
                SuppressionScope::Entity { entity } => diagnostic.entity.as_ref() == Some(entity),
                SuppressionScope::Document { document } => {
                    diagnostic
                        .primary
                        .as_ref()
                        .is_some_and(|location| &location.document == document)
                        || diagnostic
                            .related
                            .iter()
                            .any(|(location, _)| &location.document == document)
                }
                SuppressionScope::Project => true,
            };
            if !in_scope {
                continue;
            }
            match &suppression.evidence_fingerprint {
                Some(fingerprint) if *fingerprint != diagnostic.evidence_fingerprint => {
                    // Same mark, changed evidence: the suppression is stale and the mark returns.
                    if matches!(suppression.scope, SuppressionScope::Occurrence { .. }) {
                        stale.push(suppression.id.clone());
                    }
                    continue;
                }
                _ => {
                    diagnostic.suppressed_by = Some(suppression.id.clone());
                    matched.insert(suppression.id.as_str(), true);
                    break;
                }
            }
        }
        out.push(diagnostic);
    }
    stale.sort();
    stale.dedup();
    Suppressed {
        diagnostics: out,
        stale,
    }
}

/// Mark diagnostics that did not exist in a baseline set of ids.
pub fn apply_baseline(diagnostics: &mut [Diagnostic], baseline: &[String]) {
    for diagnostic in diagnostics {
        diagnostic.new_since_baseline = !baseline.contains(&diagnostic.id);
    }
}

/// The difference between two diagnostic sets by stable id.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[cfg_attr(
    feature = "ts",
    derive(ts_rs::TS),
    ts(export, export_to = "diagnostics/")
)]
pub struct MarksDiff {
    pub added: Vec<Diagnostic>,
    pub removed: Vec<String>,
    pub changed: Vec<Diagnostic>,
}

#[must_use]
pub fn diff_marks(previous: &[Diagnostic], next: &[Diagnostic]) -> MarksDiff {
    let before: BTreeMap<&str, &Diagnostic> = previous
        .iter()
        .map(|diagnostic| (diagnostic.id.as_str(), diagnostic))
        .collect();
    let after: BTreeMap<&str, &Diagnostic> = next
        .iter()
        .map(|diagnostic| (diagnostic.id.as_str(), diagnostic))
        .collect();
    let mut diff = MarksDiff::default();
    for (id, diagnostic) in &after {
        match before.get(id) {
            None => diff.added.push((*diagnostic).clone()),
            Some(old) if *old != *diagnostic => diff.changed.push((*diagnostic).clone()),
            Some(_) => {}
        }
    }
    for id in before.keys() {
        if !after.contains_key(id) {
            diff.removed.push((*id).to_owned());
        }
    }
    diff
}

#[cfg(test)]
mod tests {
    use super::*;

    fn violation() -> Violation {
        Violation {
            code: "CANON-E013".into(),
            title: "Mira's hair was settled as auburn.".into(),
            primary: Some(Location::document(DocId::new("ch19"), "chapter 19")),
            related: vec![],
            conflict: vec![
                Statement::new(
                    StatementKind::Fact,
                    "fact-1",
                    "Settled in chapter 2: Mira · hair · auburn",
                    Status::Settled,
                ),
                Statement::new(
                    StatementKind::Observation,
                    "obs-1",
                    "Chapter 19 says: \"black hair\"",
                    Status::Proposed,
                )
                .with_prior(0.15),
            ],
            tier: Tier::Must,
            pairing: Pairing::SettledObserved,
            confidence: 0.9,
            repairs: vec![
                Repair::keep(),
                Repair::new(
                    RepairKind::DeprecateFact {
                        fact: "fact-1".into(),
                    },
                    "Deprecate the fact",
                    5.0,
                ),
            ],
            entity: Some(EntityId::new("mira")),
            details: String::new(),
            severity_override: None,
        }
    }

    #[test]
    fn blame_lands_on_the_uncertain_side() {
        let culprit = diagnose(&violation().conflict);
        assert_eq!(culprit[0].statement_id, "obs-1");
        assert!(culprit[0].probability > 0.9);
    }

    #[test]
    fn severity_follows_the_table() {
        assert_eq!(
            severity_for(Pairing::SettledSettled, Tier::Must, Strictness::Quiet),
            Some(Severity::Error)
        );
        assert_eq!(
            severity_for(Pairing::SettledObserved, Tier::Must, Strictness::Quiet),
            None
        );
        assert_eq!(
            severity_for(Pairing::SettledObserved, Tier::Must, Strictness::Standard),
            Some(Severity::Warning)
        );
        assert_eq!(
            severity_for(Pairing::ProposedProposed, Tier::Must, Strictness::Standard),
            None
        );
        assert_eq!(
            severity_for(Pairing::ProposedProposed, Tier::Must, Strictness::Thorough),
            Some(Severity::Information)
        );
    }

    #[test]
    fn finished_diagnostics_have_stable_ids_and_sorted_repairs() {
        let a = finish(violation(), Strictness::Standard, "r1").unwrap();
        let mut moved = violation();
        moved.primary = Some(Location::document(DocId::new("ch19"), "chapter 19, later"));
        let b = finish(moved, Strictness::Standard, "r2").unwrap();
        assert_eq!(a.id, b.id);
        assert_eq!(a.repairs[0].label, "Deprecate the fact");
        assert!(a.message.contains("Most likely slip: ②"));
        assert!(finish(violation(), Strictness::Quiet, "r1").is_none());
    }

    #[test]
    fn suppressions_go_stale_when_evidence_changes() {
        let diagnostic = finish(violation(), Strictness::Standard, "r1").unwrap();
        let suppression = Suppression {
            id: "s1".into(),
            code: "CANON-E013".into(),
            scope: SuppressionScope::Occurrence {
                diagnostic_id: diagnostic.id.clone(),
            },
            evidence_fingerprint: Some(diagnostic.evidence_fingerprint.clone()),
            reason: "intentional".into(),
        };
        let result =
            apply_suppressions(vec![diagnostic.clone()], std::slice::from_ref(&suppression));
        assert_eq!(result.diagnostics[0].suppressed_by.as_deref(), Some("s1"));
        assert!(result.stale.is_empty());

        let mut changed = violation();
        changed.conflict[1].id = "obs-2".into();
        let mut changed = finish(changed, Strictness::Standard, "r2").unwrap();
        // Force the same diagnostic id to simulate the same mark with new evidence.
        changed.id = diagnostic.id.clone();
        let result = apply_suppressions(vec![changed], &[suppression]);
        assert!(result.diagnostics[0].suppressed_by.is_none());
        assert_eq!(result.stale, vec!["s1".to_owned()]);
    }

    #[test]
    fn diffs_marks_by_id() {
        let a = finish(violation(), Strictness::Standard, "r1").unwrap();
        let mut other = violation();
        other.code = "CANON-E101".into();
        let b = finish(other, Strictness::Standard, "r1").unwrap();
        let diff = diff_marks(std::slice::from_ref(&a), std::slice::from_ref(&b));
        assert_eq!(diff.added.len(), 1);
        assert_eq!(diff.removed, vec![a.id]);
    }
}
