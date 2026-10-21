//! The simple temporal network against brute force on small random networks
//! (§14.3): consistency, bounds, tightness and the explanation cycle.

use asten_ir::{Provenance, Status, TemporalConstraint, Tick, TimeBounds, TimePoint};
use asten_time::{Solution, Stn};
use proptest::prelude::*;

const CASES: u32 = 256;

/// Largest |weight| the generator uses. Any consistent network over `n`
/// points then has a solution with every value in `[-(n-1)·W, (n-1)·W]`
/// (take the Bellman–Ford distances from a virtual source, shifted so the
/// origin is zero), so a brute-force search over that window is complete.
const W: i64 = 3;
const MAX_NAMED: usize = 4;

/// `(from, to, lo, hi)` over point indices; index 0 is the origin.
type Raw = (usize, usize, Option<i64>, Option<i64>);

fn network() -> impl Strategy<Value = (usize, Vec<Raw>)> {
    (1..=MAX_NAMED).prop_flat_map(|named| {
        let bound = -W..=W;
        let point = 0..=named;
        (
            Just(named),
            prop::collection::vec(
                (
                    point.clone(),
                    point,
                    prop::option::weighted(0.8, bound.clone()),
                    prop::option::weighted(0.8, bound),
                ),
                1..=12,
            ),
        )
    })
}

fn point(index: usize) -> TimePoint {
    if index == 0 {
        TimePoint::Origin
    } else {
        TimePoint::Named {
            label: format!("p{index}"),
        }
    }
}

fn constraint((from, to, lo, hi): &Raw) -> TemporalConstraint {
    TemporalConstraint {
        from: point(*from),
        to: point(*to),
        lo: *lo,
        hi: *hi,
        label: format!("p{from}->p{to} in [{lo:?}, {hi:?}]"),
        provenance: Provenance::default(),
        status: Status::Settled,
        confidence: 1.0,
        anchor: None,
    }
}

fn build(named: usize, raws: &[Raw]) -> Stn {
    let mut stn = Stn::new();
    for index in 1..=named {
        stn.point(&point(index));
    }
    for raw in raws {
        stn.add(constraint(raw));
    }
    stn
}

fn satisfied(values: &[i64], (from, to, lo, hi): &Raw) -> bool {
    let diff = values[*to] - values[*from];
    lo.is_none_or(|lo| diff >= lo) && hi.is_none_or(|hi| diff <= hi)
}

/// Every solution inside the window, summarised as per-point `(min, max)`;
/// `None` when there is no solution. Depth-first with pruning: a constraint
/// is checked as soon as both its endpoints are assigned.
fn brute_force(named: usize, raws: &[Raw], window: i64) -> Option<Vec<(i64, i64)>> {
    let n = named + 1;
    let mut values = vec![0i64; n];
    let mut summary: Option<Vec<(i64, i64)>> = None;
    if raws
        .iter()
        .any(|raw| raw.0 == 0 && raw.1 == 0 && !satisfied(&values, raw))
    {
        return None;
    }
    fn search(
        depth: usize,
        values: &mut Vec<i64>,
        raws: &[Raw],
        window: i64,
        summary: &mut Option<Vec<(i64, i64)>>,
    ) {
        if depth == values.len() {
            match summary {
                Some(ranges) => {
                    for (range, value) in ranges.iter_mut().zip(values.iter()) {
                        range.0 = range.0.min(*value);
                        range.1 = range.1.max(*value);
                    }
                }
                None => *summary = Some(values.iter().map(|v| (*v, *v)).collect()),
            }
            return;
        }
        for value in -window..=window {
            values[depth] = value;
            let ok = raws
                .iter()
                .filter(|raw| raw.0.max(raw.1) == depth)
                .all(|raw| satisfied(values, raw));
            if ok {
                search(depth + 1, values, raws, window, summary);
            }
        }
    }
    search(1, &mut values, raws, window, &mut summary);
    summary
}

fn within(bounds: &TimeBounds, value: i64) -> bool {
    bounds.lo.is_none_or(|lo| lo <= value) && bounds.hi.is_none_or(|hi| value <= hi)
}

fn pinned(stn: &Stn, index: usize, value: Tick) -> Stn {
    let mut probe = stn.clone();
    probe.pin(
        &point(index),
        TimeBounds::at(value),
        "probe",
        Status::Settled,
    );
    probe
}

proptest! {
    #![proptest_config(ProptestConfig { cases: CASES, .. ProptestConfig::default() })]

    /// `solve` is consistent exactly when brute force finds a solution; its
    /// bounds contain every solution; each finite bound is attained; and an
    /// inconsistent network's explanation is itself inconsistent.
    #[test]
    fn solver_agrees_with_brute_force((named, raws) in network()) {
        let window = (named as i64) * W;
        let stn = build(named, &raws);
        let solution = stn.solve();
        let truth = brute_force(named, &raws, window);
        match (&solution, &truth) {
            (Solution::Consistent { bounds }, Some(ranges)) => {
                prop_assert_eq!(bounds.len(), named + 1);
                prop_assert_eq!(bounds[0], TimeBounds::at(0), "the origin is pinned at zero");
                for (index, (min, max)) in ranges.iter().enumerate() {
                    prop_assert!(within(&bounds[index], *min), "p{}: {} outside {:?}", index, min, bounds[index]);
                    prop_assert!(within(&bounds[index], *max), "p{}: {} outside {:?}", index, max, bounds[index]);
                    prop_assert!(bounds[index].lo.is_none_or(|lo| lo <= bounds[index].hi.unwrap_or(lo)));
                    // Tightness: pinning a point to either bound keeps the network consistent,
                    // and one step beyond a finite bound breaks it.
                    for edge in [bounds[index].lo, bounds[index].hi].into_iter().flatten() {
                        prop_assert!(
                            matches!(pinned(&stn, index, edge).solve(), Solution::Consistent { .. }),
                            "p{index} cannot actually take its bound {edge}"
                        );
                    }
                    if let Some(lo) = bounds[index].lo {
                        prop_assert!(matches!(pinned(&stn, index, lo - 1).solve(), Solution::Inconsistent { .. }), "p{} below its lower bound {} is still consistent", index, lo);
                    }
                    if let Some(hi) = bounds[index].hi {
                        prop_assert!(matches!(pinned(&stn, index, hi + 1).solve(), Solution::Inconsistent { .. }), "p{} above its upper bound {} is still consistent", index, hi);
                    }
                    // A bound within the window is the brute-force extreme itself.
                    if let Some(lo) = bounds[index].lo {
                        if lo > -window { prop_assert_eq!(lo, *min, "p{} lower bound is loose", index); }
                    }
                    if let Some(hi) = bounds[index].hi {
                        if hi < window { prop_assert_eq!(hi, *max, "p{} upper bound is loose", index); }
                    }
                }
                prop_assert_eq!(stn.bounds_of(&solution, &point(0)), TimeBounds::at(0));
            }
            (Solution::Inconsistent { cycle }, None) => {
                prop_assert!(!cycle.is_empty());
                prop_assert!(cycle.windows(2).all(|pair| pair[0] < pair[1]), "cycle is sorted and distinct: {cycle:?}");
                prop_assert!(cycle.iter().all(|index| *index < raws.len()));
                let subset: Vec<Raw> = cycle.iter().map(|index| raws[*index]).collect();
                prop_assert!(
                    matches!(build(named, &subset).solve(), Solution::Inconsistent { .. }),
                    "the explanation {cycle:?} is consistent on its own"
                );
                prop_assert!(brute_force(named, &subset, window).is_none());
                prop_assert_eq!(stn.bounds_of(&solution, &point(1)), TimeBounds::UNBOUNDED);
            }
            (Solution::Consistent { .. }, None) => prop_assert!(false, "solver says consistent, brute force finds nothing"),
            (Solution::Inconsistent { cycle }, Some(_)) => prop_assert!(false, "solver reports a cycle {cycle:?} but a solution exists"),
        }
    }

    /// `would_conflict` predicts exactly whether adding the constraint makes
    /// the network inconsistent, without changing it.
    #[test]
    fn would_conflict_is_a_dry_run((named, raws) in network(), extra in (0..=MAX_NAMED, 0..=MAX_NAMED, prop::option::of(-W..=W), prop::option::of(-W..=W))) {
        prop_assume!(extra.0 <= named && extra.1 <= named);
        let stn = build(named, &raws);
        let before = stn.constraints().len();
        let predicted = stn.would_conflict(&constraint(&extra));
        prop_assert_eq!(stn.constraints().len(), before);
        let mut with = stn.clone();
        with.add(constraint(&extra));
        let actual = matches!(with.solve(), Solution::Inconsistent { .. });
        prop_assert_eq!(predicted, actual);
    }

    /// Adding constraints never widens bounds, and a solution to the whole
    /// stays a solution to any prefix.
    #[test]
    fn bounds_only_tighten((named, raws) in network()) {
        let mut previous: Option<Vec<TimeBounds>> = None;
        for len in 0..=raws.len() {
            let stn = build(named, &raws[..len]);
            match stn.solve() {
                Solution::Consistent { bounds } => {
                    if let Some(previous) = &previous {
                        for (index, (now, then)) in bounds.iter().zip(previous).enumerate() {
                            prop_assert!(
                                now.lo.unwrap_or(i64::MIN) >= then.lo.unwrap_or(i64::MIN)
                                    && now.hi.unwrap_or(i64::MAX) <= then.hi.unwrap_or(i64::MAX),
                                "p{index} widened from {then:?} to {now:?} after constraint {len}"
                            );
                        }
                    }
                    previous = Some(bounds);
                }
                Solution::Inconsistent { .. } => {
                    // Once inconsistent, every longer prefix stays inconsistent.
                    for longer in len..=raws.len() {
                        prop_assert!(matches!(build(named, &raws[..longer]).solve(), Solution::Inconsistent { .. }), "prefix {} consistent after prefix {} was not", longer, len);
                    }
                    break;
                }
            }
        }
    }
}
