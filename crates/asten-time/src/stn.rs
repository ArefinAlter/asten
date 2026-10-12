//! The simple temporal network (Dechter, Meiri & Pearl 1991).
//!
//! Every constraint is `lo ≤ X_to − X_from ≤ hi`. In the distance graph an edge
//! `from → to` with weight `hi` and an edge `to → from` with weight `−lo` encode
//! it. The network is consistent exactly when the distance graph has no
//! negative cycle, and a negative cycle is itself the explanation: its edges are
//! a set of statements that cannot all be true. Bounds relative to the origin
//! give the earliest and latest value of every time point, which drive the
//! may/must distinction in the analyses.

use std::collections::HashMap;

use asten_ir::{TemporalConstraint, Tick, TimeBounds, TimePoint};

#[derive(Clone, Debug)]
struct Edge {
    from: usize,
    to: usize,
    weight: Tick,
    /// Index into `Stn::constraints`.
    constraint: usize,
}

/// A simple temporal network over named time points.
#[derive(Clone, Debug, Default)]
pub struct Stn {
    points: Vec<TimePoint>,
    index: HashMap<TimePoint, usize>,
    edges: Vec<Edge>,
    constraints: Vec<TemporalConstraint>,
}

/// The outcome of solving the network.
#[derive(Clone, Debug, PartialEq)]
pub enum Solution {
    Consistent {
        /// Bounds of every point relative to the origin, by point index.
        bounds: Vec<TimeBounds>,
    },
    Inconsistent {
        /// Indices into the constraint list of a negative cycle: a minimal set
        /// of statements that cannot all hold.
        cycle: Vec<usize>,
    },
}

impl Stn {
    #[must_use]
    pub fn new() -> Self {
        let mut stn = Self::default();
        stn.point(&TimePoint::Origin);
        stn
    }

    /// Intern a time point and return its index. The origin is always index 0.
    pub fn point(&mut self, point: &TimePoint) -> usize {
        if let Some(&index) = self.index.get(point) {
            return index;
        }
        let index = self.points.len();
        self.points.push(point.clone());
        self.index.insert(point.clone(), index);
        index
    }

    #[must_use]
    pub fn index_of(&self, point: &TimePoint) -> Option<usize> {
        self.index.get(point).copied()
    }

    #[must_use]
    pub fn points(&self) -> &[TimePoint] {
        &self.points
    }

    #[must_use]
    pub fn constraints(&self) -> &[TemporalConstraint] {
        &self.constraints
    }

    #[must_use]
    pub fn constraint(&self, index: usize) -> Option<&TemporalConstraint> {
        self.constraints.get(index)
    }

    /// Add a constraint. Returns its index.
    pub fn add(&mut self, constraint: TemporalConstraint) -> usize {
        let from = self.point(&constraint.from);
        let to = self.point(&constraint.to);
        let index = self.constraints.len();
        if let Some(hi) = constraint.hi {
            self.edges.push(Edge {
                from,
                to,
                weight: hi,
                constraint: index,
            });
        }
        if let Some(lo) = constraint.lo {
            self.edges.push(Edge {
                from: to,
                to: from,
                weight: lo.checked_neg().unwrap_or(i64::MAX),
                constraint: index,
            });
        }
        self.constraints.push(constraint);
        index
    }

    /// Pin a point to exact bounds relative to the origin.
    pub fn pin(
        &mut self,
        point: &TimePoint,
        bounds: TimeBounds,
        label: impl Into<String>,
        status: asten_ir::Status,
    ) -> usize {
        self.add(TemporalConstraint {
            from: TimePoint::Origin,
            to: point.clone(),
            lo: bounds.lo,
            hi: bounds.hi,
            label: label.into(),
            provenance: asten_ir::Provenance::default(),
            status,
            confidence: 1.0,
            anchor: None,
        })
    }

    /// Solve the network: consistency, bounds, or the negative cycle.
    #[must_use]
    pub fn solve(&self) -> Solution {
        let n = self.points.len();
        if n == 0 {
            return Solution::Consistent { bounds: Vec::new() };
        }
        // Negative-cycle detection with a virtual source connected to every point.
        if let Some(cycle) = self.negative_cycle() {
            return Solution::Inconsistent { cycle };
        }
        // Upper bounds: shortest paths from the origin. X_v − X_0 ≤ d(0, v).
        let upper = self.shortest_from(0, false);
        // Lower bounds: shortest paths to the origin. X_0 − X_v ≤ d(v, 0), so X_v ≥ −d(v, 0).
        let lower = self.shortest_from(0, true);
        let bounds = (0..n)
            .map(|v| TimeBounds {
                lo: lower[v].map(|d| -d),
                hi: upper[v],
            })
            .collect();
        Solution::Consistent { bounds }
    }

    /// Whether adding `constraint` would make the network inconsistent, without
    /// adding it. Used to attribute a new conflict to the newest statement.
    #[must_use]
    pub fn would_conflict(&self, constraint: &TemporalConstraint) -> bool {
        let mut probe = self.clone();
        probe.add(constraint.clone());
        matches!(probe.solve(), Solution::Inconsistent { .. })
    }

    /// Shortest distances from `source` over the distance graph (or its reverse).
    /// `None` means unreachable (unbounded).
    fn shortest_from(&self, source: usize, reversed: bool) -> Vec<Option<Tick>> {
        let n = self.points.len();
        let mut dist: Vec<Option<Tick>> = vec![None; n];
        dist[source] = Some(0);
        for _ in 0..n.saturating_sub(1) {
            let mut changed = false;
            for edge in &self.edges {
                let (u, v) = if reversed {
                    (edge.to, edge.from)
                } else {
                    (edge.from, edge.to)
                };
                if let Some(du) = dist[u] {
                    let candidate = du.saturating_add(edge.weight);
                    if dist[v].is_none_or(|dv| candidate < dv) {
                        dist[v] = Some(candidate);
                        changed = true;
                    }
                }
            }
            if !changed {
                break;
            }
        }
        dist
    }

    /// Bellman–Ford with a virtual source. Returns the constraint indices on a
    /// negative cycle, if one exists.
    fn negative_cycle(&self) -> Option<Vec<usize>> {
        let n = self.points.len();
        // Distances start at 0 everywhere (virtual source with zero-weight edges).
        let mut dist: Vec<Tick> = vec![0; n];
        let mut pred: Vec<Option<usize>> = vec![None; n]; // predecessor edge index
        let mut last_relaxed: Option<usize> = None;
        for iteration in 0..n {
            last_relaxed = None;
            for (edge_index, edge) in self.edges.iter().enumerate() {
                let candidate = dist[edge.from].saturating_add(edge.weight);
                if candidate < dist[edge.to] {
                    dist[edge.to] = candidate;
                    pred[edge.to] = Some(edge_index);
                    last_relaxed = Some(edge.to);
                }
            }
            last_relaxed?;
            if iteration + 1 == n {
                break;
            }
        }
        let mut node = last_relaxed?;
        // Walk back n steps to land inside the cycle.
        for _ in 0..n {
            let edge = pred[node]?;
            node = self.edges[edge].from;
        }
        // Collect the cycle.
        let start = node;
        let mut cycle = Vec::new();
        loop {
            let edge_index = pred[node]?;
            let edge = &self.edges[edge_index];
            if !cycle.contains(&edge.constraint) {
                cycle.push(edge.constraint);
            }
            node = edge.from;
            if node == start {
                break;
            }
        }
        cycle.sort_unstable();
        Some(cycle)
    }

    /// Bounds for a point after solving, if the network is consistent.
    #[must_use]
    pub fn bounds_of(&self, solution: &Solution, point: &TimePoint) -> TimeBounds {
        match solution {
            Solution::Consistent { bounds } => self
                .index_of(point)
                .and_then(|index| bounds.get(index).copied())
                .unwrap_or(TimeBounds::UNBOUNDED),
            Solution::Inconsistent { .. } => TimeBounds::UNBOUNDED,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use asten_ir::{Provenance, Status};

    fn constraint(
        from: &str,
        to: &str,
        lo: Option<Tick>,
        hi: Option<Tick>,
        label: &str,
    ) -> TemporalConstraint {
        TemporalConstraint {
            from: TimePoint::Named { label: from.into() },
            to: TimePoint::Named { label: to.into() },
            lo,
            hi,
            label: label.into(),
            provenance: Provenance::default(),
            status: Status::Settled,
            confidence: 1.0,
            anchor: None,
        }
    }

    #[test]
    fn computes_bounds_relative_to_the_origin() {
        let mut stn = Stn::new();
        let siege = TimePoint::Named {
            label: "siege".into(),
        };
        stn.pin(
            &siege,
            TimeBounds::at(10),
            "siege on day 10",
            Status::Settled,
        );
        stn.add(constraint(
            "siege",
            "feast",
            Some(3),
            Some(3),
            "feast three days after the siege",
        ));
        stn.add(constraint(
            "feast",
            "treaty",
            Some(2),
            Some(6),
            "treaty a few days after the feast",
        ));
        let solution = stn.solve();
        assert_eq!(
            stn.bounds_of(
                &solution,
                &TimePoint::Named {
                    label: "feast".into()
                }
            ),
            TimeBounds::at(13)
        );
        assert_eq!(
            stn.bounds_of(
                &solution,
                &TimePoint::Named {
                    label: "treaty".into()
                }
            ),
            TimeBounds::between(15, 19)
        );
    }

    #[test]
    fn explains_a_negative_cycle_with_the_offending_constraints() {
        let mut stn = Stn::new();
        let siege = TimePoint::Named {
            label: "siege".into(),
        };
        let feast = TimePoint::Named {
            label: "feast".into(),
        };
        let a = stn.pin(
            &siege,
            TimeBounds::at(10),
            "chapter 5 dates the siege to day 10",
            Status::Settled,
        );
        let b = stn.add(constraint(
            "siege",
            "feast",
            Some(3),
            Some(3),
            "chapter 3: feast three days after the siege",
        ));
        let c = stn.pin(
            &feast,
            TimeBounds::at(11),
            "chapter 4 puts the feast on day 11",
            Status::Settled,
        );
        let unrelated = stn.add(constraint(
            "feast",
            "wedding",
            Some(1),
            None,
            "the wedding comes after the feast",
        ));
        match stn.solve() {
            Solution::Inconsistent { cycle } => {
                assert_eq!(cycle, vec![a, b, c]);
                assert!(!cycle.contains(&unrelated));
            }
            Solution::Consistent { .. } => panic!("expected an inconsistency"),
        }
    }

    #[test]
    fn unconstrained_points_are_unbounded() {
        let mut stn = Stn::new();
        stn.add(constraint("a", "b", Some(1), None, "b after a"));
        let solution = stn.solve();
        assert_eq!(
            stn.bounds_of(&solution, &TimePoint::Named { label: "a".into() }),
            TimeBounds::UNBOUNDED
        );
    }
}
