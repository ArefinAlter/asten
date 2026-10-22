//! Resolve story time: build the temporal network from scenes, events and
//! constraints, solve it, and report inconsistency as CANON-E202 with the
//! negative cycle as the explanation. Analyses then read bounds per scene and
//! event from the resulting table.

use std::collections::HashMap;

use asten_diagnostics::{
    Location, Pairing, Repair, RepairKind, Statement, StatementKind, Tier, Violation, catalog,
};
use asten_ir::{
    DocId, EventId, Status, StoryIr, TemporalConstraint, TimeBounds, TimePoint, TimeRef,
};
use asten_time::{Solution, Stn};

use crate::util::{location_of, name_of, pairing_of, scene_location};

/// Bounds for every scene and event, relative to the story's origin.
#[derive(Clone, Debug, Default)]
pub struct TimeTable {
    pub scenes: HashMap<DocId, TimeBounds>,
    pub events: HashMap<EventId, TimeBounds>,
    /// Constraints that had to be set aside to make the network solvable.
    pub dropped: Vec<usize>,
}

impl TimeTable {
    #[must_use]
    pub fn scene(&self, document: &DocId) -> TimeBounds {
        self.scenes
            .get(document)
            .copied()
            .unwrap_or(TimeBounds::UNBOUNDED)
    }

    #[must_use]
    pub fn event(&self, id: &EventId) -> TimeBounds {
        self.events
            .get(id)
            .copied()
            .unwrap_or(TimeBounds::UNBOUNDED)
    }

    /// Resolve a time reference to bounds.
    #[must_use]
    pub fn of(&self, at: &TimeRef) -> TimeBounds {
        match at {
            TimeRef::Tick { tick } => TimeBounds::at(*tick),
            TimeRef::Bounds { lo, hi } => TimeBounds { lo: *lo, hi: *hi },
            TimeRef::Scene { document } => self.scene(document),
            TimeRef::Unknown => TimeBounds::UNBOUNDED,
        }
    }
}

fn pin_ref(stn: &mut Stn, point: &TimePoint, at: &TimeRef, label: &str, status: Status) {
    match at {
        TimeRef::Tick { tick } => {
            stn.pin(point, TimeBounds::at(*tick), label, status);
        }
        TimeRef::Bounds { lo, hi } => {
            if lo.is_some() || hi.is_some() {
                stn.pin(point, TimeBounds { lo: *lo, hi: *hi }, label, status);
            }
        }
        TimeRef::Scene { document } => {
            stn.add(TemporalConstraint {
                from: TimePoint::Scene {
                    document: document.clone(),
                },
                to: point.clone(),
                lo: Some(0),
                hi: Some(0),
                label: label.to_owned(),
                provenance: asten_ir::Provenance::default(),
                status,
                confidence: 1.0,
                anchor: None,
            });
        }
        TimeRef::Unknown => {}
    }
}

/// Build the network from the IR.
#[must_use]
pub fn build_stn(ir: &StoryIr) -> Stn {
    let mut stn = Stn::new();
    for scene in &ir.scenes {
        let point = TimePoint::Scene {
            document: scene.document.clone(),
        };
        stn.point(&point);
        let label = if scene.time_label.is_empty() {
            format!("{} is placed at {}", scene.title, describe(&scene.at))
        } else {
            format!(
                "{} is placed at \u{201c}{}\u{201d}",
                scene.title, scene.time_label
            )
        };
        pin_ref(&mut stn, &point, &scene.at, &label, Status::Settled);
    }
    for event in &ir.events {
        let point = TimePoint::Event {
            id: event.id.clone(),
        };
        stn.point(&point);
        let who = event
            .subject
            .as_ref()
            .map(|id| name_of(ir, id))
            .unwrap_or_default();
        let label = if event.label.is_empty() {
            format!("{who} {} at {}", event.kind.label(), describe(&event.at))
        } else {
            format!("{} at {}", event.label, describe(&event.at))
        };
        pin_ref(&mut stn, &point, &event.at, &label, event.status);
    }
    for constraint in &ir.constraints {
        if constraint.status.is_active() {
            stn.add(constraint.clone());
        }
    }
    stn
}

fn describe(at: &TimeRef) -> String {
    match at {
        TimeRef::Tick { tick } => format!("tick {tick}"),
        TimeRef::Bounds { lo, hi } => match (lo, hi) {
            (Some(lo), Some(hi)) => format!("ticks {lo}–{hi}"),
            (Some(lo), None) => format!("tick {lo} or later"),
            (None, Some(hi)) => format!("tick {hi} or earlier"),
            (None, None) => "an unplaced time".into(),
        },
        TimeRef::Scene { document } => format!("the time of scene {document}"),
        TimeRef::Unknown => "an unplaced time".into(),
    }
}

/// Solve the network. When it is inconsistent, report E202 for each negative
/// cycle and drop the least trusted constraint of the cycle until it solves,
/// so the rest of the analyses still get bounds.
#[must_use]
pub fn resolve_times(ir: &StoryIr) -> (TimeTable, Vec<Violation>) {
    let mut stn = build_stn(ir);
    let mut violations = Vec::new();
    let mut dropped = Vec::new();
    let mut guard = 0;
    let solution = loop {
        match stn.solve() {
            solution @ Solution::Consistent { .. } => break solution,
            Solution::Inconsistent { cycle } => {
                violations.push(cycle_violation(ir, &stn, &cycle));
                // Drop the least trusted constraint in the cycle and try again.
                let victim = cycle
                    .iter()
                    .copied()
                    .filter(|index| !dropped.contains(index))
                    .min_by(|a, b| {
                        let ca = stn.constraint(*a).map(trust).unwrap_or(0.0);
                        let cb = stn.constraint(*b).map(trust).unwrap_or(0.0);
                        ca.partial_cmp(&cb).unwrap_or(std::cmp::Ordering::Equal)
                    });
                let Some(victim) = victim else {
                    break Solution::Consistent { bounds: Vec::new() };
                };
                dropped.push(victim);
                stn = rebuild_without(&stn, &dropped);
                guard += 1;
                if guard > 64 {
                    break Solution::Consistent { bounds: Vec::new() };
                }
            }
        }
    };
    let mut table = TimeTable {
        dropped,
        ..TimeTable::default()
    };
    for scene in &ir.scenes {
        let point = TimePoint::Scene {
            document: scene.document.clone(),
        };
        table
            .scenes
            .insert(scene.document.clone(), stn.bounds_of(&solution, &point));
    }
    for event in &ir.events {
        let point = TimePoint::Event {
            id: event.id.clone(),
        };
        table
            .events
            .insert(event.id.clone(), stn.bounds_of(&solution, &point));
    }
    (table, violations)
}

fn trust(constraint: &TemporalConstraint) -> f32 {
    let status = match constraint.status {
        Status::Settled => 1.0,
        Status::Proposed => 0.5,
        _ => 0.0,
    };
    status * constraint.confidence
}

fn rebuild_without(stn: &Stn, dropped: &[usize]) -> Stn {
    let mut next = Stn::new();
    for point in stn.points() {
        next.point(point);
    }
    for (index, constraint) in stn.constraints().iter().enumerate() {
        if dropped.contains(&index) {
            // Keep the index space aligned with a no-op constraint so cycle
            // indices stay meaningful across rebuilds.
            next.add(TemporalConstraint {
                lo: None,
                hi: None,
                ..constraint.clone()
            });
        } else {
            next.add(constraint.clone());
        }
    }
    next
}

fn cycle_violation(ir: &StoryIr, stn: &Stn, cycle: &[usize]) -> Violation {
    let mut statements: Vec<Statement> = cycle
        .iter()
        .filter_map(|index| stn.constraint(*index))
        .map(|constraint| {
            let location = constraint
                .anchor
                .as_ref()
                .map(|anchor| location_of(ir, anchor))
                .or_else(|| match (&constraint.from, &constraint.to) {
                    (_, TimePoint::Scene { document }) | (TimePoint::Scene { document }, _) => {
                        ir.scene(document).map(scene_location)
                    }
                    _ => None,
                });
            let prior = Statement::prior_from(
                &constraint.provenance,
                constraint.status,
                &asten_ir::Context::World,
                constraint.confidence,
            );
            Statement::new(
                StatementKind::Constraint,
                constraint_id(constraint),
                constraint.label.clone(),
                constraint.status,
            )
            .at(location)
            .with_prior(prior)
            .with_confidence(constraint.confidence)
        })
        .collect();
    statements.sort_by(|a, b| {
        a.fault_prior
            .partial_cmp(&b.fault_prior)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let primary = statements
        .iter()
        .rev()
        .find_map(|statement| statement.location.clone())
        .or_else(|| {
            statements
                .iter()
                .find_map(|statement| statement.location.clone())
        });
    let related: Vec<(Location, String)> = statements
        .iter()
        .filter_map(|statement| statement.location.clone())
        .filter(|location| Some(location) != primary.as_ref())
        .map(|location| (location, "also dated here".to_owned()))
        .collect();
    let mut repairs: Vec<Repair> = statements
        .iter()
        .filter(|statement| statement.status != Status::Settled || statement.fault_prior > 0.02)
        .take(3)
        .filter_map(|statement| {
            statement.location.as_ref().map(|location| {
                Repair::new(
                    RepairKind::PlaceScene {
                        document: location.document.clone(),
                        lo: None,
                        hi: None,
                    },
                    format!(
                        "Set the time of {}",
                        if location.label.is_empty() {
                            "this scene".to_owned()
                        } else {
                            location.label.clone()
                        }
                    ),
                    2.0 + statement.fault_prior,
                )
            })
        })
        .collect();
    repairs.push(Repair::suppress());
    repairs.push(Repair::keep());
    let pairing = if statements
        .iter()
        .all(|statement| statement.status == Status::Settled)
    {
        Pairing::SettledSettled
    } else {
        pairing_of(&statements)
    };
    let confidence = statements
        .iter()
        .map(|statement| statement.confidence)
        .product::<f32>();
    Violation {
        code: catalog::TEMPORAL_INCONSISTENCY.into(),
        title: "These dates cannot all be true.".into(),
        primary,
        related,
        conflict: statements,
        tier: Tier::Must,
        pairing,
        confidence,
        repairs,
        entity: None,
        details: "The listed statements form a cycle in story time that no placement satisfies. One of them is off.".into(),
        severity_override: None,
    }
}

fn constraint_id(constraint: &TemporalConstraint) -> String {
    format!(
        "constraint:{}->{}:{}:{}",
        constraint.from.label(),
        constraint.to.label(),
        constraint.lo.map_or("-".to_owned(), |lo| lo.to_string()),
        constraint.hi.map_or("-".to_owned(), |hi| hi.to_string())
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use asten_ir::{Context, Scene};

    fn scene(document: &str, title: &str, reading_pos: i64, at: TimeRef) -> Scene {
        Scene {
            document: DocId::new(document),
            title: title.into(),
            reading_pos,
            at,
            time_label: String::new(),
            context: Context::Narration,
            continuity_run: None,
            language: None,
            blocks: Vec::new(),
        }
    }

    #[test]
    fn resolves_scene_bounds_through_constraints() {
        let mut ir = StoryIr::default();
        ir.scenes
            .push(scene("siege", "The siege", 1, TimeRef::Tick { tick: 10 }));
        ir.scenes
            .push(scene("feast", "The feast", 2, TimeRef::Unknown));
        ir.constraints.push(TemporalConstraint {
            from: TimePoint::Scene {
                document: DocId::new("siege"),
            },
            to: TimePoint::Scene {
                document: DocId::new("feast"),
            },
            lo: Some(3),
            hi: Some(3),
            label: "the feast is three days after the siege".into(),
            provenance: asten_ir::Provenance::default(),
            status: Status::Settled,
            confidence: 1.0,
            anchor: None,
        });
        let (table, violations) = resolve_times(&ir);
        assert!(violations.is_empty());
        assert_eq!(table.scene(&DocId::new("feast")), TimeBounds::at(13));
    }

    #[test]
    fn reports_the_cycle_and_still_produces_bounds() {
        let mut ir = StoryIr::default();
        ir.scenes
            .push(scene("siege", "The siege", 1, TimeRef::Tick { tick: 10 }));
        ir.scenes
            .push(scene("feast", "The feast", 2, TimeRef::Tick { tick: 11 }));
        ir.constraints.push(TemporalConstraint {
            from: TimePoint::Scene {
                document: DocId::new("siege"),
            },
            to: TimePoint::Scene {
                document: DocId::new("feast"),
            },
            lo: Some(3),
            hi: Some(3),
            label: "chapter 3 says the feast is three days after the siege".into(),
            provenance: asten_ir::Provenance::default(),
            status: Status::Proposed,
            confidence: 0.8,
            anchor: None,
        });
        let (table, violations) = resolve_times(&ir);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].code, "CANON-E202");
        assert_eq!(violations[0].conflict.len(), 3);
        // The proposed constraint is dropped; the settled dates stand.
        assert_eq!(table.scene(&DocId::new("feast")), TimeBounds::at(11));
        assert_eq!(table.dropped.len(), 1);
    }
}
