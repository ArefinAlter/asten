//! Reading order against story time (§9.6). The two clocks are independent by
//! default: a flashback is not an error. The engine reports a clock problem only
//! inside a run the author declared continuous (E201) or when a state is
//! observed before the event that causes it (E203). E202 lives in `time.rs`.

use std::collections::BTreeMap;

use asten_diagnostics::{Pairing, Repair, RepairKind, Tier, Violation, catalog};
use asten_ir::StoryIr;

use crate::state::{families, initiated};
use crate::time::TimeTable;
use crate::util::{
    event_statement, joint_confidence, location_of, name_of, observation_statement, pairing_of,
    scene_location, scene_statement,
};

/// CANON-E201 and E203.
#[must_use]
pub fn check(ir: &StoryIr, times: &TimeTable) -> Vec<Violation> {
    let mut violations = Vec::new();

    // E201: continuity runs.
    let mut runs: BTreeMap<&str, Vec<&asten_ir::Scene>> = BTreeMap::new();
    for scene in &ir.scenes {
        if let Some(run) = &scene.continuity_run {
            runs.entry(run.as_str()).or_default().push(scene);
        }
    }
    for (run, mut scenes) in runs {
        scenes.sort_by_key(|scene| scene.reading_pos);
        for pair in scenes.windows(2) {
            let (earlier, later) = (pair[0], pair[1]);
            let a = times.scene(&earlier.document);
            let b = times.scene(&later.document);
            let tier = match (b.hi, a.lo) {
                (Some(b_hi), Some(a_lo)) if b_hi < a_lo => Tier::Must,
                _ => match (b.lo, a.lo, b.hi, a.hi) {
                    (Some(b_lo), Some(a_lo), Some(b_hi), Some(a_hi))
                        if b_lo < a_lo && b_hi < a_hi =>
                    {
                        Tier::May
                    }
                    _ => continue,
                },
            };
            let conflict = vec![
                scene_statement(ir, earlier, &a),
                scene_statement(ir, later, &b),
            ];
            violations.push(Violation {
                code: catalog::CONTINUITY_RUN.into(),
                title: format!("Story time runs backward inside the continuous run \u{201c}{run}\u{201d}."),
                primary: Some(scene_location(later)),
                related: vec![(scene_location(earlier), "the earlier scene".into())],
                conflict,
                tier,
                pairing: Pairing::SettledSettled,
                confidence: 1.0,
                repairs: vec![
                    Repair::new(RepairKind::PlaceScene { document: later.document.clone(), lo: a.lo, hi: None }, "Set the scene's time", 1.0),
                    Repair::new(RepairKind::ClearContinuity { document: later.document.clone() }, "This scene is not part of the run", 1.5),
                    Repair::suppress(),
                    Repair::keep(),
                ],
                entity: None,
                details: "You declared these scenes continuous. Reorder them, redate one, or take this scene out of the run.".into(),
                severity_override: None,
            });
        }
    }

    // E203: an observed state before the only event that brings it about.
    let groups = families(ir, times);
    for observation in &ir.observations {
        if observation.negated {
            continue;
        }
        let key = (
            observation.fluent.family(),
            observation
                .context
                .comparison_key(ir.settings.narrator_reliable),
        );
        let Some(family) = groups.get(&key) else {
            continue;
        };
        let at = match &observation.at {
            asten_ir::TimeRef::Unknown => times.scene(&observation.anchor.document),
            other => times.of(other),
        };
        if at.is_unbounded() {
            continue;
        }
        // Sources that assert the observed value.
        let supporting: Vec<_> = family
            .sources
            .iter()
            .filter(|source| {
                ir.compatible(
                    &observation.fluent.predicate,
                    &source.value,
                    &observation.fluent.value,
                )
            })
            .collect();
        if supporting.is_empty() || supporting.iter().any(|source| source.fact.is_some()) {
            // Facts need no cause; only event-initiated values can be observed too early.
            continue;
        }
        let earliest = supporting
            .iter()
            .min_by_key(|source| source.may.lo.unwrap_or(i64::MIN))
            .copied();
        let Some(source) = earliest else { continue };
        let Some(event) = source.event else { continue };
        if !initiated(event)
            .iter()
            .any(|fluent| fluent.family() == observation.fluent.family())
        {
            continue;
        }
        let cause = times.event(&event.id);
        let tier = match (at.hi, cause.lo) {
            (Some(obs_hi), Some(cause_lo)) if obs_hi < cause_lo => Tier::Must,
            _ => match (at.lo, cause.lo, at.hi, cause.hi) {
                (Some(obs_lo), Some(cause_lo), Some(obs_hi), Some(cause_hi))
                    if obs_lo < cause_lo && obs_hi < cause_hi =>
                {
                    Tier::May
                }
                _ => continue,
            },
        };
        let conflict = vec![
            event_statement(ir, event, &cause),
            observation_statement(ir, observation),
        ];
        let who = name_of(ir, &observation.fluent.subject);
        violations.push(Violation {
            code: catalog::EFFECT_BEFORE_CAUSE.into(),
            title: format!(
                "{who}'s {} is {} before the event that makes it so.",
                observation.fluent.predicate.0.replace('_', " "),
                observation.fluent.value.label()
            ),
            primary: Some(location_of(ir, &observation.anchor)),
            related: conflict[0]
                .location
                .clone()
                .map(|location| (location, "the cause".to_owned()))
                .into_iter()
                .collect(),
            confidence: joint_confidence(&conflict),
            pairing: pairing_of(&conflict),
            conflict,
            tier,
            repairs: vec![
                Repair::new(
                    RepairKind::MoveEvent {
                        event: event.id.0.clone(),
                        lo: None,
                        hi: at.lo.map(|lo| lo - 1),
                    },
                    "Move the event earlier",
                    1.0,
                ),
                Repair::new(
                    RepairKind::PlaceScene {
                        document: observation.anchor.document.clone(),
                        lo: cause.hi.map(|hi| hi + 1),
                        hi: None,
                    },
                    "Set the scene's time",
                    1.5,
                ),
                Repair::suppress(),
                Repair::keep(),
            ],
            entity: Some(observation.fluent.subject.clone()),
            details: String::new(),
            severity_override: None,
        });
    }
    violations
}

#[cfg(test)]
mod tests {
    use super::*;
    use asten_ir::{Context, DocId, Scene, TimeRef};

    fn scene(document: &str, pos: i64, tick: i64, run: Option<&str>) -> Scene {
        Scene {
            document: DocId::new(document),
            title: format!("Scene {pos}"),
            reading_pos: pos,
            at: TimeRef::Tick { tick },
            time_label: String::new(),
            context: Context::Narration,
            continuity_run: run.map(str::to_owned),
            language: None,
            blocks: Vec::new(),
        }
    }

    #[test]
    fn flashbacks_are_fine_unless_declared_continuous() {
        let mut ir = StoryIr::default();
        ir.scenes.push(scene("a", 1, 10, None));
        ir.scenes.push(scene("b", 2, 4, None));
        let (times, _) = crate::resolve_times(&ir);
        assert!(check(&ir, &times).is_empty());

        ir.scenes[0].continuity_run = Some("one night".into());
        ir.scenes[1].continuity_run = Some("one night".into());
        let (times, _) = crate::resolve_times(&ir);
        let violations = check(&ir, &times);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].code, "CANON-E201");
        assert_eq!(violations[0].tier, Tier::Must);
    }
}
