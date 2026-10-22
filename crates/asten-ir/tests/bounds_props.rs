//! `TimeBounds` against a tick-set model. `lo > hi` is the empty interval:
//! nothing overlaps it, it contains nothing, and it is never "must" anything.

use std::collections::BTreeSet;

use asten_ir::{Tick, TimeBounds};
use proptest::prelude::*;

const CASES: u32 = 256;
const WINDOW: Tick = 12;

fn bounds() -> impl Strategy<Value = TimeBounds> {
    (prop::option::of(-5..=5i64), prop::option::of(-5..=5i64))
        .prop_map(|(lo, hi)| TimeBounds { lo, hi })
}

/// The ticks of the model window the interval contains.
fn ticks(bounds: &TimeBounds) -> BTreeSet<Tick> {
    (-WINDOW..=WINDOW)
        .filter(|t| bounds.lo.is_none_or(|lo| lo <= *t) && bounds.hi.is_none_or(|hi| *t <= hi))
        .collect()
}

proptest! {
    #![proptest_config(ProptestConfig { cases: CASES, .. ProptestConfig::default() })]

    #[test]
    fn empty_means_no_ticks(a in bounds()) {
        prop_assert_eq!(a.is_empty(), ticks(&a).is_empty());
        for t in -WINDOW..=WINDOW {
            prop_assert_eq!(a.contains_tick(t), ticks(&a).contains(&t));
        }
        prop_assert_eq!(a.is_point(), ticks(&a).len() == 1 && !a.is_unbounded());
    }

    #[test]
    fn overlap_is_a_shared_tick(a in bounds(), b in bounds()) {
        let shared = !ticks(&a).is_disjoint(&ticks(&b));
        prop_assert_eq!(a.may_overlap(&b), shared, "{:?} vs {:?}", a, b);
        prop_assert_eq!(a.may_overlap(&b), b.may_overlap(&a));
    }

    #[test]
    fn intersection_is_the_common_ticks(a in bounds(), b in bounds()) {
        let common: BTreeSet<Tick> = ticks(&a).intersection(&ticks(&b)).copied().collect();
        match a.intersect(&b) {
            Some(c) => {
                prop_assert_eq!(ticks(&c), common, "{:?} ∩ {:?} = {:?}", a, b, c);
                prop_assert!(!c.is_empty());
            }
            None => prop_assert!(common.is_empty(), "{a:?} ∩ {b:?} is {common:?}"),
        }
    }

    /// `must_contain`: every placement of the point lies inside the interval.
    /// A point with an open side could lie anywhere, so only an unbounded
    /// interval must contain it (the documented conservative rule); an empty
    /// point has no placement at all and is never contained.
    #[test]
    fn must_contain_covers_every_placement(a in bounds(), p in bounds()) {
        let expected = match (p.lo, p.hi) {
            (Some(_), Some(_)) => !p.is_empty() && !a.is_empty() && ticks(&p).is_subset(&ticks(&a)),
            _ => a.is_unbounded(),
        };
        prop_assert_eq!(a.must_contain(&p), expected, "{:?} must contain {:?}", a, p);
        if a.must_contain(&p) {
            prop_assert!(a.may_overlap(&p) || p.is_unbounded(), "must implies may");
        }
    }
}
