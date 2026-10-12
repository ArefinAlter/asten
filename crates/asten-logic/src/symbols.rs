//! Names and symbols (§9.1): the cheapest and most IDE-like layer.

use asten_diagnostics::{
    Pairing, Repair, RepairKind, Severity, Statement, StatementKind, Tier, Violation, catalog,
};
use asten_ir::{Sort, Status, StoryIr};
use unicode_segmentation::UnicodeSegmentation;

use crate::util::location_of;

/// NAME-W001 to W004.
#[must_use]
pub fn check(ir: &StoryIr) -> Vec<Violation> {
    let mut violations = Vec::new();

    // W001: unknown names, one mark per surface form, at its first occurrence.
    let mut seen: Vec<String> = Vec::new();
    let mut unknown: Vec<&(asten_ir::Anchor, String, i64)> = ir.unknown_names.iter().collect();
    unknown.sort_by_key(|(_, _, pos)| *pos);
    for (anchor, surface, pos) in unknown {
        let key = surface.to_lowercase();
        if seen.contains(&key) {
            continue;
        }
        seen.push(key);
        let count = ir
            .unknown_names
            .iter()
            .filter(|(_, other, _)| other.eq_ignore_ascii_case(surface))
            .count();
        let statement = Statement::new(
            StatementKind::Mention,
            format!("unknown:{surface}:{pos}"),
            format!(
                "\u{201c}{surface}\u{201d} appears {count} time{} and is not in the cast.",
                if count == 1 { "" } else { "s" }
            ),
            Status::Proposed,
        )
        .at(Some(location_of(ir, anchor)))
        .with_prior(0.5)
        .with_confidence(0.6);
        violations.push(Violation {
            code: catalog::UNKNOWN_NAME.into(),
            title: format!("Add {surface} to the cast?"),
            primary: Some(location_of(ir, anchor)),
            related: Vec::new(),
            conflict: vec![statement],
            tier: Tier::May,
            pairing: Pairing::ProposedProposed,
            confidence: 0.6,
            repairs: vec![
                Repair::new(RepairKind::AddEntity { name: surface.clone(), sort: Sort::Character }, format!("Add {surface} as a character"), 1.0),
                Repair::new(RepairKind::AddEntity { name: surface.clone(), sort: Sort::Place }, format!("Add {surface} as a place"), 1.2),
                Repair::suppress(),
                Repair::keep(),
            ],
            entity: None,
            details: "A capitalised name that links to no cast entry. Adding it lets the engine track it.".into(),
            severity_override: Some(Severity::Information),
        });
    }

    // W002: used before introduction.
    for entity in &ir.entities {
        let Some(introduced) = entity.introduced_at else {
            continue;
        };
        let Some(first) = ir
            .mentions
            .iter()
            .filter(|mention| mention.entity == entity.id && mention.reading_pos < introduced)
            .min_by_key(|mention| (mention.reading_pos, mention.anchor.start))
        else {
            continue;
        };
        let statement = Statement::new(
            StatementKind::Mention,
            format!("early:{}:{}", entity.id.0, first.anchor.block.0),
            format!(
                "{} is named here, before the scene that introduces them.",
                entity.name
            ),
            Status::Proposed,
        )
        .at(Some(location_of(ir, &first.anchor)))
        .with_prior(0.3)
        .with_confidence(first.confidence);
        violations.push(Violation {
            code: catalog::USED_BEFORE_INTRODUCTION.into(),
            title: format!("{} appears before being introduced.", entity.name),
            primary: Some(location_of(ir, &first.anchor)),
            related: Vec::new(),
            conflict: vec![statement],
            tier: Tier::May,
            pairing: Pairing::SettledObserved,
            confidence: first.confidence,
            repairs: vec![Repair::suppress(), Repair::keep()],
            entity: Some(entity.id.clone()),
            details: "Readers meet the name before the introduction you marked. Fine for a teaser; worth a look otherwise.".into(),
            severity_override: Some(Severity::Information),
        });
    }

    // W003: confusable names.
    for (index, left) in ir.entities.iter().enumerate() {
        for right in ir.entities.iter().skip(index + 1) {
            if left.name.eq_ignore_ascii_case(&right.name) {
                continue;
            }
            let distance = name_distance(&left.name, &right.name);
            if distance > 0.2 {
                continue;
            }
            let statement = Statement::new(
                StatementKind::Rule,
                format!("confusable:{}:{}", left.id.0, right.id.0),
                format!(
                    "{} and {} are {:.0}% alike.",
                    left.name,
                    right.name,
                    (1.0 - distance) * 100.0
                ),
                Status::Settled,
            )
            .with_prior(0.5);
            violations.push(Violation {
                code: catalog::CONFUSABLE_NAMES.into(),
                title: format!("{} and {} may be confused.", left.name, right.name),
                primary: None,
                related: Vec::new(),
                conflict: vec![statement],
                tier: Tier::May,
                pairing: Pairing::SettledSettled,
                confidence: 1.0,
                repairs: vec![Repair::suppress(), Repair::keep()],
                entity: Some(left.id.clone()),
                details: "Measured on grapheme clusters with a Jaro–Winkler similarity, so the check works in any script.".into(),
                severity_override: Some(Severity::Hint),
            });
        }
    }

    // W004: dangling references.
    for fact in ir.facts.iter().filter(|fact| fact.status.is_active()) {
        if ir.entity(&fact.fluent.subject).is_none() {
            let statement = crate::util::fact_statement(ir, fact);
            violations.push(Violation {
                code: catalog::DANGLING_REFERENCE.into(),
                title: format!(
                    "\u{201c}{}\u{201d} is not in the cast.",
                    fact.fluent.subject.0
                ),
                primary: statement.location.clone(),
                related: Vec::new(),
                conflict: vec![statement],
                tier: Tier::Must,
                pairing: Pairing::SettledSettled,
                confidence: 1.0,
                repairs: vec![
                    Repair::new(
                        RepairKind::AddEntity {
                            name: fact.fluent.subject.0.replace('_', " "),
                            sort: Sort::Character,
                        },
                        "Add to the cast",
                        1.0,
                    ),
                    Repair::new(
                        RepairKind::DeprecateFact {
                            fact: fact.id.0.clone(),
                        },
                        "Retire the fact",
                        2.0,
                    ),
                    Repair::keep(),
                ],
                entity: Some(fact.fluent.subject.clone()),
                details: String::new(),
                severity_override: Some(Severity::Information),
            });
        }
    }
    violations
}

/// A script-aware name distance in `[0, 1]`: 1 − Jaro–Winkler over grapheme
/// clusters of the case-folded names. 0 means identical.
#[must_use]
pub fn name_distance(a: &str, b: &str) -> f32 {
    let a: Vec<String> = a
        .to_lowercase()
        .graphemes(true)
        .map(str::to_owned)
        .collect();
    let b: Vec<String> = b
        .to_lowercase()
        .graphemes(true)
        .map(str::to_owned)
        .collect();
    let a: Vec<&str> = a.iter().map(String::as_str).collect();
    let b: Vec<&str> = b.iter().map(String::as_str).collect();
    1.0 - jaro_winkler(&a, &b)
}

fn jaro_winkler(a: &[&str], b: &[&str]) -> f32 {
    if a.is_empty() && b.is_empty() {
        return 1.0;
    }
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }
    let window = (a.len().max(b.len()) / 2).saturating_sub(1);
    let mut a_matched = vec![false; a.len()];
    let mut b_matched = vec![false; b.len()];
    let mut matches = 0usize;
    for (i, ga) in a.iter().enumerate() {
        let lo = i.saturating_sub(window);
        let hi = (i + window + 1).min(b.len());
        for j in lo..hi {
            if !b_matched[j] && b[j] == *ga {
                a_matched[i] = true;
                b_matched[j] = true;
                matches += 1;
                break;
            }
        }
    }
    if matches == 0 {
        return 0.0;
    }
    let mut transpositions = 0usize;
    let mut k = 0usize;
    for (i, matched) in a_matched.iter().enumerate() {
        if !matched {
            continue;
        }
        while !b_matched[k] {
            k += 1;
        }
        if a[i] != b[k] {
            transpositions += 1;
        }
        k += 1;
    }
    let m = matches as f32;
    let jaro =
        (m / a.len() as f32 + m / b.len() as f32 + (m - transpositions as f32 / 2.0) / m) / 3.0;
    let prefix = a
        .iter()
        .zip(b.iter())
        .take(4)
        .take_while(|(x, y)| x == y)
        .count() as f32;
    jaro + prefix * 0.1 * (1.0 - jaro)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distance_is_script_aware() {
        assert!(name_distance("Mira", "Mira") < 0.01);
        assert!(name_distance("Mira", "Mina") < 0.2);
        assert!(name_distance("Mira", "Oren") >= 0.5);
        assert!(name_distance("মিরা", "মিনা") < 0.35);
        assert!(name_distance("Aric", "Arik") < 0.2);
    }
}
