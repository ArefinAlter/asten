//! Typestate for lifecycles (§9.4): acting after death is use-after-free.
//!
//! Each entity's lifecycle events run in story-time order through a small
//! automaton. An act (an event or a mention in narration) while the entity is
//! `dead` or `destroyed`, or before it is `born`, is a violation. Because the
//! check runs on story time, a flashback needs no special marking: a scene late
//! in reading order but early in story time feeds its acts at the early time.

use asten_diagnostics::{
    Location, Repair, RepairKind, Statement, StatementKind, Tier, Violation, catalog,
};
use asten_ir::{Context, Entity, Event, EventKind, Mention, Sort, Status, StoryIr, TimeBounds};

use crate::time::TimeTable;
use crate::util::{event_statement, joint_confidence, location_of, pairing_of, scene_statement};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Transition {
    Birth,
    Death,
    Return,
}

struct Boundary<'a> {
    transition: Transition,
    bounds: TimeBounds,
    event: &'a Event,
}

struct Act<'a> {
    bounds: TimeBounds,
    statement: Statement,
    document: Option<asten_ir::DocId>,
    event: Option<&'a Event>,
    mention: Option<&'a Mention>,
}

fn transition_of(kind: &EventKind, sort: &Sort) -> Option<Transition> {
    match (kind, sort) {
        (EventKind::Born, _) => Some(Transition::Birth),
        (EventKind::Dies, Sort::Character | Sort::Group | Sort::Other) => Some(Transition::Death),
        (EventKind::Destroyed, _) => Some(Transition::Death),
        (EventKind::Resurrected | EventKind::Repaired, _) => Some(Transition::Return),
        _ => None,
    }
}

/// CANON-E101, E102, E103.
#[must_use]
pub fn check(ir: &StoryIr, times: &TimeTable) -> Vec<Violation> {
    let mut violations = Vec::new();
    let reliable = ir.settings.narrator_reliable;
    for entity in &ir.entities {
        let boundaries: Vec<Boundary<'_>> = ir
            .events
            .iter()
            .filter(|event| event.status.is_active() && event.subject.as_ref() == Some(&entity.id))
            .filter(|event| event.context.projects_to_world(reliable))
            .filter_map(|event| {
                transition_of(&event.kind, &entity.sort).map(|transition| Boundary {
                    transition,
                    bounds: times.event(&event.id),
                    event,
                })
            })
            .collect();
        if boundaries.is_empty() {
            continue;
        }
        let acts = acts_of(ir, times, entity);
        for boundary in &boundaries {
            match boundary.transition {
                Transition::Death => {
                    for act in &acts {
                        if act
                            .event
                            .is_some_and(|event| std::ptr::eq(event, boundary.event))
                        {
                            continue;
                        }
                        let Some(tier) = after(&act.bounds, &boundary.bounds) else {
                            continue;
                        };
                        // A return between the death and the act clears it.
                        let returned = boundaries.iter().any(|other| {
                            other.transition == Transition::Return
                                && after(&other.bounds, &boundary.bounds).is_some()
                                && after(&act.bounds, &other.bounds)
                                    .is_some_and(|t| t == Tier::Must || tier == Tier::May)
                        });
                        if returned {
                            continue;
                        }
                        if act.event.is_some_and(|event| {
                            matches!(event.kind, EventKind::Resurrected | EventKind::Repaired)
                        }) {
                            continue;
                        }
                        let code = if entity.sort == Sort::Character || entity.sort == Sort::Group {
                            catalog::LIFECYCLE_AFTER_DEATH
                        } else {
                            catalog::DESTROYED_REAPPEARS
                        };
                        violations.push(lifecycle_violation(ir, entity, code, boundary, act, tier));
                    }
                }
                Transition::Birth => {
                    for act in &acts {
                        if act
                            .event
                            .is_some_and(|event| std::ptr::eq(event, boundary.event))
                        {
                            continue;
                        }
                        let Some(tier) = after(&boundary.bounds, &act.bounds) else {
                            continue;
                        };
                        violations.push(lifecycle_violation(
                            ir,
                            entity,
                            catalog::BEFORE_BIRTH,
                            boundary,
                            act,
                            tier,
                        ));
                    }
                }
                Transition::Return => {}
            }
        }
    }
    violations
}

/// Whether `later` comes after `earlier`: `Must` when certainly, `May` when possibly.
fn after(later: &TimeBounds, earlier: &TimeBounds) -> Option<Tier> {
    match (later.lo, earlier.hi) {
        (Some(lo), Some(hi)) if lo > hi => return Some(Tier::Must),
        _ => {}
    }
    match (later.hi, earlier.lo) {
        (Some(later_hi), Some(earlier_lo)) if later_hi <= earlier_lo => None,
        // Unbounded on either side: possible either way, unless both unknown.
        (None, None) if later.is_unbounded() && earlier.is_unbounded() => None,
        _ => Some(Tier::May),
    }
}

fn acts_of<'a>(ir: &'a StoryIr, times: &TimeTable, entity: &Entity) -> Vec<Act<'a>> {
    let reliable = ir.settings.narrator_reliable;
    let mut acts = Vec::new();
    for event in ir.events.iter().filter(|event| event.status.is_active()) {
        let involved =
            event.subject.as_ref() == Some(&entity.id) || event.participants.contains(&entity.id);
        if !involved || !event.context.projects_to_world(reliable) {
            continue;
        }
        if transition_of(&event.kind, &entity.sort).is_some() {
            continue;
        }
        // Being talked about or told something is not acting.
        if matches!(event.kind, EventKind::Tells { ref to, .. } if *to == entity.id) {
            continue;
        }
        let bounds = times.event(&event.id);
        acts.push(Act {
            bounds,
            statement: event_statement(ir, event, &bounds),
            document: event.document.clone(),
            event: Some(event),
            mention: None,
        });
    }
    for mention in ir
        .mentions
        .iter()
        .filter(|mention| mention.entity == entity.id)
    {
        if !mention.context.projects_to_world(reliable) {
            continue;
        }
        let Some(scene) = ir.scene(&mention.anchor.document) else {
            continue;
        };
        if !scene.context.projects_to_world(reliable) {
            continue;
        }
        let bounds = times.scene(&scene.document);
        if bounds.is_unbounded() {
            continue;
        }
        let text = format!(
            "{} mentions {} in narration: \u{201c}{}\u{201d}",
            scene.title, entity.name, mention.anchor.quote
        );
        let statement = Statement::new(
            StatementKind::Mention,
            format!(
                "mention:{}:{}:{}",
                mention.anchor.document.0, mention.anchor.block.0, mention.anchor.start
            ),
            text,
            Status::Proposed,
        )
        .at(Some(location_of(ir, &mention.anchor)))
        .with_prior((1.0 - mention.confidence).max(0.12))
        .with_confidence(mention.confidence);
        acts.push(Act {
            bounds,
            statement,
            document: Some(scene.document.clone()),
            event: None,
            mention: Some(mention),
        });
    }
    acts
}

fn lifecycle_violation(
    ir: &StoryIr,
    entity: &Entity,
    code: &str,
    boundary: &Boundary<'_>,
    act: &Act<'_>,
    tier: Tier,
) -> Violation {
    let boundary_statement = event_statement(ir, boundary.event, &boundary.bounds);
    let mut conflict = vec![boundary_statement.clone(), act.statement.clone()];
    if let Some(document) = &act.document {
        if let Some(scene) = ir.scene(document) {
            if act.mention.is_some() {
                conflict.push(scene_statement(ir, scene, &times_for(ir, scene, act)));
            }
        }
    }
    let title = match code {
        catalog::BEFORE_BIRTH => format!("{} appears before being born.", entity.name),
        catalog::DESTROYED_REAPPEARS => format!("{} is used after being destroyed.", entity.name),
        _ => format!("{} acts after a settled death.", entity.name),
    };
    let mut repairs = Vec::new();
    if let Some(document) = &act.document {
        repairs.push(Repair::new(
            RepairKind::MarkContext {
                document: document.clone(),
                context: Context::Memory,
            },
            "Mark the scene as a memory",
            1.0,
        ));
        repairs.push(Repair::new(
            RepairKind::PlaceScene {
                document: document.clone(),
                lo: None,
                hi: if code == catalog::BEFORE_BIRTH {
                    None
                } else {
                    boundary.bounds.lo.map(|lo| lo - 1)
                },
            },
            "Set the scene's time",
            1.5,
        ));
    }
    repairs.push(Repair::new(
        RepairKind::MoveEvent {
            event: boundary.event.id.0.clone(),
            lo: if code == catalog::BEFORE_BIRTH {
                None
            } else {
                act.bounds.hi.map(|hi| hi + 1)
            },
            hi: if code == catalog::BEFORE_BIRTH {
                act.bounds.lo.map(|lo| lo - 1)
            } else {
                None
            },
        },
        match code {
            catalog::BEFORE_BIRTH => "Move the birth earlier",
            catalog::DESTROYED_REAPPEARS => "Move the destruction later",
            _ => "Move the death later",
        },
        if boundary.event.status == Status::Settled {
            5.0
        } else {
            2.0
        },
    ));
    repairs.push(Repair::suppress());
    repairs.push(Repair::keep());
    let related: Vec<(Location, String)> = boundary_statement
        .location
        .clone()
        .map(|location| (location, "settled here".to_owned()))
        .into_iter()
        .collect();
    Violation {
        code: code.into(),
        title,
        primary: act.statement.location.clone().or_else(|| boundary_statement.location.clone()),
        related,
        confidence: joint_confidence(&conflict),
        pairing: pairing_of(&conflict),
        conflict,
        tier,
        repairs,
        entity: Some(entity.id.clone()),
        details: "Lifecycle checks run in story time, so a flashback needs no marking: place the scene and the check follows.".into(),
        severity_override: None,
    }
}

fn times_for(_ir: &StoryIr, _scene: &asten_ir::Scene, act: &Act<'_>) -> TimeBounds {
    act.bounds
}

#[cfg(test)]
mod tests {
    use super::*;
    use asten_ir::{Anchor, DocId, EntityId, EventId, Scene, TimeRef};

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

    fn death(at: TimeRef) -> Event {
        Event {
            id: EventId::new("e-death"),
            kind: EventKind::Dies,
            label: String::new(),
            subject: Some(EntityId::new("oren")),
            participants: Vec::new(),
            at,
            reading_pos: Some(9),
            context: Context::Narration,
            status: Status::Settled,
            confidence: 1.0,
            provenance: asten_ir::Provenance::default(),
            anchor: None,
            document: Some(DocId::new("ch9")),
            initiates: Vec::new(),
            terminates: Vec::new(),
        }
    }

    fn mention(document: &str, pos: i64) -> Mention {
        Mention {
            anchor: Anchor::new(document, "b1", 0, 4).with_text("Oren poured the tea."),
            entity: EntityId::new("oren"),
            surface: "Oren".into(),
            confidence: 0.97,
            reading_pos: pos,
            context: Context::Narration,
        }
    }

    fn ir() -> StoryIr {
        let mut ir = StoryIr::default();
        ir.entities.push(Entity {
            id: EntityId::new("oren"),
            sort: Sort::Character,
            name: "Captain Oren".into(),
            aliases: Vec::new(),
            introduced_at: None,
        });
        ir.scenes.push(scene("ch9", "Chapter 9", 9, 14));
        ir.scenes.push(scene("ch14", "Chapter 14", 14, 20));
        ir.scenes.push(scene("ch3", "Chapter 3", 3, 5));
        ir.events.push(death(TimeRef::Scene {
            document: DocId::new("ch9"),
        }));
        ir
    }

    #[test]
    fn acting_after_death_is_flagged_and_flashbacks_are_not() {
        let mut ir = ir();
        ir.mentions.push(mention("ch14", 14));
        ir.mentions.push(mention("ch3", 3)); // a flashback: late in reading order would not matter either
        let (times, _) = crate::resolve_times(&ir);
        let violations = check(&ir, &times);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].code, "CANON-E101");
        assert_eq!(violations[0].tier, Tier::Must);
        assert_eq!(
            violations[0].primary.as_ref().unwrap().document,
            DocId::new("ch14")
        );
    }

    #[test]
    fn uncertain_scene_time_is_a_may_violation() {
        let mut ir = ir();
        ir.scenes[1].at = TimeRef::Bounds {
            lo: Some(10),
            hi: Some(20),
        };
        ir.mentions.push(mention("ch14", 14));
        let (times, _) = crate::resolve_times(&ir);
        let violations = check(&ir, &times);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].tier, Tier::May);
    }

    #[test]
    fn a_return_clears_later_acts() {
        let mut ir = ir();
        ir.events.push(Event {
            id: EventId::new("e-return"),
            kind: EventKind::Resurrected,
            at: TimeRef::Tick { tick: 16 },
            ..death(TimeRef::Unknown)
        });
        ir.mentions.push(mention("ch14", 14));
        let (times, _) = crate::resolve_times(&ir);
        assert!(check(&ir, &times).is_empty());
    }

    #[test]
    fn memories_and_dialogue_do_not_count_as_acting() {
        let mut ir = ir();
        ir.scenes[1].context = Context::Memory;
        ir.scenes[1].at = TimeRef::Tick { tick: 8 };
        ir.mentions.push(mention("ch14", 14));
        let mut spoken = mention("ch14", 14);
        spoken.context = Context::Says { speaker: None };
        ir.mentions.push(spoken);
        let (times, _) = crate::resolve_times(&ir);
        assert!(check(&ir, &times).is_empty());
    }
}
