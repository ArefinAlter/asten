//! Space and travel (§9.7). Travel reduces to time: consecutive appearances of
//! an entity at two places must be at least the shortest route apart. Without
//! a route graph nothing is generated.

use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap};

use asten_diagnostics::{Repair, RepairKind, Statement, StatementKind, Tier, Violation, catalog};
use asten_ir::{EntityId, EventKind, Status, StoryIr, Tick, TimeBounds, Value};

use crate::time::TimeTable;
use crate::util::{event_statement, fact_statement, joint_confidence, name_of, pairing_of};

struct Appearance {
    place: EntityId,
    bounds: TimeBounds,
    statement: Statement,
}

/// Shortest travel time between places over the route graph, in ticks.
#[must_use]
pub fn shortest_travel(ir: &StoryIr, from: &EntityId, to: &EntityId) -> Option<Tick> {
    if from == to {
        return Some(0);
    }
    let mut adjacency: HashMap<&EntityId, Vec<(&EntityId, Tick)>> = HashMap::new();
    for route in &ir.routes {
        adjacency
            .entry(&route.from)
            .or_default()
            .push((&route.to, route.min_ticks));
        if route.bidirectional {
            adjacency
                .entry(&route.to)
                .or_default()
                .push((&route.from, route.min_ticks));
        }
    }
    let mut best: HashMap<&EntityId, Tick> = HashMap::new();
    let mut heap = BinaryHeap::new();
    best.insert(from, 0);
    heap.push(Reverse((0, from)));
    while let Some(Reverse((cost, node))) = heap.pop() {
        if node == to {
            return Some(cost);
        }
        if best.get(node).is_some_and(|known| *known < cost) {
            continue;
        }
        for (next, weight) in adjacency.get(node).into_iter().flatten() {
            let candidate = cost + weight;
            if best.get(next).is_none_or(|known| candidate < *known) {
                best.insert(next, candidate);
                heap.push(Reverse((candidate, *next)));
            }
        }
    }
    None
}

/// CANON-E301.
#[must_use]
pub fn check(ir: &StoryIr, times: &TimeTable) -> Vec<Violation> {
    if ir.routes.is_empty() {
        return Vec::new();
    }
    let mut violations = Vec::new();
    let mut by_entity: HashMap<&EntityId, Vec<Appearance>> = HashMap::new();
    for event in ir.events.iter().filter(|event| event.status.is_active()) {
        let place = match &event.kind {
            EventKind::Arrives { place } => place,
            EventKind::Travels {
                to: Some(place), ..
            } => place,
            _ => continue,
        };
        let Some(subject) = &event.subject else {
            continue;
        };
        let bounds = times.event(&event.id);
        by_entity.entry(subject).or_default().push(Appearance {
            place: place.clone(),
            bounds,
            statement: event_statement(ir, event, &bounds),
        });
    }
    for fact in ir
        .facts
        .iter()
        .filter(|fact| fact.status.is_active() && fact.fluent.predicate.0 == "location")
    {
        let Value::Entity { id: place } = &fact.fluent.value else {
            continue;
        };
        by_entity
            .entry(&fact.fluent.subject)
            .or_default()
            .push(Appearance {
                place: place.clone(),
                bounds: fact.valid,
                statement: fact_statement(ir, fact),
            });
    }
    for (entity, mut appearances) in by_entity {
        appearances
            .retain(|appearance| appearance.bounds.lo.is_some() || appearance.bounds.hi.is_some());
        appearances.sort_by_key(|appearance| appearance.bounds.lo.unwrap_or(i64::MIN));
        for pair in appearances.windows(2) {
            let (a, b) = (&pair[0], &pair[1]);
            if a.place == b.place {
                continue;
            }
            let Some(travel) = shortest_travel(ir, &a.place, &b.place) else {
                continue;
            };
            // Certainly impossible: even the widest gap is too short.
            let tier = match (b.bounds.hi, a.bounds.lo) {
                (Some(b_hi), Some(a_lo)) if b_hi - a_lo < travel => Tier::Must,
                _ => match (b.bounds.lo, a.bounds.hi) {
                    (Some(b_lo), Some(a_hi)) if b_lo - a_hi < travel => Tier::May,
                    _ => continue,
                },
            };
            let route = Statement::new(
                StatementKind::Rule,
                format!("route:{}:{}", a.place.0, b.place.0),
                format!(
                    "The shortest route from {} to {} takes {}.",
                    name_of(ir, &a.place),
                    name_of(ir, &b.place),
                    crate::util::when(ir, &TimeBounds::at(travel))
                        .trim_start_matches("day ")
                        .to_owned()
                        + " days"
                ),
                Status::Settled,
            )
            .with_prior(0.1);
            let conflict = vec![a.statement.clone(), b.statement.clone(), route];
            violations.push(Violation {
                code: catalog::IMPOSSIBLE_JOURNEY.into(),
                title: format!("{} cannot reach {} from {} in time.", name_of(ir, entity), name_of(ir, &b.place), name_of(ir, &a.place)),
                primary: b.statement.location.clone(),
                related: a.statement.location.clone().map(|location| (location, "left from here".to_owned())).into_iter().collect(),
                confidence: joint_confidence(&conflict),
                pairing: pairing_of(&conflict),
                conflict,
                tier,
                repairs: vec![Repair::new(RepairKind::Keep, "Add a faster route", 1.0), Repair::suppress(), Repair::keep()],
                entity: Some(entity.clone()),
                details: "Travel reduces to time: the gap between the two appearances is shorter than the shortest route allows.".into(),
                severity_override: None,
            });
        }
    }
    violations
}

#[cfg(test)]
mod tests {
    use super::*;
    use asten_ir::{Context, Entity, Event, EventId, Route, Sort, TimeRef};

    #[test]
    fn journeys_shorter_than_the_route_are_flagged() {
        let mut ir = StoryIr::default();
        for (id, sort) in [
            ("mira", Sort::Character),
            ("veld", Sort::Place),
            ("harbor", Sort::Place),
        ] {
            ir.entities.push(Entity {
                id: EntityId::new(id),
                sort,
                name: id.to_uppercase(),
                aliases: Vec::new(),
                introduced_at: None,
            });
        }
        ir.routes.push(Route {
            from: EntityId::new("veld"),
            to: EntityId::new("harbor"),
            min_ticks: 3,
            max_ticks: None,
            mode: "road".into(),
            bidirectional: true,
            available: TimeBounds::UNBOUNDED,
        });
        let arrive = |id: &str, place: &str, tick: i64| Event {
            id: EventId::new(id),
            kind: EventKind::Arrives {
                place: EntityId::new(place),
            },
            label: String::new(),
            subject: Some(EntityId::new("mira")),
            participants: Vec::new(),
            at: TimeRef::Tick { tick },
            reading_pos: None,
            context: Context::Narration,
            status: Status::Settled,
            confidence: 1.0,
            provenance: asten_ir::Provenance::default(),
            anchor: None,
            document: None,
            initiates: Vec::new(),
            terminates: Vec::new(),
        };
        ir.events.push(arrive("a", "veld", 10));
        ir.events.push(arrive("b", "harbor", 11));
        let (times, _) = crate::resolve_times(&ir);
        let violations = check(&ir, &times);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].code, "CANON-E301");
        ir.events[1].at = TimeRef::Tick { tick: 14 };
        let (times, _) = crate::resolve_times(&ir);
        assert!(check(&ir, &times).is_empty());
    }
}
