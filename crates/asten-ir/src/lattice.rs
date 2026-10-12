//! Concept lattices (§9.2). Values are concepts, not strings, so "dark hair"
//! does not contradict "black hair" and the Bengali "কালো" and the English
//! "black" can share one concept.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct ConceptId(pub String);

impl ConceptId {
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

impl From<&str> for ConceptId {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

/// A word in some language that names a concept.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Lexicalisation {
    /// NFKC, case-folded matching form.
    pub form: String,
    /// BCP 47 tag, or "und" when unknown. Lexicon data, not logic.
    #[serde(default = "und")]
    pub language: String,
}

fn und() -> String {
    "und".into()
}

/// A concept hierarchy with a partial order `child ⊑ parent`. Compatibility is
/// a meet: two concepts are compatible when one is a specialisation of the
/// other, or when they share a descendant that is more specific than both.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct ConceptLattice {
    pub name: String,
    /// concept → its parents (more general concepts).
    #[serde(default)]
    pub parents: BTreeMap<ConceptId, Vec<ConceptId>>,
    /// concept → surface forms.
    #[serde(default)]
    pub lexicon: BTreeMap<ConceptId, Vec<Lexicalisation>>,
}

impl ConceptLattice {
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Self::default()
        }
    }

    /// Declare a concept with parents and surface forms in the undetermined language.
    pub fn concept(&mut self, id: &str, parents: &[&str], forms: &[&str]) -> &mut Self {
        let concept = ConceptId::new(id);
        self.parents
            .entry(concept.clone())
            .or_default()
            .extend(parents.iter().map(|parent| ConceptId::new(*parent)));
        for parent in parents {
            self.parents.entry(ConceptId::new(*parent)).or_default();
        }
        let entry = self.lexicon.entry(concept).or_default();
        for form in forms {
            entry.push(Lexicalisation {
                form: fold(form),
                language: und(),
            });
        }
        self
    }

    /// Add a surface form in a language.
    pub fn lexicalise(&mut self, id: &str, form: &str, language: &str) -> &mut Self {
        let concept = ConceptId::new(id);
        self.parents.entry(concept.clone()).or_default();
        self.lexicon
            .entry(concept)
            .or_default()
            .push(Lexicalisation {
                form: fold(form),
                language: language.to_owned(),
            });
        self
    }

    #[must_use]
    pub fn contains(&self, id: &ConceptId) -> bool {
        self.parents.contains_key(id)
    }

    /// All ancestors of a concept, including itself.
    #[must_use]
    pub fn ancestors(&self, id: &ConceptId) -> BTreeSet<ConceptId> {
        let mut seen = BTreeSet::new();
        let mut stack = vec![id.clone()];
        while let Some(next) = stack.pop() {
            if !seen.insert(next.clone()) {
                continue;
            }
            if let Some(parents) = self.parents.get(&next) {
                stack.extend(parents.iter().cloned());
            }
        }
        seen
    }

    /// All descendants of a concept, including itself.
    #[must_use]
    pub fn descendants(&self, id: &ConceptId) -> BTreeSet<ConceptId> {
        let mut seen = BTreeSet::new();
        let mut stack = vec![id.clone()];
        while let Some(next) = stack.pop() {
            if !seen.insert(next.clone()) {
                continue;
            }
            for (child, parents) in &self.parents {
                if parents.contains(&next) {
                    stack.push(child.clone());
                }
            }
        }
        seen
    }

    /// `a ⊑ b`: `a` is `b` or a specialisation of `b`.
    #[must_use]
    pub fn specialises(&self, a: &ConceptId, b: &ConceptId) -> bool {
        self.ancestors(a).contains(b)
    }

    /// The meet is non-bottom when the concepts share a descendant.
    #[must_use]
    pub fn compatible(&self, a: &ConceptId, b: &ConceptId) -> bool {
        if a == b {
            return true;
        }
        if !self.contains(a) || !self.contains(b) {
            return false;
        }
        let left = self.descendants(a);
        let right = self.descendants(b);
        left.intersection(&right).next().is_some()
    }

    /// Resolve a surface form to a concept. Exact match on the folded form.
    #[must_use]
    pub fn resolve(&self, form: &str) -> Option<ConceptId> {
        let folded = fold(form);
        self.lexicon
            .iter()
            .find(|(_, forms)| forms.iter().any(|lex| lex.form == folded))
            .map(|(id, _)| id.clone())
    }

    /// Every surface form with its concept, for building matchers.
    pub fn forms(&self) -> impl Iterator<Item = (&str, &ConceptId)> {
        self.lexicon
            .iter()
            .flat_map(|(id, forms)| forms.iter().map(move |lex| (lex.form.as_str(), id)))
    }

    /// A sibling concept (shares a parent, is not an ancestor or descendant).
    /// Used by the perturbation pipeline to inject plausible contradictions.
    #[must_use]
    pub fn siblings(&self, id: &ConceptId) -> Vec<ConceptId> {
        let parents = self.parents.get(id).cloned().unwrap_or_default();
        let related = {
            let mut set = self.ancestors(id);
            set.extend(self.descendants(id));
            set
        };
        let mut out: Vec<ConceptId> = self
            .parents
            .iter()
            .filter(|(other, other_parents)| {
                !related.contains(*other)
                    && other_parents.iter().any(|parent| parents.contains(parent))
            })
            .map(|(other, _)| other.clone())
            .collect();
        out.sort();
        out.dedup();
        out
    }
}

/// The matching form of a word: lowercased, trimmed. Scripts without case are
/// unaffected. Full NFKC folding lives in `asten-perceive`; the lattice only
/// needs a stable key.
#[must_use]
pub fn fold(form: &str) -> String {
    form.trim().to_lowercase()
}

/// The built-in color lattice. Packs and the author extend it.
#[must_use]
pub fn color_lattice() -> ConceptLattice {
    let mut lattice = ConceptLattice::new("color");
    lattice
        .concept("dark", &[], &["dark", "darkish"])
        .concept("light", &[], &["light", "fair", "pale"])
        .concept(
            "black",
            &["dark"],
            &["black", "raven", "jet", "jet-black", "ebony"],
        )
        .concept(
            "brown",
            &["dark"],
            &["brown", "chestnut", "brunette", "chocolate", "mahogany"],
        )
        .concept(
            "red",
            &[],
            &["red", "ginger", "copper", "scarlet", "crimson"],
        )
        .concept(
            "auburn",
            &["red", "brown"],
            &["auburn", "reddish-brown", "russet"],
        )
        .concept(
            "blond",
            &["light"],
            &["blond", "blonde", "golden", "flaxen", "honey"],
        )
        .concept("white", &["light"], &["white", "snow-white", "snowy"])
        .concept(
            "gray",
            &["light"],
            &["gray", "grey", "silver", "ash", "ashen"],
        )
        .concept("blue", &[], &["blue", "azure", "sapphire", "cobalt"])
        .concept("green", &[], &["green", "emerald", "jade"])
        .concept("hazel", &["brown", "green"], &["hazel"])
        .concept("amber", &["brown"], &["amber"])
        .concept("violet", &[], &["violet", "purple", "lilac"])
        .concept("pink", &[], &["pink", "rose"]);
    // A few lexicalisations in other languages, as a seed. Language data lives
    // in packs; these only show that the lattice is language-independent.
    lattice
        .lexicalise("black", "কালো", "bn")
        .lexicalise("red", "লাল", "bn")
        .lexicalise("brown", "বাদামি", "bn")
        .lexicalise("white", "সাদা", "bn")
        .lexicalise("auburn", "তামাটে", "bn")
        .lexicalise("black", "negro", "es")
        .lexicalise("black", "negra", "es")
        .lexicalise("red", "rojo", "es")
        .lexicalise("red", "roja", "es")
        .lexicalise("brown", "castaño", "es")
        .lexicalise("blond", "rubio", "es")
        .lexicalise("blond", "rubia", "es")
        .lexicalise("black", "noir", "fr")
        .lexicalise("black", "noirs", "fr")
        .lexicalise("red", "roux", "fr")
        .lexicalise("red", "rousse", "fr")
        .lexicalise("brown", "brun", "fr")
        .lexicalise("brown", "brune", "fr")
        .lexicalise("blond", "blonde", "fr")
        .lexicalise("black", "schwarz", "de")
        .lexicalise("red", "rot", "de")
        .lexicalise("brown", "braun", "de")
        .lexicalise("blond", "blond", "de")
        .lexicalise("black", "黒", "ja")
        .lexicalise("black", "黒い", "ja")
        .lexicalise("red", "赤", "ja")
        .lexicalise("black", "黑", "zh")
        .lexicalise("red", "红", "zh")
        .lexicalise("black", "أسود", "ar")
        .lexicalise("red", "أحمر", "ar")
        .lexicalise("black", "काला", "hi")
        .lexicalise("red", "लाल", "hi");
    lattice
}

/// Lifecycle states as a flat lattice with no overlaps.
#[must_use]
pub fn lifecycle_lattice() -> ConceptLattice {
    let mut lattice = ConceptLattice::new("lifecycle");
    lattice
        .concept("alive", &[], &["alive", "living", "lives"])
        .concept(
            "dead",
            &[],
            &["dead", "deceased", "died", "killed", "slain", "perished"],
        )
        .concept("unborn", &[], &["unborn"])
        .concept("intact", &[], &["intact", "whole"])
        .concept("damaged", &[], &["damaged", "broken", "cracked"])
        .concept(
            "destroyed",
            &[],
            &["destroyed", "ruined", "shattered", "burned", "burnt"],
        );
    lattice
}

/// Common materials.
#[must_use]
pub fn material_lattice() -> ConceptLattice {
    let mut lattice = ConceptLattice::new("material");
    lattice
        .concept("metal", &[], &["metal", "metallic"])
        .concept("iron", &["metal"], &["iron"])
        .concept("steel", &["metal", "iron"], &["steel"])
        .concept("bronze", &["metal"], &["bronze"])
        .concept("silver", &["metal"], &["silver"])
        .concept("gold", &["metal"], &["gold", "golden"])
        .concept("wood", &[], &["wood", "wooden", "oak", "ash-wood", "pine"])
        .concept("stone", &[], &["stone", "granite", "marble", "flint"])
        .concept("glass", &[], &["glass"])
        .concept("bone", &[], &["bone", "ivory"])
        .concept("leather", &[], &["leather"])
        .concept("cloth", &[], &["cloth", "linen", "wool", "silk", "cotton"]);
    lattice
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn meet_respects_specialisation() {
        let colors = color_lattice();
        let black = ConceptId::new("black");
        let dark = ConceptId::new("dark");
        let auburn = ConceptId::new("auburn");
        let red = ConceptId::new("red");
        assert!(colors.compatible(&black, &dark));
        assert!(colors.compatible(&auburn, &red));
        assert!(!colors.compatible(&auburn, &black));
        assert!(!colors.compatible(&black, &blond()));
        assert!(colors.specialises(&auburn, &ConceptId::new("brown")));
    }

    fn blond() -> ConceptId {
        ConceptId::new("blond")
    }

    #[test]
    fn resolves_forms_across_languages() {
        let colors = color_lattice();
        assert_eq!(colors.resolve("Black"), Some(ConceptId::new("black")));
        assert_eq!(colors.resolve("কালো"), Some(ConceptId::new("black")));
        assert_eq!(colors.resolve("tamaté"), None);
    }

    #[test]
    fn siblings_exclude_relatives() {
        let colors = color_lattice();
        let siblings = colors.siblings(&ConceptId::new("black"));
        assert!(siblings.contains(&ConceptId::new("brown")));
        assert!(!siblings.contains(&ConceptId::new("auburn")));
        assert!(!siblings.contains(&ConceptId::new("dark")));
    }
}
