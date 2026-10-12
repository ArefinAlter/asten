//! The symbolic analyses: the type checker of the canon engine (§9).
//!
//! Every analysis reads the Story IR and emits candidate violations with their
//! minimal conflict sets. Given the same IR they produce the same results on
//! every machine. They are exact over settled facts; the confidence numbers
//! they carry come from perception and are combined, never judged, here.

use asten_diagnostics::Violation;
use asten_ir::StoryIr;

pub mod clocks;
pub mod invariants;
pub mod knowledge;
pub mod lifecycle;
pub mod relations;
pub mod space;
pub mod state;
pub mod symbols;
pub mod time;
pub mod util;

pub use time::{TimeTable, resolve_times};

/// The output of one full analysis pass.
#[derive(Clone, Debug, Default)]
pub struct Report {
    pub violations: Vec<Violation>,
    pub times: TimeTable,
    /// Pending obligations from author invariants (the setup/payoff ledger).
    pub ledger: Vec<invariants::Obligation>,
}

/// Run every analysis over the IR.
#[must_use]
pub fn analyze(ir: &StoryIr) -> Report {
    let (times, mut violations) = resolve_times(ir);
    violations.extend(state::check(ir, &times));
    violations.extend(lifecycle::check(ir, &times));
    violations.extend(clocks::check(ir, &times));
    violations.extend(symbols::check(ir));
    violations.extend(relations::check(ir, &times));
    violations.extend(knowledge::check(ir, &times));
    violations.extend(space::check(ir, &times));
    let (invariant_violations, ledger) = invariants::check(ir, &times);
    violations.extend(invariant_violations);
    // Deterministic order: by code, then by stable id.
    violations.sort_by(|a, b| {
        a.code
            .cmp(&b.code)
            .then_with(|| a.stable_id().cmp(&b.stable_id()))
    });
    violations
        .dedup_by(|a, b| a.code == b.code && a.evidence_fingerprint() == b.evidence_fingerprint());
    Report {
        violations,
        times,
        ledger,
    }
}
