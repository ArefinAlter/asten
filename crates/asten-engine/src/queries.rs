//! The read queries behind the story language server: fact cards, mentions,
//! where a fact was settled, rename previews, scene summaries and cast search.

use asten_diagnostics::Diagnostic;
use asten_ir::{DocId, EntityId, Status, StoryIr, TimeBounds};
use asten_ls::{
    CastSearchHit, FactCard, FactSummary, MentionRef, RenameChange, RenamePreview, SceneSummary,
    WhereSettled,
};
use asten_perceive::BlockPerception;

use crate::ProjectInputs;

fn when(ir: &StoryIr, bounds: &TimeBounds) -> String {
    asten_logic::util::when(ir, bounds)
}

/// `story/factCard`: what holds for an entity, at a scene's time when given.
#[must_use]
pub fn fact_card(
    ir: &StoryIr,
    diagnostics: &[Diagnostic],
    entity: &EntityId,
    at_scene: Option<&DocId>,
) -> Option<FactCard> {
    let found = ir.entity(entity)?;
    let times = asten_logic::resolve_times(ir).0;
    let at = at_scene.map(|document| times.scene(document));
    let mut facts: Vec<FactSummary> = ir
        .facts
        .iter()
        .filter(|fact| fact.fluent.subject == *entity && fact.status.is_active())
        .filter(|fact| at.is_none_or(|bounds| fact.valid.may_overlap(&bounds)))
        .map(|fact| FactSummary {
            id: fact.id.0.clone(),
            predicate: ir
                .predicate(&fact.fluent.predicate)
                .map(|sig| sig.label.clone())
                .unwrap_or_else(|| fact.fluent.predicate.0.replace('_', " ")),
            value: fact.fluent.value.label(),
            status: format!("{:?}", fact.status).to_lowercase(),
            context: fact.context.label(),
            valid_label: if fact.valid.is_unbounded() {
                String::new()
            } else {
                when(ir, &fact.valid)
            },
            settled_in: fact.anchor.as_ref().map(|anchor| anchor.document.clone()),
        })
        .collect();
    facts.sort_by(|a, b| {
        (a.status != "settled")
            .cmp(&(b.status != "settled"))
            .then_with(|| a.predicate.cmp(&b.predicate))
    });
    Some(FactCard {
        entity: entity.clone(),
        name: found.name.clone(),
        sort: Some(found.sort.clone()),
        aliases: found
            .aliases
            .iter()
            .map(|alias| alias.form.clone())
            .collect(),
        facts,
        at_label: at.map(|bounds| when(ir, &bounds)).unwrap_or_default(),
        mention_count: ir
            .mentions
            .iter()
            .filter(|mention| mention.entity == *entity)
            .count(),
        open_marks: diagnostics
            .iter()
            .filter(|mark| mark.entity.as_ref() == Some(entity) && mark.suppressed_by.is_none())
            .count(),
    })
}

/// `story/mentions`.
#[must_use]
pub fn mentions_of(ir: &StoryIr, entity: &EntityId) -> Vec<MentionRef> {
    let mut mentions: Vec<MentionRef> = ir
        .mentions
        .iter()
        .filter(|mention| mention.entity == *entity)
        .map(|mention| MentionRef {
            document: mention.anchor.document.clone(),
            document_title: ir
                .scene(&mention.anchor.document)
                .map(|scene| scene.title.clone())
                .unwrap_or_default(),
            block: mention.anchor.block.clone(),
            start: mention.anchor.start,
            end: mention.anchor.end,
            quote: context_quote(mention),
            reading_pos: mention.reading_pos,
            confidence: mention.confidence,
            context: mention.context.label(),
        })
        .collect();
    mentions.sort_by_key(|mention| (mention.reading_pos, mention.block.0.clone(), mention.start));
    mentions
}

fn context_quote(mention: &asten_ir::Mention) -> String {
    format!(
        "{}{}{}",
        mention.anchor.context_before, mention.anchor.quote, mention.anchor.context_after
    )
}

/// `story/whereSettled`.
#[must_use]
pub fn where_settled(ir: &StoryIr, fact_id: &str) -> Option<WhereSettled> {
    let fact = ir.facts.iter().find(|fact| fact.id.0 == fact_id)?;
    Some(WhereSettled {
        fact: fact.id.0.clone(),
        document: fact.anchor.as_ref().map(|anchor| anchor.document.clone()),
        block: fact.anchor.as_ref().map(|anchor| anchor.block.clone()),
        start: fact.anchor.as_ref().map_or(0, |anchor| anchor.start),
        end: fact.anchor.as_ref().map_or(0, |anchor| anchor.end),
        quote: fact
            .anchor
            .as_ref()
            .map(|anchor| anchor.quote.clone())
            .unwrap_or_default(),
        settled_by: fact
            .settled_at
            .as_ref()
            .map(|settled| settled.actor.clone())
            .unwrap_or_default(),
        settled_at: fact
            .settled_at
            .as_ref()
            .map(|settled| settled.at.clone())
            .unwrap_or_default(),
    })
}

/// `story/rename` preview: every mention of the entity's current name becomes
/// the new name. Aliases are left alone; the author renames those explicitly.
#[must_use]
pub fn rename_preview(
    ir: &StoryIr,
    inputs: &ProjectInputs,
    entity: &EntityId,
    to: &str,
) -> Option<RenamePreview> {
    let found = ir.entity(entity)?;
    let from = found.name.clone();
    let from_key = asten_perceive::matching_form(&from);
    let changes: Vec<RenameChange> = ir
        .mentions
        .iter()
        .filter(|mention| {
            mention.entity == *entity && asten_perceive::matching_form(&mention.surface) == from_key
        })
        .filter_map(|mention| {
            let block = inputs
                .blocks
                .iter()
                .find(|block| block.block == mention.anchor.block)?;
            let chars: Vec<char> = block.text.chars().collect();
            let before: String = chars
                .get(mention.anchor.start as usize..mention.anchor.end as usize)?
                .iter()
                .collect();
            // Only exact-form occurrences are renamed; inflected forms are left for the author.
            if asten_perceive::matching_form(&before) != from_key {
                return None;
            }
            let after = match_case(&before, to);
            Some(RenameChange {
                document: mention.anchor.document.clone(),
                document_title: ir
                    .scene(&mention.anchor.document)
                    .map(|scene| scene.title.clone())
                    .unwrap_or_default(),
                block: mention.anchor.block.clone(),
                start: mention.anchor.start,
                end: mention.anchor.end,
                before,
                after,
            })
        })
        .collect();
    Some(RenamePreview {
        entity: entity.clone(),
        from,
        to: to.to_owned(),
        changes,
    })
}

/// Keep an all-caps or all-lowercase occurrence in its case.
fn match_case(before: &str, to: &str) -> String {
    if before.chars().all(|c| !c.is_lowercase()) && before.chars().any(char::is_alphabetic) {
        to.to_uppercase()
    } else if before.chars().all(|c| !c.is_uppercase()) {
        to.to_lowercase()
    } else {
        to.to_owned()
    }
}

/// `story/sceneSummary`.
#[must_use]
pub fn scene_summary(
    ir: &StoryIr,
    inputs: &ProjectInputs,
    perceptions: &[BlockPerception],
    diagnostics: &[Diagnostic],
    document: &DocId,
) -> Option<SceneSummary> {
    let scene = ir.scene(document)?;
    let times = asten_logic::resolve_times(ir).0;
    let bounds = times.scene(document);
    let blocks: Vec<&crate::BlockInput> = inputs
        .blocks
        .iter()
        .filter(|block| block.document == *document)
        .collect();
    let mentions: Vec<&asten_ir::Mention> = ir
        .mentions
        .iter()
        .filter(|mention| mention.anchor.document == *document)
        .collect();
    let mut entities: Vec<EntityId> = mentions
        .iter()
        .map(|mention| mention.entity.clone())
        .collect();
    entities.sort();
    entities.dedup();
    let script = perceptions
        .iter()
        .filter(|perception| {
            blocks
                .iter()
                .any(|block| block.fingerprint == perception.fingerprint)
        })
        .map(|perception| perception.script.clone())
        .find(|script| script != "Zyyy")
        .unwrap_or_else(|| "Zyyy".into());
    Some(SceneSummary {
        document: document.clone(),
        title: scene.title.clone(),
        reading_pos: scene.reading_pos,
        time_label: scene.time_label.clone(),
        bounds,
        bounds_label: when(ir, &bounds),
        context: scene.context.label(),
        continuity_run: scene.continuity_run.clone(),
        mentions: mentions.len(),
        entities,
        events: ir.events.iter().filter(|event| event.document.as_ref() == Some(document) || matches!(&event.at, asten_ir::TimeRef::Scene { document: d } if d == document)).count(),
        observations: ir.observations.iter().filter(|observation| observation.anchor.document == *document).count(),
        marks: diagnostics
            .iter()
            .filter(|mark| mark.suppressed_by.is_none())
            .filter(|mark| mark.primary.as_ref().is_some_and(|location| location.document == *document) || mark.related.iter().any(|(location, _)| location.document == *document))
            .count(),
        tier: format!("{:?}", asten_perceive::tier_for(&script)),
        script,
        words: blocks.iter().map(|block| block.text.split_whitespace().count()).sum(),
    })
}

/// `story/castSearch`: fuzzy search over names and aliases.
#[must_use]
pub fn cast_search(ir: &StoryIr, query: &str, limit: usize) -> Vec<CastSearchHit> {
    let needle = asten_perceive::matching_form(query);
    if needle.is_empty() {
        return Vec::new();
    }
    let mut hits: Vec<CastSearchHit> = ir
        .entities
        .iter()
        .filter_map(|entity| {
            let score = entity
                .surface_forms()
                .map(|form| {
                    let folded = asten_perceive::matching_form(form);
                    if folded == needle {
                        1.0
                    } else if folded.starts_with(&needle) {
                        0.9
                    } else if folded.contains(&needle) {
                        0.7
                    } else {
                        1.0 - asten_logic::symbols::name_distance(&folded, &needle)
                    }
                })
                .fold(0.0_f32, f32::max);
            (score >= 0.6).then(|| CastSearchHit {
                entity: entity.id.clone(),
                name: entity.name.clone(),
                sort: entity.sort.clone(),
                score,
                mentions: ir
                    .mentions
                    .iter()
                    .filter(|mention| mention.entity == entity.id)
                    .count(),
            })
        })
        .collect();
    hits.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| b.mentions.cmp(&a.mentions))
    });
    hits.truncate(limit);
    hits
}

/// Facts a repair settles or retires, by status filter, for the proposals queue.
#[must_use]
pub fn proposals(ir: &StoryIr) -> Vec<&asten_ir::Fact> {
    ir.facts
        .iter()
        .filter(|fact| fact.status == Status::Proposed)
        .collect()
}
