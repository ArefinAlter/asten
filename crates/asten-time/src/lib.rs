//! Story time (§9.6 of the engine design).
//!
//! Two clocks are first-class: reading order is a total order on blocks, and
//! story time is integer ticks in a project-chosen base unit. A `Clock` maps
//! human labels to ticks through a calendar, and the simple temporal network in
//! [`stn`] decides whether a set of difference constraints is consistent and,
//! when it is not, which constraints form the negative cycle that explains it.

pub mod calendar;
pub mod stn;

pub use calendar::{Clock, Granularity, Parsed, Relation, TimeUnit};
pub use stn::{Solution, Stn};
