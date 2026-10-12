//! Knowledge and information flow (§9.8). Each secret carries the set of
//! agents who know it, which grows only along learning, telling and revealing
//! events in story time. Using knowledge before any of those is a leak.

use asten_diagnostics::{
    Pairing, Repair, RepairKind, Statement, StatementKind, Tier, Violation, catalog,
};
use asten_ir::{EntityId, EventKind, Status, StoryIr, TimeBounds};

use crate::time::TimeTable;
use crate::util::{event_statement, joint_confidence, name_of, pairing_of};

fn normalise(secret: &str) -> String {
    secret.trim().to_lowercase()
}

/// CANON-E401.
#[must_use]
pub fn check(ir: &StoryIr, times: &TimeTable) -> Vec<Violation> {
    let mut violations = Vec::new();
    for use_event in ir.events.iter().filter(|event| event.status.is_active()) {
        let EventKind::UsesKnowledge { secret } = &use_event.kind else {
            continue;
        };
        let Some(who) = &use_event.subject else {
            continue;
        };
        let secret_key = normalise(secret);
        let at = times.event(&use_event.id);
        // Learning events for this agent and secret.
        let sources: Vec<(TimeBounds, &asten_ir::Event)> = ir
            .events
            .iter()
            .filter(|event| event.status.is_active())
            .filter(|event| learns(event, who, &secret_key))
            .map(|event| (times.event(&event.id), event))
            .collect();
        // Must leak: no source may have happened before the use.
        let any_may_before = sources.iter().any(|(bounds, _)| match (bounds.lo, at.hi) {
            (Some(lo), Some(hi)) => lo <= hi,
            _ => true,
        });
        let any_must_before = sources
            .iter()
            .any(|(bounds, _)| matches!((bounds.hi, at.lo), (Some(hi), Some(lo)) if hi <= lo));
        let tier = if !any_may_before {
            Tier::Must
        } else if !any_must_before {
            Tier::May
        } else {
            continue;
        };
        let mut conflict = vec![event_statement(ir, use_event, &at)];
        if sources.is_empty() {
            conflict.push(
                Statement::new(
                    StatementKind::Rule,
                    format!("rule:never-learns:{}:{secret_key}", who.0),
                    format!("No event lets {} learn {secret}.", name_of(ir, who)),
                    Status::Settled,
                )
                .with_prior(0.3),
            );
        } else {
            for (bounds, event) in &sources {
                conflict.push(event_statement(ir, event, bounds));
            }
        }
        let primary = conflict[0].location.clone();
        let earliest_source = sources.iter().filter_map(|(bounds, _)| bounds.lo).min();
        violations.push(Violation {
            code: catalog::KNOWLEDGE_LEAK.into(),
            title: format!("{} knows about {secret} before learning it.", name_of(ir, who)),
            primary,
            related: Vec::new(),
            confidence: joint_confidence(&conflict),
            pairing: if sources.is_empty() { Pairing::SettledObserved } else { pairing_of(&conflict) },
            conflict,
            tier,
            repairs: vec![
                Repair::new(RepairKind::MoveEvent { event: use_event.id.0.clone(), lo: earliest_source, hi: None }, "Move the use later", 1.0),
                Repair::new(RepairKind::MarkContext { document: use_event.document.clone().unwrap_or_default(), context: asten_ir::Context::Hypothetical }, "Mark as a guess", 2.0),
                Repair::suppress(),
                Repair::keep(),
            ],
            entity: Some(who.clone()),
            details: "Add a learning, telling or reveal event before this point, or let the character deduce it on the page.".into(),
            severity_override: None,
        });
    }
    violations
}

fn learns(event: &asten_ir::Event, who: &EntityId, secret: &str) -> bool {
    match &event.kind {
        EventKind::Learns { secret: learned } => {
            event.subject.as_ref() == Some(who) && normalise(learned) == secret
        }
        EventKind::Tells { to, secret: told } => to == who && normalise(told) == secret,
        EventKind::Reveals { secret: revealed } => normalise(revealed) == secret,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use asten_ir::{Context, Entity, Event, EventId, Sort, TimeRef};

    fn event(id: &str, kind: EventKind, subject: &str, tick: i64) -> Event {
        Event {
            id: EventId::new(id),
            kind,
            label: String::new(),
            subject: Some(EntityId::new(subject)),
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
        }
    }

    #[test]
    fn leaks_before_learning_and_not_after() {
        let mut ir = StoryIr::default();
        ir.entities.push(Entity {
            id: EntityId::new("mira"),
            sort: Sort::Character,
            name: "Mira".into(),
            aliases: Vec::new(),
            introduced_at: None,
        });
        ir.events.push(event(
            "use",
            EventKind::UsesKnowledge {
                secret: "the will".into(),
            },
            "mira",
            10,
        ));
        ir.events.push(event(
            "learn",
            EventKind::Learns {
                secret: "The Will".into(),
            },
            "mira",
            12,
        ));
        let (times, _) = crate::resolve_times(&ir);
        let violations = check(&ir, &times);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].tier, Tier::Must);
        ir.events[1].at = TimeRef::Tick { tick: 8 };
        let (times, _) = crate::resolve_times(&ir);
        assert!(check(&ir, &times).is_empty());
    }
}
