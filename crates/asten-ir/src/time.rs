//! Story time is integer ticks in a project-chosen base unit (§6.2, §9.6).
//! Calendars that map labels to ticks live in `asten-time`; the IR only needs
//! the tick type and bounded intervals.

use serde::{Deserialize, Serialize};

/// A point in story time, in the project's base unit.
pub type Tick = i64;

/// A closed interval in story time with optional bounds. `None` is unbounded.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export, export_to = "ir/"))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct TimeBounds {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(value_type = Option<i64>))]
    pub lo: Option<Tick>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(value_type = Option<i64>))]
    pub hi: Option<Tick>,
}

impl TimeBounds {
    pub const UNBOUNDED: TimeBounds = TimeBounds { lo: None, hi: None };

    #[must_use]
    pub const fn at(tick: Tick) -> Self {
        Self {
            lo: Some(tick),
            hi: Some(tick),
        }
    }

    #[must_use]
    pub const fn between(lo: Tick, hi: Tick) -> Self {
        Self {
            lo: Some(lo),
            hi: Some(hi),
        }
    }

    #[must_use]
    pub const fn from(lo: Tick) -> Self {
        Self {
            lo: Some(lo),
            hi: None,
        }
    }

    #[must_use]
    pub const fn until(hi: Tick) -> Self {
        Self {
            lo: None,
            hi: Some(hi),
        }
    }

    #[must_use]
    pub fn is_unbounded(&self) -> bool {
        self.lo.is_none() && self.hi.is_none()
    }

    #[must_use]
    pub fn is_point(&self) -> bool {
        matches!((self.lo, self.hi), (Some(a), Some(b)) if a == b)
    }

    /// Whether the interval holds no tick at all: `lo > hi`. The analyses use
    /// this form when clipping shows a value never holds (§9.3); no other
    /// `TimeBounds` can say "never".
    #[must_use]
    pub fn is_empty(&self) -> bool {
        matches!((self.lo, self.hi), (Some(lo), Some(hi)) if lo > hi)
    }

    /// Whether the two intervals can overlap under some choice within their
    /// bounds. An empty interval overlaps nothing.
    #[must_use]
    pub fn may_overlap(&self, other: &TimeBounds) -> bool {
        if self.is_empty() || other.is_empty() {
            return false;
        }
        let starts_before_other_ends = match (self.lo, other.hi) {
            (Some(lo), Some(hi)) => lo <= hi,
            _ => true,
        };
        let other_starts_before_self_ends = match (other.lo, self.hi) {
            (Some(lo), Some(hi)) => lo <= hi,
            _ => true,
        };
        starts_before_other_ends && other_starts_before_self_ends
    }

    /// Whether `point`, whose own time is only known within `point_bounds`, is
    /// necessarily inside this interval under every allowed placement. An
    /// empty interval contains nothing, and an empty point has no placement.
    #[must_use]
    pub fn must_contain(&self, point_bounds: &TimeBounds) -> bool {
        if self.is_empty() || point_bounds.is_empty() {
            return false;
        }
        let (Some(p_lo), Some(p_hi)) = (point_bounds.lo, point_bounds.hi) else {
            // An unbounded point could lie anywhere; only an unbounded interval must contain it.
            return self.is_unbounded();
        };
        let lower_ok = self.lo.is_none_or(|lo| lo <= p_lo);
        let upper_ok = self.hi.is_none_or(|hi| p_hi <= hi);
        lower_ok && upper_ok
    }

    /// The intersection, or `None` when empty.
    #[must_use]
    pub fn intersect(&self, other: &TimeBounds) -> Option<TimeBounds> {
        let lo = match (self.lo, other.lo) {
            (Some(a), Some(b)) => Some(a.max(b)),
            (a, b) => a.or(b),
        };
        let hi = match (self.hi, other.hi) {
            (Some(a), Some(b)) => Some(a.min(b)),
            (a, b) => a.or(b),
        };
        match (lo, hi) {
            (Some(l), Some(h)) if l > h => None,
            _ => Some(TimeBounds { lo, hi }),
        }
    }

    #[must_use]
    pub fn contains_tick(&self, tick: Tick) -> bool {
        self.lo.is_none_or(|lo| lo <= tick) && self.hi.is_none_or(|hi| tick <= hi)
    }
}

#[cfg(test)]
mod tests {
    use super::TimeBounds;

    #[test]
    fn overlap_and_containment() {
        let a = TimeBounds::between(3, 10);
        let b = TimeBounds::between(8, 12);
        assert!(a.may_overlap(&b));
        assert!(!a.may_overlap(&TimeBounds::from(11)));
        assert!(TimeBounds::from(3).must_contain(&TimeBounds::at(19)));
        assert!(!TimeBounds::between(3, 10).must_contain(&TimeBounds::between(8, 12)));
        assert!(TimeBounds::UNBOUNDED.must_contain(&TimeBounds::UNBOUNDED));
        assert!(!TimeBounds::from(3).must_contain(&TimeBounds::UNBOUNDED));
        assert_eq!(a.intersect(&b), Some(TimeBounds::between(8, 10)));
        assert_eq!(a.intersect(&TimeBounds::from(11)), None);
    }
}
