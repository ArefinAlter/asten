//! Author invariants as the Dwyer, Avrunin and Corbett specification patterns
//! over a finite scene trace (§9.10). Verdicts are three-valued: a response
//! obligation is pending, not violated, while the payoff is still unwritten.

use asten_diagnostics::{
    Pairing, Repair, RepairKind, Severity, Statement, StatementKind, Tier, Violation, catalog,
};
use asten_ir::{
    EventKindPattern, Invariant, InvariantPattern, Proposition, Scene, Status, StoryIr, TraceClock,
};

use crate::state::{Holds, families, holds_at};
use crate::time::TimeTable;
use crate::util::{scene_location, scene_statement};

/// A pending obligation for the setup/payoff ledger.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Obligation {
    pub invariant: String,
    pub label: String,
    /// The scene where the trigger happened.
    pub since: asten_ir::DocId,
    pub since_title: String,
}

struct SceneState<'a> {
    scene: &'a Scene,
    bounds: asten_ir::TimeBounds,
}

/// CANON-E601 and the ledger (PLOT-I001).
#[must_use]
pub fn check(ir: &StoryIr, times: &TimeTable) -> (Vec<Violation>, Vec<Obligation>) {
    let mut violations = Vec::new();
    let mut ledger = Vec::new();
    let groups = families(ir, times);
    for invariant in ir.invariants.iter().filter(|invariant| invariant.enabled) {
        let mut trace: Vec<SceneState<'_>> = ir
            .scenes
            .iter()
            .map(|scene| SceneState {
                scene,
                bounds: times.scene(&scene.document),
            })
            .collect();
        match invariant.clock {
            TraceClock::ReadingOrder => trace.sort_by_key(|state| state.scene.reading_pos),
            TraceClock::StoryTime => trace.sort_by_key(|state| {
                (state.bounds.lo.unwrap_or(i64::MAX), state.scene.reading_pos)
            }),
        }
        let holds = |proposition: &Proposition, state: &SceneState<'_>| -> bool {
            match proposition {
                Proposition::Holds { fluent } => {
                    let key = (fluent.family(), asten_ir::Context::World);
                    groups
                        .get(&key)
                        .is_some_and(|family| holds_at(family, ir, &fluent.predicate, &fluent.value, &state.bounds) == Holds::Must)
                }
                Proposition::Happens { subject, event } => ir.events.iter().filter(|candidate| candidate.status.is_active()).any(|candidate| {
                    let in_scene = candidate.document.as_ref() == Some(&state.scene.document)
                        || matches!(&candidate.at, asten_ir::TimeRef::Scene { document } if *document == state.scene.document)
                        || (candidate.document.is_none() && !state.bounds.is_unbounded() && {
                            let bounds = times.event(&candidate.id);
                            bounds.is_point() && state.bounds.contains_tick(bounds.lo.unwrap_or_default())
                        });
                    in_scene && subject.as_ref().is_none_or(|subject| candidate.subject.as_ref() == Some(subject)) && event.matches(&candidate.kind)
                }),
                Proposition::Mentioned { entity } => ir.mentions.iter().any(|mention| mention.entity == *entity && mention.anchor.document == state.scene.document),
                Proposition::InContext { context } => state.scene.context == *context,
            }
        };
        match &invariant.pattern {
            InvariantPattern::AbsenceAfter { after, never } => {
                let mut armed: Option<&SceneState<'_>> = None;
                for state in &trace {
                    if let Some(trigger) = armed {
                        if holds(never, state) {
                            violations.push(violation(
                                ir,
                                invariant,
                                &[trigger, state],
                                format!("{} happens after it was ruled out.", describe(ir, never)),
                            ));
                        }
                    } else if holds(after, state) {
                        armed = Some(state);
                    }
                }
            }
            InvariantPattern::Response { trigger, response } => {
                let mut pending: Vec<&SceneState<'_>> = Vec::new();
                for state in &trace {
                    if !pending.is_empty() && holds(response, state) {
                        pending.clear();
                    }
                    if holds(trigger, state) {
                        pending.push(state);
                    }
                }
                for state in pending {
                    ledger.push(Obligation {
                        invariant: invariant.id.clone(),
                        label: invariant.label.clone(),
                        since: state.scene.document.clone(),
                        since_title: state.scene.title.clone(),
                    });
                    violations.push(Violation {
                        code: catalog::PENDING_SETUP.into(),
                        title: format!("{}: not paid off yet.", invariant.label),
                        primary: Some(scene_location(state.scene)),
                        related: Vec::new(),
                        conflict: vec![
                            scene_statement(ir, state.scene, &state.bounds),
                            invariant_statement(invariant),
                        ],
                        tier: Tier::May,
                        pairing: Pairing::SettledSettled,
                        confidence: 1.0,
                        repairs: vec![Repair::suppress(), Repair::keep()],
                        entity: None,
                        details:
                            "A ledger entry, not an error. It clears when the payoff is written."
                                .into(),
                        severity_override: Some(Severity::Information),
                    });
                }
            }
            InvariantPattern::Precedence { first, later } => {
                let mut seen_first = false;
                for state in &trace {
                    if holds(first, state) {
                        seen_first = true;
                    }
                    if !seen_first && holds(later, state) {
                        violations.push(violation(
                            ir,
                            invariant,
                            &[state],
                            format!(
                                "{} comes before {}.",
                                describe(ir, later),
                                describe(ir, first)
                            ),
                        ));
                    }
                }
            }
            InvariantPattern::UniversalityBetween {
                from,
                until,
                always,
            } => {
                let mut inside: Option<&SceneState<'_>> = None;
                for state in &trace {
                    if let Some(start) = inside {
                        if holds(until, state) {
                            inside = None;
                            continue;
                        }
                        if !holds(always, state) {
                            violations.push(violation(
                                ir,
                                invariant,
                                &[start, state],
                                format!("{} stops holding here.", describe(ir, always)),
                            ));
                        }
                    } else if holds(from, state) {
                        inside = Some(state);
                    }
                }
            }
            InvariantPattern::BoundedExistence { event, max } => {
                let hits: Vec<&SceneState<'_>> =
                    trace.iter().filter(|state| holds(event, state)).collect();
                if hits.len() as u32 > *max {
                    violations.push(violation(
                        ir,
                        invariant,
                        &hits,
                        format!(
                            "{} happens {} times; at most {} allowed.",
                            describe(ir, event),
                            hits.len(),
                            max
                        ),
                    ));
                }
            }
        }
    }
    (violations, ledger)
}

fn invariant_statement(invariant: &Invariant) -> Statement {
    Statement::new(
        StatementKind::Invariant,
        format!("invariant:{}", invariant.id),
        format!("Your rule: {}", invariant.label),
        Status::Settled,
    )
    .with_prior(0.1)
}

fn violation(
    ir: &StoryIr,
    invariant: &Invariant,
    scenes: &[&SceneState<'_>],
    why: String,
) -> Violation {
    let mut conflict = vec![invariant_statement(invariant)];
    conflict.extend(
        scenes
            .iter()
            .map(|state| scene_statement(ir, state.scene, &state.bounds)),
    );
    let primary = scenes.last().map(|state| scene_location(state.scene));
    Violation {
        code: catalog::INVARIANT_VIOLATED.into(),
        title: format!("{}: {why}", invariant.label),
        primary,
        related: scenes.iter().rev().skip(1).map(|state| (scene_location(state.scene), "also here".to_owned())).collect(),
        conflict,
        tier: Tier::Must,
        pairing: Pairing::SettledSettled,
        confidence: 1.0,
        repairs: vec![Repair::new(RepairKind::Keep, "Edit the rule", 1.0), Repair::suppress(), Repair::keep()],
        entity: None,
        details: "An invariant you wrote fails on the scene trace. Change the scenes, or change the rule.".into(),
        severity_override: None,
    }
}

fn describe(ir: &StoryIr, proposition: &Proposition) -> String {
    match proposition {
        Proposition::Holds { fluent } => crate::util::fluent_label(ir, fluent),
        Proposition::Happens { subject, event } => {
            let who = subject
                .as_ref()
                .map(|id| crate::util::name_of(ir, id))
                .unwrap_or_else(|| "someone".into());
            let what = match event {
                EventKindPattern::Any => "does anything".to_owned(),
                EventKindPattern::Other(label) => label.clone(),
                other => format!("{other:?}").to_lowercase(),
            };
            format!("{who} {what}")
        }
        Proposition::Mentioned { entity } => {
            format!("{} is mentioned", crate::util::name_of(ir, entity))
        }
        Proposition::InContext { context } => format!("the scene is {}", context.label()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use asten_ir::{
        Anchor, Context, DocId, Entity, EntityId, Event, EventId, EventKind, Mention, Sort, TimeRef,
    };

    fn scene(document: &str, pos: i64, tick: i64) -> Scene {
        Scene {
            document: DocId::new(document),
            title: format!("Chapter {pos}"),
            reading_pos: pos,
            at: TimeRef::Tick { tick },
            time_label: String::new(),
            context: Context::Narration,
            continuity_run: None,
            language: None,
            blocks: Vec::new(),
        }
    }

    fn ir() -> StoryIr {
        let mut ir = StoryIr::default();
        ir.entities.push(Entity {
            id: EntityId::new("oren"),
            sort: Sort::Character,
            name: "Oren".into(),
            aliases: Vec::new(),
            introduced_at: None,
        });
        for (doc, pos, tick) in [("c1", 1, 1), ("c2", 2, 5), ("c3", 3, 9)] {
            ir.scenes.push(scene(doc, pos, tick));
        }
        ir.events.push(Event {
            id: EventId::new("death"),
            kind: EventKind::Dies,
            label: String::new(),
            subject: Some(EntityId::new("oren")),
            participants: Vec::new(),
            at: TimeRef::Scene {
                document: DocId::new("c2"),
            },
            reading_pos: Some(2),
            context: Context::Narration,
            status: Status::Settled,
            confidence: 1.0,
            provenance: asten_ir::Provenance::default(),
            anchor: None,
            document: Some(DocId::new("c2")),
            initiates: Vec::new(),
            terminates: Vec::new(),
        });
        ir
    }

    #[test]
    fn absence_after_and_response_patterns() {
        let mut ir = ir();
        ir.mentions.push(Mention {
            anchor: Anchor::new("c3", "b", 0, 4),
            entity: EntityId::new("oren"),
            surface: "Oren".into(),
            confidence: 1.0,
            reading_pos: 3,
            context: Context::Narration,
        });
        ir.invariants.push(Invariant {
            id: "no-ghosts".into(),
            label: "Once Oren dies he is never mentioned".into(),
            pattern: InvariantPattern::AbsenceAfter {
                after: Proposition::Happens {
                    subject: Some(EntityId::new("oren")),
                    event: EventKindPattern::Dies,
                },
                never: Proposition::Mentioned {
                    entity: EntityId::new("oren"),
                },
            },
            clock: TraceClock::StoryTime,
            enabled: true,
        });
        ir.invariants.push(Invariant {
            id: "payoff".into(),
            label: "Every death is mourned".into(),
            pattern: InvariantPattern::Response {
                trigger: Proposition::Happens {
                    subject: None,
                    event: EventKindPattern::Dies,
                },
                response: Proposition::InContext {
                    context: Context::Memory,
                },
            },
            clock: TraceClock::ReadingOrder,
            enabled: true,
        });
        let (times, _) = crate::resolve_times(&ir);
        let (violations, ledger) = check(&ir, &times);
        assert!(
            violations
                .iter()
                .any(|violation| violation.code == "CANON-E601")
        );
        assert!(
            violations
                .iter()
                .any(|violation| violation.code == "PLOT-I001")
        );
        assert_eq!(ledger.len(), 1);
    }
}
