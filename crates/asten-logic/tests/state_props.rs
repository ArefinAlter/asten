//! Event-calculus clipping (§9.3) against naive simulation over every
//! placement of the change events: the must interval only claims ticks where
//! the value holds under every placement, the may interval only ticks where
//! it holds under some, and no interval is ever malformed.

use asten_ir::{
    ConceptId, Context, Entity, EntityId, Event, EventId, EventKind, Fact, FactId, Fluent,
    PredicateId, Provenance, Sort, Status, StoryIr, TimeBounds, TimeRef, Value,
};
use asten_logic::resolve_times;
use asten_logic::state::families;
use proptest::prelude::*;

const CASES: u32 = 256;
/// Event times live in `0..=LAST`; the simulation looks a little beyond.
const LAST: i64 = 8;
const LO: i64 = -2;
const HI: i64 = LAST + 3;

fn value(index: u8) -> Value {
    let label = ["auburn", "black", "grey"][index as usize];
    Value::Concept {
        id: ConceptId::new(label),
        label: label.into(),
    }
}

fn fluent(index: u8) -> Fluent {
    Fluent {
        predicate: PredicateId::new("hair_color"),
        subject: EntityId::new("mira"),
        value: value(index),
    }
}

#[derive(Clone, Debug)]
struct FactSpec {
    value: u8,
    lo: Option<i64>,
    hi: Option<i64>,
}

#[derive(Clone, Debug)]
struct EventSpec {
    value: u8,
    lo: i64,
    hi: i64,
}

fn fact_spec() -> impl Strategy<Value = FactSpec> {
    (
        0..3u8,
        prop::option::of(0..=LAST),
        prop::option::of(0..=LAST),
    )
        .prop_map(|(value, a, b)| {
            let (lo, hi) = match (a, b) {
                (Some(a), Some(b)) => (Some(a.min(b)), Some(a.max(b))),
                other => other,
            };
            FactSpec { value, lo, hi }
        })
}

fn event_spec() -> impl Strategy<Value = EventSpec> {
    (0..3u8, 0..=LAST, 0..=LAST).prop_map(|(value, a, b)| EventSpec {
        value,
        lo: a.min(b),
        hi: a.max(b),
    })
}

fn story() -> impl Strategy<Value = (Vec<FactSpec>, Vec<EventSpec>)> {
    (
        prop::collection::vec(fact_spec(), 0..=2),
        prop::collection::vec(event_spec(), 0..=3),
    )
        .prop_filter("something to clip", |(facts, events)| {
            !facts.is_empty() || !events.is_empty()
        })
}

fn build(facts: &[FactSpec], events: &[EventSpec]) -> StoryIr {
    let mut ir = StoryIr::default();
    ir.entities.push(Entity {
        id: EntityId::new("mira"),
        sort: Sort::Character,
        name: "Mira".into(),
        aliases: Vec::new(),
        introduced_at: None,
    });
    for (index, spec) in facts.iter().enumerate() {
        ir.facts.push(Fact {
            id: FactId::new(format!("f{index}")),
            fluent: fluent(spec.value),
            context: Context::World,
            valid: TimeBounds {
                lo: spec.lo,
                hi: spec.hi,
            },
            status: Status::Settled,
            provenance: Provenance::default(),
            confidence: 1.0,
            anchor: None,
            settled_at: None,
        });
    }
    for (index, spec) in events.iter().enumerate() {
        ir.events.push(Event {
            id: EventId::new(format!("e{index}")),
            kind: EventKind::Changes {
                fluent: fluent(spec.value),
            },
            label: format!("change {index}"),
            subject: Some(EntityId::new("mira")),
            participants: Vec::new(),
            at: if spec.lo == spec.hi {
                TimeRef::Tick { tick: spec.lo }
            } else {
                TimeRef::Bounds {
                    lo: Some(spec.lo),
                    hi: Some(spec.hi),
                }
            },
            reading_pos: None,
            context: Context::Narration,
            status: Status::Settled,
            confidence: 1.0,
            provenance: Provenance::default(),
            anchor: None,
            document: None,
            initiates: Vec::new(),
            terminates: Vec::new(),
        });
    }
    ir
}

/// Which source a family entry came from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Who {
    Fact(usize),
    Event(usize),
}

/// Whether source `who` holds at `t` when the events happen at `times`.
/// A value holds from its start (a fact's `valid.lo`, an event's time) until
/// its end or until a change to another value happens at or after the start.
fn holds(facts: &[FactSpec], events: &[EventSpec], times: &[i64], who: Who, t: i64) -> bool {
    let (value, start, end) = match who {
        Who::Fact(index) => (facts[index].value, facts[index].lo, facts[index].hi),
        Who::Event(index) => (events[index].value, Some(times[index]), None),
    };
    if start.is_some_and(|start| t < start) || end.is_some_and(|end| t > end) {
        return false;
    }
    !events.iter().enumerate().any(|(index, change)| {
        change.value != value
            && start.is_none_or(|start| start <= times[index])
            && times[index] <= t
    })
}

/// Every assignment of a time to each event within its bounds.
fn placements(events: &[EventSpec]) -> Vec<Vec<i64>> {
    let mut out = vec![Vec::new()];
    for event in events {
        out = out
            .into_iter()
            .flat_map(|prefix| {
                (event.lo..=event.hi).map(move |time| {
                    let mut next = prefix.clone();
                    next.push(time);
                    next
                })
            })
            .collect();
    }
    out
}

fn contains(bounds: &TimeBounds, t: i64) -> bool {
    bounds.lo.is_none_or(|lo| lo <= t) && bounds.hi.is_none_or(|hi| t <= hi)
}

proptest! {
    #![proptest_config(ProptestConfig { cases: CASES, .. ProptestConfig::default() })]

    #[test]
    fn clipping_is_sound_and_well_formed((facts, events) in story()) {
        let ir = build(&facts, &events);
        let (times, violations) = resolve_times(&ir);
        prop_assert!(violations.is_empty(), "the network is consistent by construction");
        let groups = families(&ir, &times);
        prop_assert_eq!(groups.len(), 1, "one family");
        let family = groups.values().next().unwrap();
        prop_assert_eq!(family.sources.len(), facts.len() + events.len());
        let all = placements(&events);

        for source in &family.sources {
            let who = match (source.fact, source.event) {
                (Some(fact), None) => Who::Fact(fact.id.0[1..].parse().unwrap()),
                (None, Some(event)) => Who::Event(event.id.0[1..].parse().unwrap()),
                other => panic!("source without a single origin: {other:?}"),
            };
            // Well-formed, or the one canonical empty form `hi == lo - 1`.
            for (name, bounds) in [("must", source.must), ("may", source.may)] {
                if let (Some(lo), Some(hi)) = (bounds.lo, bounds.hi) {
                    prop_assert!(hi >= lo - 1, "{who:?} {name} is {bounds:?}");
                }
            }
            if !source.must.is_empty() {
                prop_assert!(
                    source.must.lo.unwrap_or(i64::MIN) >= source.may.lo.unwrap_or(i64::MIN)
                        && source.must.hi.unwrap_or(i64::MAX) <= source.may.hi.unwrap_or(i64::MAX),
                    "{who:?}: must {:?} is not inside may {:?}", source.must, source.may
                );
            }
            for t in LO..=HI {
                let outcomes: Vec<bool> = all.iter().map(|times| holds(&facts, &events, times, who, t)).collect();
                let exact_must = outcomes.iter().all(|held| *held);
                let exact_may = outcomes.iter().any(|held| *held);
                if contains(&source.must, t) {
                    prop_assert!(exact_must, "{who:?} must {:?} claims tick {t}, but it does not always hold", source.must);
                }
                if contains(&source.may, t) {
                    prop_assert!(exact_may, "{who:?} may {:?} claims tick {t}, but it never holds", source.may);
                }
                if exact_must {
                    prop_assert!(contains(&source.may, t), "{who:?} always holds at {t} but may {:?} excludes it", source.may);
                }
            }
        }
    }

    /// Clipping only ever shortens a source: without events, must and may are
    /// the fact's own validity.
    #[test]
    fn facts_alone_keep_their_validity(facts in prop::collection::vec(fact_spec(), 1..=3)) {
        let ir = build(&facts, &[]);
        let (times, _) = resolve_times(&ir);
        let groups = families(&ir, &times);
        let family = groups.values().next().unwrap();
        for source in &family.sources {
            let fact = source.fact.expect("a fact");
            prop_assert_eq!(source.must, fact.valid);
            prop_assert_eq!(source.may, fact.valid);
        }
    }
}
