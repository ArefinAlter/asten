//! Story state: the event calculus over bounded time (§9.3).
//!
//! A fluent holds from the fact or event that initiates it until an event
//! terminates it. Functional predicates auto-terminate other values. Because
//! story time is usually a partial order, every statement has *must* bounds
//! (holds under every allowed placement) and *may* bounds (holds under some).
//! An observation that disagrees with a value that must hold is a must
//! violation; one that disagrees with a value that may hold is a may violation.

use std::collections::BTreeMap;

use asten_diagnostics::{
    Pairing, Repair, RepairKind, Statement, StatementKind, Tier, Violation, catalog,
};
use asten_ir::{
    Cardinality, Context, Event, EventKind, Fact, Fluent, Mutability, Observation, PredicateId,
    Status, StoryIr, TimeBounds, Value,
};

use crate::time::TimeTable;
use crate::util::{
    event_statement, fact_statement, joint_confidence, location_of, name_of, observation_statement,
    pairing_of,
};

/// A statement that a value holds, from a fact or an initiating event.
#[derive(Clone, Debug)]
pub struct Source<'a> {
    pub value: Value,
    pub status: Status,
    pub confidence: f32,
    /// When it certainly holds and may hold, after clipping.
    pub must: TimeBounds,
    pub may: TimeBounds,
    /// The time of the statement itself: a fact's validity or an event's time.
    pub at: TimeBounds,
    pub statement: Statement,
    pub fact: Option<&'a Fact>,
    pub event: Option<&'a Event>,
}

/// All sources for one fluent family in one comparison context, with their
/// effective must/may intervals after clipping by change events.
#[derive(Clone, Debug, Default)]
pub struct Family<'a> {
    pub sources: Vec<Source<'a>>,
}

/// Values a predicate takes for a subject, grouped by comparison context.
#[must_use]
pub fn families<'a>(ir: &'a StoryIr, times: &TimeTable) -> BTreeMap<(String, Context), Family<'a>> {
    let reliable = ir.settings.narrator_reliable;
    let mut out: BTreeMap<(String, Context), Family<'a>> = BTreeMap::new();

    for fact in ir.facts.iter().filter(|fact| fact.status.is_active()) {
        let key = (fact.fluent.family(), fact.context.comparison_key(reliable));
        out.entry(key).or_default().sources.push(Source {
            value: fact.fluent.value.clone(),
            status: fact.status,
            confidence: fact.confidence,
            must: fact.valid,
            may: fact.valid,
            at: fact.valid,
            statement: fact_statement(ir, fact),
            fact: Some(fact),
            event: None,
        });
    }
    for event in ir.events.iter().filter(|event| event.status.is_active()) {
        let bounds = times.event(&event.id);
        for fluent in initiated(event) {
            let key = (fluent.family(), event.context.comparison_key(reliable));
            // Certainly holds from the event's latest possible time; may hold from its earliest.
            let must = TimeBounds {
                lo: bounds.hi,
                hi: None,
            };
            let may = TimeBounds {
                lo: bounds.lo,
                hi: None,
            };
            out.entry(key).or_default().sources.push(Source {
                value: fluent.value.clone(),
                status: event.status,
                confidence: event.confidence,
                must,
                may,
                at: bounds,
                statement: event_statement(ir, event, &bounds),
                fact: None,
                event: Some(event),
            });
        }
    }

    // Clip each source by change events to a different value in the same family.
    for ((family, _), group) in out.iter_mut() {
        let predicate = family.split('·').nth(1).map(PredicateId::new);
        let functional = predicate
            .as_ref()
            .and_then(|id| ir.predicate(id))
            .is_none_or(|sig| sig.cardinality == Cardinality::Functional);
        if !functional {
            continue;
        }
        let changes: Vec<(TimeBounds, Value)> = group
            .sources
            .iter()
            .filter(|source| source.event.is_some())
            .map(|source| (source.at, source.value.clone()))
            .collect();
        for source in &mut group.sources {
            for (change, value) in &changes {
                if values_equal(value, &source.value) {
                    continue;
                }
                // A change may end this source only if it may happen after the source began.
                let may_follow = match (change.hi, source.may.lo) {
                    (Some(change_hi), Some(start)) => change_hi >= start,
                    _ => true,
                };
                if !may_follow {
                    continue;
                }
                // Certainly ended once the change certainly happened; may have ended once it may have.
                if let Some(change_lo) = change.lo {
                    let must_hi = change_lo - 1;
                    source.must.hi = Some(source.must.hi.map_or(must_hi, |hi| hi.min(must_hi)));
                }
                if let Some(change_hi) = change.hi {
                    let may_hi = change_hi - 1;
                    source.may.hi = Some(source.may.hi.map_or(may_hi, |hi| hi.min(may_hi)));
                }
            }
        }
    }
    out
}

fn values_equal(a: &Value, b: &Value) -> bool {
    a.key() == b.key()
}

/// The fluents an event initiates: declared ones plus those implied by its kind.
#[must_use]
pub fn initiated(event: &Event) -> Vec<Fluent> {
    let mut fluents = event.initiates.clone();
    if let EventKind::Changes { fluent } = &event.kind {
        fluents.push(fluent.clone());
    }
    if let Some(subject) = &event.subject {
        let lifecycle = |value: &str| Fluent {
            predicate: PredicateId::new("lifecycle"),
            subject: subject.clone(),
            value: Value::Concept {
                id: asten_ir::ConceptId::new(value),
                label: value.into(),
            },
        };
        match &event.kind {
            EventKind::Born | EventKind::Resurrected => fluents.push(lifecycle("alive")),
            EventKind::Dies => fluents.push(lifecycle("dead")),
            EventKind::Destroyed => fluents.push(lifecycle("destroyed")),
            EventKind::Repaired => fluents.push(lifecycle("intact")),
            EventKind::Arrives { place }
            | EventKind::Travels {
                to: Some(place), ..
            } => fluents.push(Fluent {
                predicate: PredicateId::new("location"),
                subject: subject.clone(),
                value: Value::Entity { id: place.clone() },
            }),
            _ => {}
        }
    }
    fluents
}

/// Whether a value `must` or `may` hold at `at`, given a family's sources.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Holds {
    Must,
    May,
    No,
}

#[must_use]
pub fn holds_at(
    family: &Family<'_>,
    ir: &StoryIr,
    predicate: &PredicateId,
    value: &Value,
    at: &TimeBounds,
) -> Holds {
    let mut best = Holds::No;
    for source in &family.sources {
        if !ir.compatible(predicate, &source.value, value) {
            continue;
        }
        if source.must.must_contain(at) {
            return Holds::Must;
        }
        if source.may.may_overlap(at) {
            best = Holds::May;
        }
    }
    best
}

/// CANON-E013, E014 and E015.
#[must_use]
pub fn check(ir: &StoryIr, times: &TimeTable) -> Vec<Violation> {
    let mut violations = Vec::new();
    let groups = families(ir, times);

    // E015: sort mismatches.
    for fact in ir.facts.iter().filter(|fact| fact.status.is_active()) {
        let Some(sig) = ir.predicate(&fact.fluent.predicate) else {
            continue;
        };
        if sig.subject_sorts.is_empty() {
            continue;
        }
        let Some(entity) = ir.entity(&fact.fluent.subject) else {
            continue;
        };
        if !sig.subject_sorts.contains(&entity.sort) {
            let statement = fact_statement(ir, fact);
            violations.push(Violation {
                code: catalog::SORT_MISMATCH.into(),
                title: format!(
                    "{} is a {}, and {} does not apply to one.",
                    entity.name,
                    entity.sort.label(),
                    sig.label
                ),
                primary: statement.location.clone(),
                related: Vec::new(),
                conflict: vec![statement],
                tier: Tier::Must,
                pairing: if fact.status == Status::Settled {
                    Pairing::SettledSettled
                } else {
                    Pairing::SettledObserved
                },
                confidence: fact.confidence,
                repairs: vec![
                    Repair::new(
                        RepairKind::DeprecateFact {
                            fact: fact.id.0.clone(),
                        },
                        "Retire this fact",
                        1.0,
                    ),
                    Repair::keep(),
                ],
                entity: Some(entity.id.clone()),
                details: String::new(),
                severity_override: None,
            });
        }
    }

    // E014: a change event on an immutable predicate.
    for event in ir.events.iter().filter(|event| event.status.is_active()) {
        for fluent in initiated(event) {
            let Some(sig) = ir.predicate(&fluent.predicate) else {
                continue;
            };
            if sig.mutability != Mutability::Immutable {
                continue;
            }
            let bounds = times.event(&event.id);
            let statement = event_statement(ir, event, &bounds);
            violations.push(Violation {
                code: catalog::IMMUTABLE_CHANGED.into(),
                title: format!("{} was declared unchanging, and an event changes it.", sig.label),
                primary: statement.location.clone(),
                related: Vec::new(),
                conflict: vec![statement],
                tier: Tier::Must,
                pairing: if event.status == Status::Settled { Pairing::SettledSettled } else { Pairing::SettledObserved },
                confidence: event.confidence,
                repairs: vec![Repair::keep(), Repair::suppress()],
                entity: event.subject.clone(),
                details: "Mark the change as a deliberate retcon by making the predicate mutable in its signature, or remove the event.".into(),
                severity_override: None,
            });
        }
    }

    // E013: incompatible values of a functional predicate in one context and time.
    for ((family_key, context), family) in &groups {
        let Some(predicate_id) = family_key.split('·').nth(1).map(PredicateId::new) else {
            continue;
        };
        let functional = ir
            .predicate(&predicate_id)
            .is_none_or(|sig| sig.cardinality == Cardinality::Functional);
        if !functional {
            continue;
        }
        // Source against source.
        for (index, left) in family.sources.iter().enumerate() {
            for right in family.sources.iter().skip(index + 1) {
                if ir.compatible(&predicate_id, &left.value, &right.value) {
                    continue;
                }
                let tier = if left.must.intersect(&right.must).is_some() {
                    Tier::Must
                } else if left.may.intersect(&right.may).is_some() {
                    Tier::May
                } else {
                    continue;
                };
                violations.push(source_conflict(
                    ir,
                    &predicate_id,
                    context,
                    left,
                    right,
                    tier,
                ));
            }
        }
        // Observations against sources.
        for observation in ir.observations.iter().filter(|observation| {
            observation.fluent.family() == *family_key
                && observation
                    .context
                    .comparison_key(ir.settings.narrator_reliable)
                    == *context
        }) {
            let at = observation_time(observation, times);
            for source in &family.sources {
                if observation.negated {
                    continue;
                }
                if ir.compatible(&predicate_id, &source.value, observation.value()) {
                    continue;
                }
                let tier = if source.must.must_contain(&at) {
                    Tier::Must
                } else if source.may.may_overlap(&at) {
                    Tier::May
                } else {
                    continue;
                };
                violations.push(observation_conflict(
                    ir,
                    &predicate_id,
                    source,
                    observation,
                    &at,
                    tier,
                ));
            }
        }
    }
    violations
}

trait ObservedValue {
    fn value(&self) -> &Value;
}

impl ObservedValue for Observation {
    fn value(&self) -> &Value {
        &self.fluent.value
    }
}

fn observation_time(observation: &Observation, times: &TimeTable) -> TimeBounds {
    match &observation.at {
        asten_ir::TimeRef::Unknown => times.scene(&observation.anchor.document),
        other => times.of(other),
    }
}

fn source_conflict(
    ir: &StoryIr,
    predicate: &PredicateId,
    context: &Context,
    left: &Source<'_>,
    right: &Source<'_>,
    tier: Tier,
) -> Violation {
    let (first, second) =
        if left.statement.status == Status::Settled || right.statement.status != Status::Settled {
            (left, right)
        } else {
            (right, left)
        };
    let subject = first
        .fact
        .map(|fact| fact.fluent.subject.clone())
        .or_else(|| second.fact.map(|fact| fact.fluent.subject.clone()))
        .or_else(|| first.event.and_then(|event| event.subject.clone()));
    let who = subject
        .as_ref()
        .map(|id| name_of(ir, id))
        .unwrap_or_default();
    let label = ir
        .predicate(predicate)
        .map(|sig| sig.label.clone())
        .unwrap_or_else(|| predicate.0.replace('_', " "));
    let conflict = vec![first.statement.clone(), second.statement.clone()];
    let pairing = pairing_of(&conflict);
    let culprit_fact = if second.statement.status != Status::Settled {
        second
    } else {
        first
    };
    let mut repairs = Vec::new();
    if let Some(fact) = culprit_fact.fact {
        repairs.push(Repair::new(
            RepairKind::DeprecateFact {
                fact: fact.id.0.clone(),
            },
            format!("Retire \u{201c}{}\u{201d}", fact.fluent.value.label()),
            if fact.status == Status::Settled {
                6.0
            } else {
                2.0
            },
        ));
        if let Some(other_lo) = other_start(first, second) {
            repairs.push(Repair::new(
                RepairKind::SetValidity {
                    fact: fact.id.0.clone(),
                    from_tick: None,
                    to_tick: Some(other_lo - 1),
                },
                format!(
                    "{} holds until {}",
                    fact.fluent.value.label(),
                    crate::util::when(ir, &TimeBounds::at(other_lo - 1))
                ),
                1.5,
            ));
        }
    }
    if let Some(subject) = &subject {
        repairs.push(Repair::new(
            RepairKind::AddChangeEvent {
                entity: subject.clone(),
                predicate: predicate.0.clone(),
                to_value: serde_json::to_value(&second.value).unwrap_or_default(),
                after_tick: first.may.lo,
                before_tick: second.may.lo,
            },
            format!("Add an event: {who}'s {label} changes"),
            1.0,
        ));
    }
    repairs.push(Repair::suppress());
    repairs.push(Repair::keep());
    let context_note = if *context == Context::World {
        String::new()
    } else {
        format!(" ({})", context.label())
    };
    Violation {
        code: catalog::ATTRIBUTE_CONTRADICTION.into(),
        title: format!("{who}'s {label} is both {} and {}{context_note}.", first.value.label(), second.value.label()),
        primary: second.statement.location.clone().or_else(|| first.statement.location.clone()),
        related: first.statement.location.clone().filter(|_| second.statement.location.is_some()).map(|location| (location, "settled here".to_owned())).into_iter().collect(),
        conflict,
        tier,
        pairing,
        confidence: left.confidence * right.confidence,
        repairs,
        entity: subject,
        details: "Two statements give one-valued attribute different values over overlapping story time, and no change event separates them.".into(),
        severity_override: None,
    }
}

fn other_start(first: &Source<'_>, second: &Source<'_>) -> Option<i64> {
    second
        .may
        .lo
        .filter(|lo| first.may.lo.is_none_or(|first_lo| *lo > first_lo))
}

fn observation_conflict(
    ir: &StoryIr,
    predicate: &PredicateId,
    source: &Source<'_>,
    observation: &Observation,
    at: &TimeBounds,
    tier: Tier,
) -> Violation {
    let who = name_of(ir, &observation.fluent.subject);
    let label = ir
        .predicate(predicate)
        .map(|sig| sig.label.clone())
        .unwrap_or_else(|| predicate.0.replace('_', " "));
    let observation_statement = observation_statement(ir, observation);
    let mut third: Option<Statement> = None;
    if source.fact.is_some() {
        third = Some(
            Statement::new(
                StatementKind::Rule,
                format!(
                    "rule:no-change:{}:{}",
                    observation.fluent.family(),
                    observation.id.0
                ),
                format!(
                    "No event between {} and {} changes {who}'s {label}.",
                    crate::util::when(ir, &source.may),
                    crate::util::when(ir, at)
                ),
                Status::Settled,
            )
            .with_prior(0.02),
        );
    }
    let mut conflict = vec![source.statement.clone(), observation_statement.clone()];
    if let Some(third) = third {
        conflict.push(third);
    }
    let pairing = if source.status == Status::Settled {
        Pairing::SettledObserved
    } else {
        Pairing::ProposedProposed
    };
    let mut repairs = vec![
        Repair::new(
            RepairKind::AddChangeEvent {
                entity: observation.fluent.subject.clone(),
                predicate: predicate.0.clone(),
                to_value: serde_json::to_value(&observation.fluent.value).unwrap_or_default(),
                after_tick: source.may.lo,
                before_tick: at.lo,
            },
            "Add an event".to_owned(),
            1.0,
        ),
        Repair::new(
            RepairKind::MarkContext {
                document: observation.anchor.document.clone(),
                context: Context::Memory,
            },
            "Mark the scene as a memory".to_owned(),
            2.0,
        ),
    ];
    if let Some(fact) = source.fact {
        if let Some(lo) = at.lo {
            repairs.push(Repair::new(
                RepairKind::SetValidity {
                    fact: fact.id.0.clone(),
                    from_tick: None,
                    to_tick: Some(lo - 1),
                },
                format!(
                    "{} holds until {}",
                    fact.fluent.value.label(),
                    crate::util::when(ir, &TimeBounds::at(lo - 1))
                ),
                1.5,
            ));
        }
        repairs.push(Repair::new(
            RepairKind::DeprecateFact {
                fact: fact.id.0.clone(),
            },
            format!("Retire \u{201c}{}\u{201d}", fact.fluent.value.label()),
            6.0,
        ));
    }
    repairs.push(Repair::suppress());
    repairs.push(Repair::keep());
    Violation {
        code: catalog::ATTRIBUTE_CONTRADICTION.into(),
        title: format!("{who}'s {label} was settled as {}.", source.value.label()),
        primary: Some(location_of(ir, &observation.anchor)),
        related: source
            .statement
            .location
            .clone()
            .map(|location| (location, "settled here".to_owned()))
            .into_iter()
            .collect(),
        confidence: joint_confidence(&conflict),
        conflict,
        tier,
        pairing,
        repairs,
        entity: Some(observation.fluent.subject.clone()),
        details: format!(
            "The page says {}; canon says {}.",
            observation.fluent.value.label(),
            source.value.label()
        ),
        severity_override: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use asten_ir::{
        Anchor, ConceptId, DocId, Entity, EntityId, EventId, FactId, ObservationId, PredicateSig,
        Scene, Sort, TimeRef, ValueDomain, lattice::color_lattice,
    };

    fn color(id: &str) -> Value {
        Value::Concept {
            id: ConceptId::new(id),
            label: id.into(),
        }
    }

    fn fluent(subject: &str, predicate: &str, value: Value) -> Fluent {
        Fluent {
            predicate: PredicateId::new(predicate),
            subject: EntityId::new(subject),
            value,
        }
    }

    fn base_ir() -> StoryIr {
        let mut ir = StoryIr::default();
        ir.lattices.insert("color".into(), color_lattice());
        ir.predicates.push(PredicateSig {
            id: PredicateId::new("hair_color"),
            label: "hair".into(),
            subject_sorts: vec![Sort::Character],
            domain: ValueDomain::Concept {
                lattice: "color".into(),
            },
            cardinality: Cardinality::Functional,
            mutability: Mutability::Mutable,
            relation: Default::default(),
            cues: vec!["hair".into()],
            pack: "scribe".into(),
        });
        ir.entities.push(Entity {
            id: EntityId::new("mira"),
            sort: Sort::Character,
            name: "Mira".into(),
            aliases: Vec::new(),
            introduced_at: None,
        });
        for (doc, title, pos, tick) in [
            ("ch2", "Chapter 2", 2, 3),
            ("ch12", "Chapter 12", 12, 16),
            ("ch19", "Chapter 19", 19, 22),
        ] {
            ir.scenes.push(Scene {
                document: DocId::new(doc),
                title: title.into(),
                reading_pos: pos,
                at: TimeRef::Tick { tick },
                time_label: format!("day {tick}"),
                context: Context::Narration,
                continuity_run: None,
                language: None,
                blocks: Vec::new(),
            });
        }
        ir.facts.push(Fact {
            id: FactId::new("f-auburn"),
            fluent: fluent("mira", "hair_color", color("auburn")),
            context: Context::World,
            valid: TimeBounds::from(3),
            status: Status::Settled,
            provenance: asten_ir::Provenance::default(),
            confidence: 1.0,
            anchor: Some(Anchor::new("ch2", "b1", 0, 6)),
            settled_at: None,
        });
        ir.observations.push(Observation {
            id: ObservationId::new("o-black"),
            fluent: fluent("mira", "hair_color", color("black")),
            context: Context::Narration,
            at: TimeRef::Unknown,
            anchor: Anchor::new("ch19", "b9", 16, 26)
                .with_text("Mira pushed her black hair out of her eyes."),
            confidence: 0.9,
            negated: false,
            provenance: asten_ir::Provenance::default(),
            reading_pos: 19,
        });
        ir
    }

    #[test]
    fn observation_against_settled_fact_is_a_must_violation() {
        let ir = base_ir();
        let (times, _) = crate::resolve_times(&ir);
        let violations = check(&ir, &times);
        assert_eq!(violations.len(), 1);
        let violation = &violations[0];
        assert_eq!(violation.code, "CANON-E013");
        assert_eq!(violation.tier, Tier::Must);
        assert_eq!(violation.pairing, Pairing::SettledObserved);
        assert_eq!(
            violation.primary.as_ref().unwrap().document,
            DocId::new("ch19")
        );
        assert!(
            violation
                .repairs
                .iter()
                .any(|repair| repair.label == "Add an event")
        );
    }

    #[test]
    fn a_change_event_explains_the_new_value() {
        let mut ir = base_ir();
        ir.events.push(Event {
            id: EventId::new("e-dye"),
            kind: EventKind::Changes {
                fluent: fluent("mira", "hair_color", color("black")),
            },
            label: "Mira dyes her hair black".into(),
            subject: Some(EntityId::new("mira")),
            participants: Vec::new(),
            at: TimeRef::Scene {
                document: DocId::new("ch12"),
            },
            reading_pos: Some(12),
            context: Context::Narration,
            status: Status::Settled,
            confidence: 1.0,
            provenance: asten_ir::Provenance::default(),
            anchor: None,
            document: Some(DocId::new("ch12")),
            initiates: Vec::new(),
            terminates: Vec::new(),
        });
        let (times, _) = crate::resolve_times(&ir);
        assert!(check(&ir, &times).is_empty());

        // An auburn observation after the dye is now the error.
        ir.observations[0].fluent.value = color("auburn");
        ir.observations[0].anchor = Anchor::new("ch19", "b9", 0, 6);
        let violations = check(&ir, &times);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].tier, Tier::Must);
    }

    #[test]
    fn compatible_lattice_values_do_not_conflict() {
        let mut ir = base_ir();
        ir.observations[0].fluent.value = color("red");
        let (times, _) = crate::resolve_times(&ir);
        assert!(check(&ir, &times).is_empty());
    }

    #[test]
    fn uncertain_time_makes_a_may_violation() {
        let mut ir = base_ir();
        ir.facts[0].valid = TimeBounds::between(3, 20);
        ir.scenes[2].at = TimeRef::Bounds {
            lo: Some(18),
            hi: Some(25),
        };
        let (times, _) = crate::resolve_times(&ir);
        let violations = check(&ir, &times);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].tier, Tier::May);
    }

    #[test]
    fn settled_against_settled_is_an_error_pairing() {
        let mut ir = base_ir();
        ir.observations.clear();
        ir.facts.push(Fact {
            id: FactId::new("f-black"),
            fluent: fluent("mira", "hair_color", color("black")),
            context: Context::World,
            valid: TimeBounds::UNBOUNDED,
            status: Status::Settled,
            provenance: asten_ir::Provenance::default(),
            confidence: 1.0,
            anchor: None,
            settled_at: None,
        });
        let (times, _) = crate::resolve_times(&ir);
        let violations = check(&ir, &times);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].pairing, Pairing::SettledSettled);
    }

    #[test]
    fn dialogue_is_a_different_context() {
        let mut ir = base_ir();
        ir.observations[0].context = Context::Says { speaker: None };
        let (times, _) = crate::resolve_times(&ir);
        assert!(check(&ir, &times).is_empty());
    }
}
