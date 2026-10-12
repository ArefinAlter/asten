//! Shared helpers for building statements and locations.

use asten_diagnostics::{Location, Statement, StatementKind};
use asten_ir::{Anchor, Event, Fact, Observation, Scene, Status, StoryIr, TimeBounds};
use asten_time::{Clock, TimeUnit};

/// A location from an anchor, labelled with the scene title when known.
#[must_use]
pub fn location_of(ir: &StoryIr, anchor: &Anchor) -> Location {
    let mut location = Location::from_anchor(anchor);
    location.label = ir
        .scene(&anchor.document)
        .map(|scene| scene.title.clone())
        .unwrap_or_default();
    location
}

#[must_use]
pub fn scene_location(scene: &Scene) -> Location {
    Location::document(scene.document.clone(), scene.title.clone())
}

/// Format bounds in the project's day-count clock for explanations.
#[must_use]
pub fn when(ir: &StoryIr, bounds: &TimeBounds) -> String {
    let unit = match ir.settings.ticks_per_day {
        1440 => TimeUnit::Minute,
        24 => TimeUnit::Hour,
        _ => TimeUnit::Day,
    };
    let clock = Clock::new(unit, asten_time::calendar::Calendar::DayCount);
    if bounds.is_unbounded() {
        return "an unplaced time".into();
    }
    clock.format_bounds(bounds).to_lowercase()
}

#[must_use]
pub fn fact_statement(ir: &StoryIr, fact: &Fact) -> Statement {
    let where_settled = fact
        .anchor
        .as_ref()
        .and_then(|anchor| ir.scene(&anchor.document))
        .map(|scene| format!(" in {}", scene.title))
        .unwrap_or_default();
    let verb = match fact.status {
        Status::Settled => "Settled",
        Status::Proposed => "Proposed",
        Status::Deprecated => "Deprecated",
        Status::Rejected => "Rejected",
    };
    let validity = if fact.valid.is_unbounded() {
        String::new()
    } else {
        format!(", {}", when(ir, &fact.valid))
    };
    let text = format!(
        "{verb}{where_settled}: {}{validity}",
        fluent_label(ir, &fact.fluent)
    );
    let prior = Statement::prior_from(
        &fact.provenance,
        fact.status,
        &fact.context,
        fact.confidence,
    );
    Statement::new(StatementKind::Fact, fact.id.0.clone(), text, fact.status)
        .at(fact.anchor.as_ref().map(|anchor| location_of(ir, anchor)))
        .with_prior(prior)
        .with_confidence(fact.confidence)
}

#[must_use]
pub fn observation_statement(ir: &StoryIr, observation: &Observation) -> Statement {
    let scene = ir
        .scene(&observation.anchor.document)
        .map(|scene| scene.title.clone())
        .unwrap_or_else(|| "the page".into());
    let quote = if observation.anchor.quote.is_empty() {
        fluent_label(ir, &observation.fluent)
    } else {
        format!("\u{201c}{}\u{201d}", observation.anchor.quote)
    };
    let context = match &observation.context {
        asten_ir::Context::Narration | asten_ir::Context::World => String::new(),
        other => format!(" ({})", other.label()),
    };
    let text = format!("{scene} says: {quote}{context}");
    let prior = Statement::prior_from(
        &observation.provenance,
        Status::Proposed,
        &observation.context,
        observation.confidence,
    );
    Statement::new(
        StatementKind::Observation,
        observation.id.0.clone(),
        text,
        Status::Proposed,
    )
    .at(Some(location_of(ir, &observation.anchor)))
    .with_prior(prior)
    .with_confidence(observation.confidence)
}

#[must_use]
pub fn event_statement(ir: &StoryIr, event: &Event, bounds: &TimeBounds) -> Statement {
    let who = event
        .subject
        .as_ref()
        .and_then(|id| ir.entity(id))
        .map(|entity| entity.name.clone())
        .or_else(|| event.subject.as_ref().map(|id| id.0.clone()))
        .unwrap_or_default();
    let label = if event.label.is_empty() {
        format!("{who} {}", event.kind.label())
    } else {
        event.label.clone()
    };
    let text = format!("{label} ({})", when(ir, bounds));
    let location = event
        .anchor
        .as_ref()
        .map(|anchor| location_of(ir, anchor))
        .or_else(|| {
            event
                .document
                .as_ref()
                .and_then(|document| ir.scene(document))
                .map(scene_location)
        });
    let prior = Statement::prior_from(
        &event.provenance,
        event.status,
        &event.context,
        event.confidence,
    );
    Statement::new(StatementKind::Event, event.id.0.clone(), text, event.status)
        .at(location)
        .with_prior(prior)
        .with_confidence(event.confidence)
}

#[must_use]
pub fn scene_statement(ir: &StoryIr, scene: &Scene, bounds: &TimeBounds) -> Statement {
    let placed = if scene.time_label.is_empty() {
        when(ir, bounds)
    } else {
        format!("{} ({})", scene.time_label, when(ir, bounds))
    };
    Statement::new(
        StatementKind::Scene,
        format!("scene:{}", scene.document.0),
        format!("{}: placed at {placed}", scene.title),
        Status::Settled,
    )
    .at(Some(scene_location(scene)))
    .with_prior(0.05)
}

/// The product of confidences, as the probability every statement is right.
#[must_use]
pub fn joint_confidence(statements: &[Statement]) -> f32 {
    statements
        .iter()
        .map(|statement| statement.confidence)
        .product::<f32>()
        .clamp(0.0, 1.0)
}

/// Pairing from the statuses in a conflict set.
#[must_use]
pub fn pairing_of(statements: &[Statement]) -> asten_diagnostics::Pairing {
    let settled = statements
        .iter()
        .filter(|statement| statement.status == Status::Settled)
        .count();
    if settled == statements.len() {
        asten_diagnostics::Pairing::SettledSettled
    } else if settled > 0 {
        asten_diagnostics::Pairing::SettledObserved
    } else {
        asten_diagnostics::Pairing::ProposedProposed
    }
}

/// Entity display name.
#[must_use]
pub fn name_of(ir: &StoryIr, id: &asten_ir::EntityId) -> String {
    ir.entity(id)
        .map(|entity| entity.name.clone())
        .unwrap_or_else(|| id.0.clone())
}

/// A fluent as the author reads it: `Mara · eyes · green`, with the entity's
/// display name and the predicate's label instead of their internal keys.
#[must_use]
pub fn fluent_label(ir: &StoryIr, fluent: &asten_ir::Fluent) -> String {
    let predicate = ir
        .predicate(&fluent.predicate)
        .map(|sig| sig.label.clone())
        .filter(|label| !label.is_empty())
        .unwrap_or_else(|| fluent.predicate.0.clone());
    let value = match &fluent.value {
        asten_ir::Value::Entity { id } => name_of(ir, id),
        other => other.label(),
    };
    format!("{} · {predicate} · {value}", name_of(ir, &fluent.subject))
}
