//! `resolve_times` on random inconsistent networks: it terminates, drops one
//! constraint per reported cycle, and leaves a consistent table whose bounds
//! are those of the surviving network. Every E202 names a set of statements
//! that really cannot all hold.

use asten_ir::{
    Context, DocId, Provenance, Scene, Status, StoryIr, TemporalConstraint, TimePoint, TimeRef,
};
use asten_logic::time::{build_stn, resolve_times};
use asten_time::{Solution, Stn};
use proptest::prelude::*;

const CASES: u32 = 256;
const SCENES: usize = 4;

fn doc(index: usize) -> DocId {
    DocId::new(format!("s{index}"))
}

fn point(index: usize) -> TimePoint {
    if index == 0 {
        TimePoint::Origin
    } else {
        TimePoint::Scene {
            document: doc(index - 1),
        }
    }
}

fn time_ref() -> impl Strategy<Value = TimeRef> {
    prop_oneof![
        2 => Just(TimeRef::Unknown),
        2 => (-6..=6i64).prop_map(|tick| TimeRef::Tick { tick }),
        2 => (prop::option::of(-6..=6i64), prop::option::of(-6..=6i64))
            .prop_map(|(lo, hi)| TimeRef::Bounds { lo, hi }),
        1 => (0..SCENES).prop_map(|index| TimeRef::Scene { document: doc(index) }),
    ]
}

fn status() -> impl Strategy<Value = Status> {
    prop_oneof![
        3 => Just(Status::Settled),
        3 => Just(Status::Proposed),
        1 => Just(Status::Deprecated),
    ]
}

fn constraint() -> impl Strategy<Value = TemporalConstraint> {
    (
        0..=SCENES,
        0..=SCENES,
        prop::option::weighted(0.8, -4..=4i64),
        prop::option::weighted(0.8, -4..=4i64),
        status(),
        prop::sample::select(vec![0.2f32, 0.5, 0.8, 1.0]),
    )
        .prop_map(
            |(from, to, lo, hi, status, confidence)| TemporalConstraint {
                from: point(from),
                to: point(to),
                lo,
                hi,
                label: format!("{from}->{to} in [{lo:?}, {hi:?}]"),
                provenance: Provenance::default(),
                status,
                confidence,
                anchor: None,
            },
        )
}

fn story() -> impl Strategy<Value = StoryIr> {
    (
        prop::collection::vec(time_ref(), SCENES),
        prop::collection::vec(constraint(), 0..=12),
    )
        .prop_map(|(placements, constraints)| {
            let mut ir = StoryIr::default();
            for (index, at) in placements.into_iter().enumerate() {
                ir.scenes.push(Scene {
                    document: doc(index),
                    title: format!("Scene {index}"),
                    reading_pos: index as i64,
                    at,
                    time_label: String::new(),
                    context: Context::Narration,
                    continuity_run: None,
                    language: None,
                    blocks: Vec::new(),
                });
            }
            ir.constraints = constraints;
            ir
        })
}

/// The statement id `cycle_violation` gives a constraint.
fn constraint_id(constraint: &TemporalConstraint) -> String {
    format!(
        "constraint:{}->{}:{}:{}",
        constraint.from.label(),
        constraint.to.label(),
        constraint.lo.map_or("-".to_owned(), |lo| lo.to_string()),
        constraint.hi.map_or("-".to_owned(), |hi| hi.to_string())
    )
}

fn trust(constraint: &TemporalConstraint) -> f32 {
    let status = match constraint.status {
        Status::Settled => 1.0,
        Status::Proposed => 0.5,
        _ => 0.0,
    };
    status * constraint.confidence
}

/// The network with the given constraint indices removed.
fn without(full: &Stn, dropped: &[usize]) -> Stn {
    let mut stn = Stn::new();
    for point in full.points() {
        stn.point(point);
    }
    for (index, constraint) in full.constraints().iter().enumerate() {
        if !dropped.contains(&index) {
            stn.add(constraint.clone());
        }
    }
    stn
}

proptest! {
    #![proptest_config(ProptestConfig { cases: CASES, .. ProptestConfig::default() })]

    #[test]
    fn dropping_least_trusted_constraints_reaches_consistency(ir in story()) {
        let full = build_stn(&ir);
        // Statement ids name a constraint by endpoints and bounds, so two
        // constraints that differ only in status share one; keep them apart so
        // an explanation maps back to exactly its constraints.
        let mut ids: Vec<String> = full.constraints().iter().map(constraint_id).collect();
        ids.sort();
        ids.dedup();
        prop_assume!(ids.len() == full.constraints().len());
        let (table, violations) = resolve_times(&ir);

        // One E202 per dropped constraint, each with a non-empty conflict set.
        prop_assert_eq!(violations.len(), table.dropped.len());
        prop_assert!(table.dropped.len() <= full.constraints().len());
        let mut seen = std::collections::BTreeSet::new();
        for index in &table.dropped {
            prop_assert!(*index < full.constraints().len());
            prop_assert!(seen.insert(*index), "constraint {index} dropped twice");
        }
        for violation in &violations {
            prop_assert_eq!(violation.code.as_str(), "CANON-E202");
            prop_assert!(!violation.conflict.is_empty());
        }

        // What remains is consistent, and the table reports exactly its bounds.
        let remaining = without(&full, &table.dropped);
        let solution = remaining.solve();
        prop_assert!(matches!(solution, Solution::Consistent { .. }), "{:?}", table.dropped);
        for scene in &ir.scenes {
            let point = TimePoint::Scene { document: scene.document.clone() };
            prop_assert_eq!(table.scene(&scene.document), remaining.bounds_of(&solution, &point));
            let bounds = table.scene(&scene.document);
            prop_assert!(bounds.lo.is_none_or(|lo| bounds.hi.is_none_or(|hi| lo <= hi)), "{bounds:?}");
        }
        if violations.is_empty() {
            prop_assert!(matches!(full.solve(), Solution::Consistent { .. }), "nothing dropped but the network is inconsistent");
        }

        // Each explanation names statements that cannot all hold, and the
        // constraint dropped for it is among the least trusted of them.
        for (violation, dropped) in violations.iter().zip(&table.dropped) {
            let ids: Vec<&str> = violation.conflict.iter().map(|statement| statement.id.as_str()).collect();
            let members: Vec<usize> = full
                .constraints()
                .iter()
                .enumerate()
                .filter(|(_, constraint)| ids.contains(&constraint_id(constraint).as_str()))
                .map(|(index, _)| index)
                .collect();
            prop_assert!(members.contains(dropped), "dropped {dropped} is not in the cycle {members:?}");
            let core = members.iter().map(|index| full.constraints()[*index].clone()).fold(Stn::new(), |mut stn, constraint| {
                stn.add(constraint);
                stn
            });
            prop_assert!(matches!(core.solve(), Solution::Inconsistent { .. }), "explanation {members:?} is consistent");
            let dropped_trust = trust(&full.constraints()[*dropped]);
            for index in &members {
                prop_assert!(trust(&full.constraints()[*index]) >= dropped_trust - f32::EPSILON);
            }
        }
    }

    /// Resolution is deterministic.
    #[test]
    fn resolution_is_deterministic(ir in story()) {
        let (first, first_violations) = resolve_times(&ir);
        let (second, second_violations) = resolve_times(&ir);
        prop_assert_eq!(first.dropped, second.dropped);
        prop_assert_eq!(first.scenes, second.scenes);
        prop_assert_eq!(first_violations.len(), second_violations.len());
    }
}
