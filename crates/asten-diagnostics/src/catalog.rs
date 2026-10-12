//! The diagnostic catalogue. Codes are namespaced, stable forever, and never
//! reused (§12.2). CANON-E201 changed meaning deliberately: it now fires only
//! where the author declared a continuous run, and E202 covers genuine
//! temporal inconsistency.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Area {
    Attributes,
    Lifecycles,
    Time,
    Space,
    Knowledge,
    Relations,
    Rules,
    Names,
    Ledger,
    Craft,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CodeInfo {
    pub code: &'static str,
    pub area: Area,
    pub title: &'static str,
    pub summary: &'static str,
    /// Which pack owns the rule. Core rules say "scribe".
    pub pack: &'static str,
}

pub const ATTRIBUTE_CONTRADICTION: &str = "CANON-E013";
pub const IMMUTABLE_CHANGED: &str = "CANON-E014";
pub const SORT_MISMATCH: &str = "CANON-E015";
pub const LIFECYCLE_AFTER_DEATH: &str = "CANON-E101";
pub const BEFORE_BIRTH: &str = "CANON-E102";
pub const DESTROYED_REAPPEARS: &str = "CANON-E103";
pub const CONTINUITY_RUN: &str = "CANON-E201";
pub const TEMPORAL_INCONSISTENCY: &str = "CANON-E202";
pub const EFFECT_BEFORE_CAUSE: &str = "CANON-E203";
pub const IMPOSSIBLE_JOURNEY: &str = "CANON-E301";
pub const TWO_PLACES: &str = "CANON-E302";
pub const KNOWLEDGE_LEAK: &str = "CANON-E401";
pub const GENEALOGY_CYCLE: &str = "CANON-E501";
pub const EXCLUSIVE_OVERLAP: &str = "CANON-E502";
pub const PARENT_AGE: &str = "CANON-E503";
pub const INVARIANT_VIOLATED: &str = "CANON-E601";
pub const PACK_RULE: &str = "CANON-E602";
pub const UNKNOWN_NAME: &str = "NAME-W001";
pub const USED_BEFORE_INTRODUCTION: &str = "NAME-W002";
pub const CONFUSABLE_NAMES: &str = "NAME-W003";
pub const DANGLING_REFERENCE: &str = "NAME-W004";
pub const PENDING_SETUP: &str = "PLOT-I001";
pub const UNUSED_SETUP: &str = "PLOT-I002";

const CATALOG: &[CodeInfo] = &[
    CodeInfo {
        code: ATTRIBUTE_CONTRADICTION,
        area: Area::Attributes,
        title: "Attribute contradiction",
        summary: "Two values of a one-valued attribute hold at the same story time in the same context, and no change event explains it.",
        pack: "scribe",
    },
    CodeInfo {
        code: IMMUTABLE_CHANGED,
        area: Area::Attributes,
        title: "Immutable attribute changed",
        summary: "A change event alters an attribute declared immutable (a birthplace).",
        pack: "scribe",
    },
    CodeInfo {
        code: SORT_MISMATCH,
        area: Area::Attributes,
        title: "Attribute on the wrong kind of thing",
        summary: "A predicate is attached to an entity whose sort it does not accept.",
        pack: "scribe",
    },
    CodeInfo {
        code: LIFECYCLE_AFTER_DEATH,
        area: Area::Lifecycles,
        title: "Acts after a settled death",
        summary: "A character acts in the world after the story time of a settled death, with no return.",
        pack: "lifecycle",
    },
    CodeInfo {
        code: BEFORE_BIRTH,
        area: Area::Lifecycles,
        title: "Appears before birth",
        summary: "An entity acts before the story time it is born or created.",
        pack: "lifecycle",
    },
    CodeInfo {
        code: DESTROYED_REAPPEARS,
        area: Area::Lifecycles,
        title: "Destroyed and present",
        summary: "An item or place is used after it was destroyed and before any repair.",
        pack: "lifecycle",
    },
    CodeInfo {
        code: CONTINUITY_RUN,
        area: Area::Time,
        title: "Story time runs backward inside a continuous run",
        summary: "Scenes the author declared continuous step backward in story time.",
        pack: "clocks",
    },
    CodeInfo {
        code: TEMPORAL_INCONSISTENCY,
        area: Area::Time,
        title: "The dates cannot all be true",
        summary: "The story-time constraints form a cycle that no placement satisfies. The listed statements are the cycle.",
        pack: "clocks",
    },
    CodeInfo {
        code: EFFECT_BEFORE_CAUSE,
        area: Area::Time,
        title: "Effect before cause",
        summary: "A state is observed before the event that brings it about.",
        pack: "clocks",
    },
    CodeInfo {
        code: IMPOSSIBLE_JOURNEY,
        area: Area::Space,
        title: "The journey does not fit the time",
        summary: "An entity appears in two places closer in story time than the shortest route allows.",
        pack: "atlas",
    },
    CodeInfo {
        code: TWO_PLACES,
        area: Area::Space,
        title: "In two places at once",
        summary: "An entity is placed at two places at the same story time.",
        pack: "atlas",
    },
    CodeInfo {
        code: KNOWLEDGE_LEAK,
        area: Area::Knowledge,
        title: "Knows it before learning it",
        summary: "A character uses knowledge no event has given them yet.",
        pack: "knowledge",
    },
    CodeInfo {
        code: GENEALOGY_CYCLE,
        area: Area::Relations,
        title: "Their own ancestor",
        summary: "An acyclic relation (ancestor, contains) loops back on itself.",
        pack: "relations",
    },
    CodeInfo {
        code: EXCLUSIVE_OVERLAP,
        area: Area::Relations,
        title: "Two of an exclusive relation at once",
        summary: "Two relations declared exclusive overlap in story time.",
        pack: "relations",
    },
    CodeInfo {
        code: PARENT_AGE,
        area: Area::Relations,
        title: "Parent too young",
        summary: "A parent's birth is too close to the child's for the project's minimum parent age.",
        pack: "relations",
    },
    CodeInfo {
        code: INVARIANT_VIOLATED,
        area: Area::Rules,
        title: "An author invariant is broken",
        summary: "A rule the author wrote in the invariants panel fails on the scene trace.",
        pack: "invariants",
    },
    CodeInfo {
        code: PACK_RULE,
        area: Area::Rules,
        title: "A pack rule is broken",
        summary: "A world rule from an enabled pack fails.",
        pack: "packs",
    },
    CodeInfo {
        code: UNKNOWN_NAME,
        area: Area::Names,
        title: "A name not in the cast",
        summary: "A span that looks like a name links to no entity.",
        pack: "scribe",
    },
    CodeInfo {
        code: USED_BEFORE_INTRODUCTION,
        area: Area::Names,
        title: "Named before being introduced",
        summary: "An entity is mentioned before the scene the author marked as its introduction.",
        pack: "scribe",
    },
    CodeInfo {
        code: CONFUSABLE_NAMES,
        area: Area::Names,
        title: "Confusable names",
        summary: "Two cast names are close enough to be misread.",
        pack: "scribe",
    },
    CodeInfo {
        code: DANGLING_REFERENCE,
        area: Area::Names,
        title: "Points at a removed entity",
        summary: "A fact or event refers to an entity that no longer exists.",
        pack: "scribe",
    },
    CodeInfo {
        code: PENDING_SETUP,
        area: Area::Ledger,
        title: "Not yet paid off",
        summary: "A setup the author marked for payoff has no payoff yet. A ledger entry, not an error.",
        pack: "invariants",
    },
    CodeInfo {
        code: UNUSED_SETUP,
        area: Area::Ledger,
        title: "Set up and never used",
        summary: "A planted element has no payoff by the end of a manuscript marked complete.",
        pack: "invariants",
    },
];

#[must_use]
pub fn catalog() -> &'static [CodeInfo] {
    CATALOG
}

#[must_use]
pub fn info(code: &str) -> Option<&'static CodeInfo> {
    CATALOG.iter().find(|entry| entry.code == code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_are_unique() {
        let mut codes: Vec<&str> = CATALOG.iter().map(|entry| entry.code).collect();
        let total = codes.len();
        codes.sort_unstable();
        codes.dedup();
        assert_eq!(codes.len(), total);
        assert_eq!(info("CANON-E013").unwrap().area, Area::Attributes);
    }
}
