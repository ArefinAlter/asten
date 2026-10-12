//! The Story IR.
//!
//! Everything the canon engine reasons over is expressed in these types. Nothing
//! here is a word in a natural language except display labels and the source
//! spans that point back into the manuscript. Perception frontends produce IR
//! atoms; the analyses in `asten-logic` consume them; the diagnostics in
//! `asten-diagnostics` explain them. See `docs/ENGINE_DESIGN.md` §6 and §8.4.

use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub mod lattice;
pub mod time;

pub use lattice::{ConceptId, ConceptLattice, Lexicalisation};
pub use time::{Tick, TimeBounds};

// ───────────────────────── identities ─────────────────────────

macro_rules! string_id {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Clone, Debug, Default, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
        #[serde(transparent)]
        pub struct $name(pub String);

        impl $name {
            #[must_use]
            pub fn new(value: impl Into<String>) -> Self {
                Self(value.into())
            }

            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(&self.0)
            }
        }

        impl From<&str> for $name {
            fn from(value: &str) -> Self {
                Self(value.to_owned())
            }
        }

        impl From<String> for $name {
            fn from(value: String) -> Self {
                Self(value)
            }
        }
    };
}

string_id!(
    /// A stable block identity. Dynodoc mints ULIDs; today's Asten documents
    /// carry UUIDv7 in `attrs.id`. The engine only requires that the id is
    /// stable, unique, and never reused, so both schemes fit.
    BlockId
);
string_id!(
    /// A document in the binder (a scene, chapter, wiki page, note).
    DocId
);
string_id!(
    /// An entity key. Normalised, language-independent once assigned.
    EntityId
);
string_id!(
    /// A fact identity.
    FactId
);
string_id!(
    /// An event identity.
    EventId
);
string_id!(
    /// An observation identity (derived from its anchor and fluent).
    ObservationId
);
string_id!(
    /// A predicate identity, for example `hair_color`.
    PredicateId
);

// ───────────────────────── anchors ─────────────────────────

/// Where a span of text was valid: the document revision and the block fingerprint.
#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct AnchorRevision {
    /// The document's event sequence number at which the offsets were valid.
    pub seq: i64,
    /// The block's content fingerprint (lowercase hex SHA-256) at that revision.
    pub fingerprint: String,
}

/// A block-relative anchor. Offsets count Unicode scalar values inside the
/// block's plain text, matching Dynodoc's `RichTextPatch` units. User-facing
/// code converts to grapheme clusters at the boundary.
#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Anchor {
    pub document: DocId,
    pub block: BlockId,
    pub start: u32,
    pub end: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub revision: Option<AnchorRevision>,
    /// The anchored text at that revision, kept for re-attachment and display.
    #[serde(default)]
    pub quote: String,
    /// Up to ~32 scalars before and after the quote, for fuzzy re-attachment.
    #[serde(default)]
    pub context_before: String,
    #[serde(default)]
    pub context_after: String,
    /// Set when the anchor could not be mapped or re-attached after an edit.
    #[serde(default)]
    pub stale: bool,
}

impl Anchor {
    #[must_use]
    pub fn new(
        document: impl Into<DocId>,
        block: impl Into<BlockId>,
        start: u32,
        end: u32,
    ) -> Self {
        Self {
            document: document.into(),
            block: block.into(),
            start,
            end,
            ..Self::default()
        }
    }

    /// Fill the quote and surrounding context from the block's plain text.
    #[must_use]
    pub fn with_text(mut self, block_text: &str) -> Self {
        let chars: Vec<char> = block_text.chars().collect();
        let start = (self.start as usize).min(chars.len());
        let end = (self.end as usize).clamp(start, chars.len());
        self.quote = chars[start..end].iter().collect();
        let before = start.saturating_sub(32);
        let after = (end + 32).min(chars.len());
        self.context_before = chars[before..start].iter().collect();
        self.context_after = chars[end..after].iter().collect();
        self
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.start >= self.end
    }
}

// ───────────────────────── sorts, predicates, values ─────────────────────────

/// What kind of thing an entity is. Packs may add sorts through `Other`.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Sort {
    Character,
    Place,
    Item,
    Group,
    Event,
    Concept,
    Rule,
    Source,
    Other,
}

impl Sort {
    #[must_use]
    pub fn label(&self) -> &'static str {
        match self {
            Sort::Character => "character",
            Sort::Place => "place",
            Sort::Item => "item",
            Sort::Group => "group",
            Sort::Event => "event",
            Sort::Concept => "concept",
            Sort::Rule => "rule",
            Sort::Source => "source",
            Sort::Other => "other",
        }
    }
}

/// How many values a predicate may hold at one time, and whether it may change.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Cardinality {
    /// One value at a time (hair color). Drives contradiction checks.
    Functional,
    /// Many values at once (allies).
    Multi,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Mutability {
    Mutable,
    /// Any change event is an error unless declared as a retcon.
    Immutable,
}

/// The value domain of a predicate.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum ValueDomain {
    /// Values are concepts in a named lattice (colors, materials, lifecycle states).
    Concept {
        lattice: String,
    },
    /// Values are numeric intervals with a unit dimension.
    Quantity {
        dimension: Dimension,
    },
    /// Values are entities of a sort.
    Entity {
        sort: Option<Sort>,
    },
    /// Free text; compared after normalisation only.
    Text,
    Boolean,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Dimension {
    Length,
    Mass,
    Duration,
    Count,
    Age,
}

/// Algebraic properties of relation predicates (`parent_of`, `married_to`).
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct RelationProps {
    #[serde(default)]
    pub symmetric: bool,
    #[serde(default)]
    pub transitive: bool,
    #[serde(default)]
    pub acyclic: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inverse: Option<PredicateId>,
    /// Two simultaneous relations of this kind are an error (two marriages).
    #[serde(default)]
    pub exclusive: bool,
}

/// A predicate signature: `hair_color : Character → Color`.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PredicateSig {
    pub id: PredicateId,
    pub label: String,
    #[serde(default)]
    pub subject_sorts: Vec<Sort>,
    pub domain: ValueDomain,
    pub cardinality: Cardinality,
    pub mutability: Mutability,
    #[serde(default)]
    pub relation: RelationProps,
    /// Words that cue this predicate near a mention in prose ("hair", "eyes").
    /// Used only by the deterministic tier-B frontend; language data, not logic.
    #[serde(default)]
    pub cues: Vec<String>,
    /// Which pack declared it. Core predicates say "scribe".
    #[serde(default)]
    pub pack: String,
}

/// A value in a predicate's domain. Strings never reach the analyses: concepts,
/// intervals and entity ids do.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum Value {
    Concept {
        id: ConceptId,
        /// The author's or the text's spelling, for display only.
        #[serde(default)]
        label: String,
    },
    Quantity {
        /// Closed interval in the dimension's base unit (centimetres, grams, minutes, count).
        lo: f64,
        hi: f64,
        #[serde(default)]
        label: String,
    },
    Entity {
        id: EntityId,
    },
    Text {
        /// NFKC-folded matching form.
        norm: String,
        #[serde(default)]
        label: String,
    },
    Bool {
        value: bool,
    },
}

impl Value {
    #[must_use]
    pub fn label(&self) -> String {
        match self {
            Value::Concept { id, label } => {
                if label.is_empty() {
                    id.0.clone()
                } else {
                    label.clone()
                }
            }
            Value::Quantity { label, lo, hi, .. } => {
                if label.is_empty() {
                    if (lo - hi).abs() < f64::EPSILON {
                        format!("{lo}")
                    } else {
                        format!("{lo}–{hi}")
                    }
                } else {
                    label.clone()
                }
            }
            Value::Entity { id } => id.0.clone(),
            Value::Text { norm, label } => {
                if label.is_empty() {
                    norm.clone()
                } else {
                    label.clone()
                }
            }
            Value::Bool { value } => value.to_string(),
        }
    }

    /// A stable key for grouping identical values.
    #[must_use]
    pub fn key(&self) -> String {
        match self {
            Value::Concept { id, .. } => format!("c:{}", id.0),
            Value::Quantity { lo, hi, .. } => format!("q:{lo}:{hi}"),
            Value::Entity { id } => format!("e:{}", id.0),
            Value::Text { norm, .. } => format!("t:{norm}"),
            Value::Bool { value } => format!("b:{value}"),
        }
    }
}

// ───────────────────────── contexts ─────────────────────────

/// The epistemic frame a statement lives in (§6.3). Contradictions are only
/// computed inside one context after inheritance.
#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum Context {
    /// World truth: what is actually so in the story world.
    World,
    /// Narration. Projects into `World` unless the narrator is marked unreliable.
    #[default]
    Narration,
    /// What a character claims. Does not project into the world.
    Says {
        speaker: Option<EntityId>,
    },
    /// What a character believes. Projects into their knowledge state only.
    Believes {
        who: EntityId,
    },
    /// A memory or flashback told from a later vantage. Projects into the world
    /// at the remembered time; time placement comes from the scene.
    Memory,
    Dream {
        key: String,
    },
    Vision {
        key: String,
    },
    Frame {
        key: String,
    },
    Hypothetical,
}

impl Context {
    /// Whether facts asserted in this context are visible in the world context.
    #[must_use]
    pub fn projects_to_world(&self, narrator_reliable: bool) -> bool {
        match self {
            Context::World | Context::Memory => true,
            Context::Narration => narrator_reliable,
            _ => false,
        }
    }

    /// The context two statements must share to be compared.
    #[must_use]
    pub fn comparison_key(&self, narrator_reliable: bool) -> Context {
        if self.projects_to_world(narrator_reliable) {
            Context::World
        } else {
            self.clone()
        }
    }

    #[must_use]
    pub fn label(&self) -> String {
        match self {
            Context::World => "world".into(),
            Context::Narration => "narration".into(),
            Context::Says { speaker: Some(who) } => format!("said by {who}"),
            Context::Says { speaker: None } => "dialogue".into(),
            Context::Believes { who } => format!("believed by {who}"),
            Context::Memory => "memory".into(),
            Context::Dream { .. } => "dream".into(),
            Context::Vision { .. } => "vision".into(),
            Context::Frame { key } => format!("frame story {key}"),
            Context::Hypothetical => "hypothetical".into(),
        }
    }
}

// ───────────────────────── status and provenance ─────────────────────────

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    /// Author-confirmed. Exact checks apply.
    Settled,
    /// Extracted or inferred, awaiting the author.
    #[default]
    Proposed,
    /// Retracted but kept for history.
    Deprecated,
    Rejected,
}

impl Status {
    #[must_use]
    pub fn is_active(self) -> bool {
        matches!(self, Status::Settled | Status::Proposed)
    }
}

/// Where a statement came from. Not decoration: explanation and blame are
/// computed from it (§6.2).
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum Provenance {
    /// An explicit author action in the desk, the wiki, or the cast.
    Author {
        #[serde(default)]
        action: String,
    },
    /// A span in the manuscript, found deterministically.
    Text { anchor: Anchor },
    /// A wiki field or a world card.
    Wiki { document: DocId },
    /// A perception model.
    Model {
        id: String,
        version: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        anchor: Option<Anchor>,
        #[serde(default)]
        input_fingerprint: String,
    },
    /// Derived by a rule from premises.
    Rule {
        code: String,
        #[serde(default)]
        premises: Vec<String>,
    },
    /// The bootstrap importer.
    Import {
        #[serde(default)]
        job: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        anchor: Option<Anchor>,
    },
}

impl Default for Provenance {
    fn default() -> Self {
        Provenance::Author {
            action: String::new(),
        }
    }
}

impl Provenance {
    #[must_use]
    pub fn anchor(&self) -> Option<&Anchor> {
        match self {
            Provenance::Text { anchor } => Some(anchor),
            Provenance::Model {
                anchor: Some(anchor),
                ..
            }
            | Provenance::Import {
                anchor: Some(anchor),
                ..
            } => Some(anchor),
            _ => None,
        }
    }

    /// The prior probability that a component with this provenance is the one
    /// at fault in a conflict (§10.6). Settled author facts are rarely wrong;
    /// extracted observations in dialogue often are.
    #[must_use]
    pub fn fault_prior(&self, status: Status, context: &Context, confidence: f32) -> f32 {
        let base = match (self, status) {
            (Provenance::Author { .. }, Status::Settled) => 0.01,
            (Provenance::Wiki { .. }, Status::Settled) => 0.03,
            (Provenance::Author { .. } | Provenance::Wiki { .. }, _) => 0.08,
            (Provenance::Text { .. }, _) => (1.0 - confidence).max(0.05),
            (Provenance::Model { .. } | Provenance::Import { .. }, _) => {
                (1.0 - confidence).max(0.1)
            }
            (Provenance::Rule { .. }, _) => 0.02,
        };
        match context {
            Context::Says { .. } | Context::Believes { .. } => base.max(0.3),
            _ => base,
        }
    }
}

// ───────────────────────── entities and mentions ─────────────────────────

/// A name an entity goes by, possibly scoped in story time or to a speaker.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Alias {
    pub form: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from_tick: Option<Tick>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub used_by: Option<EntityId>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Entity {
    pub id: EntityId,
    pub sort: Sort,
    pub name: String,
    #[serde(default)]
    pub aliases: Vec<Alias>,
    /// The reading position at which the entity is introduced, when the author
    /// marked one. Used by the "used before introduction" check.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub introduced_at: Option<i64>,
}

impl Entity {
    /// All surface forms the entity may appear under.
    pub fn surface_forms(&self) -> impl Iterator<Item = &str> {
        std::iter::once(self.name.as_str())
            .chain(self.aliases.iter().map(|alias| alias.form.as_str()))
    }
}

/// An anchored span that refers to an entity, with a link confidence.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Mention {
    pub anchor: Anchor,
    pub entity: EntityId,
    pub surface: String,
    pub confidence: f32,
    /// Reading position of the containing block.
    pub reading_pos: i64,
    pub context: Context,
}

// ───────────────────────── fluents, facts, observations, events ─────────────────────────

/// A property that can hold or stop holding over story time.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Fluent {
    pub predicate: PredicateId,
    pub subject: EntityId,
    pub value: Value,
}

impl Fluent {
    /// The family key `(subject, predicate)`: functional fluents conflict within one family.
    #[must_use]
    pub fn family(&self) -> String {
        format!("{}·{}", self.subject.0, self.predicate.0)
    }

    #[must_use]
    pub fn label(&self) -> String {
        format!(
            "{} · {} · {}",
            self.subject.0,
            self.predicate.0,
            self.value.label()
        )
    }
}

/// A fact: a fluent that holds in a context over a validity interval.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Fact {
    pub id: FactId,
    pub fluent: Fluent,
    #[serde(default)]
    pub context: Context,
    /// Validity in story time. Unbounded sides are `None`.
    #[serde(default)]
    pub valid: TimeBounds,
    pub status: Status,
    #[serde(default)]
    pub provenance: Provenance,
    /// Calibrated confidence in `[0, 1]`. Author facts are 1.0.
    #[serde(default = "one")]
    pub confidence: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub anchor: Option<Anchor>,
    /// The event or author action that settled the fact (for blame).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub settled_at: Option<SettledAt>,
}

fn one() -> f32 {
    1.0
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SettledAt {
    pub actor: String,
    pub at: String,
}

/// Where a statement sits in story time.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum TimeRef {
    /// Known exactly.
    Tick { tick: Tick },
    /// Known within bounds (from the temporal network or a fuzzy label).
    Bounds { lo: Option<Tick>, hi: Option<Tick> },
    /// Placed at a scene: it inherits the scene's time.
    Scene { document: DocId },
    #[default]
    Unknown,
}

/// A statement in the prose that *reports* a fluent without causing it.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Observation {
    pub id: ObservationId,
    pub fluent: Fluent,
    #[serde(default)]
    pub context: Context,
    #[serde(default)]
    pub at: TimeRef,
    pub anchor: Anchor,
    pub confidence: f32,
    #[serde(default)]
    pub negated: bool,
    #[serde(default)]
    pub provenance: Provenance,
    #[serde(default)]
    pub reading_pos: i64,
}

/// What kind of thing happened. Packs can add kinds through `Other`.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum EventKind {
    Born,
    Dies,
    Resurrected,
    Destroyed,
    Repaired,
    Travels {
        from: Option<EntityId>,
        to: Option<EntityId>,
    },
    Arrives {
        place: EntityId,
    },
    Acquires {
        item: EntityId,
    },
    Loses {
        item: EntityId,
    },
    Learns {
        secret: String,
    },
    Tells {
        to: EntityId,
        secret: String,
    },
    Reveals {
        secret: String,
    },
    UsesKnowledge {
        secret: String,
    },
    Marries {
        to: EntityId,
    },
    Divorces {
        to: EntityId,
    },
    /// A declared change of a fluent: "Mira dyes her hair black".
    Changes {
        fluent: Fluent,
    },
    /// Any act: speaking, moving, fighting. Used by lifecycle checks.
    Acts,
    Other {
        label: String,
    },
}

impl EventKind {
    #[must_use]
    pub fn label(&self) -> String {
        match self {
            EventKind::Born => "is born".into(),
            EventKind::Dies => "dies".into(),
            EventKind::Resurrected => "returns from death".into(),
            EventKind::Destroyed => "is destroyed".into(),
            EventKind::Repaired => "is repaired".into(),
            EventKind::Travels { to: Some(to), .. } => format!("travels to {to}"),
            EventKind::Travels { .. } => "travels".into(),
            EventKind::Arrives { place } => format!("arrives at {place}"),
            EventKind::Acquires { item } => format!("acquires {item}"),
            EventKind::Loses { item } => format!("loses {item}"),
            EventKind::Learns { secret } => format!("learns {secret}"),
            EventKind::Tells { to, secret } => format!("tells {to} about {secret}"),
            EventKind::Reveals { secret } => format!("reveals {secret}"),
            EventKind::UsesKnowledge { secret } => format!("uses knowledge of {secret}"),
            EventKind::Marries { to } => format!("marries {to}"),
            EventKind::Divorces { to } => format!("divorces {to}"),
            EventKind::Changes { fluent } => format!("changes {}", fluent.label()),
            EventKind::Acts => "acts".into(),
            EventKind::Other { label } => label.clone(),
        }
    }
}

/// Something that happens at a story time and can initiate or terminate fluents.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Event {
    pub id: EventId,
    pub kind: EventKind,
    #[serde(default)]
    pub label: String,
    /// The primary participant (who dies, who travels).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subject: Option<EntityId>,
    #[serde(default)]
    pub participants: Vec<EntityId>,
    #[serde(default)]
    pub at: TimeRef,
    /// Reading position of the scene that narrates it, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reading_pos: Option<i64>,
    #[serde(default)]
    pub context: Context,
    pub status: Status,
    #[serde(default = "one")]
    pub confidence: f32,
    #[serde(default)]
    pub provenance: Provenance,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub anchor: Option<Anchor>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub document: Option<DocId>,
    /// Fluents the event starts and ends, beyond what its kind implies.
    #[serde(default)]
    pub initiates: Vec<Fluent>,
    #[serde(default)]
    pub terminates: Vec<Fluent>,
}

/// A unit of narration with a reading position and a story-time placement.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Scene {
    pub document: DocId,
    pub title: String,
    pub reading_pos: i64,
    #[serde(default)]
    pub at: TimeRef,
    /// The label the author typed ("day 14", "1890-03-02", "three days after the siege").
    #[serde(default)]
    pub time_label: String,
    #[serde(default)]
    pub context: Context,
    /// Scenes sharing a run key were declared continuous by the author.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub continuity_run: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    /// Blocks in reading order.
    #[serde(default)]
    pub blocks: Vec<BlockId>,
}

// ───────────────────────── constraints ─────────────────────────

/// What a temporal constraint relates: a scene or an event, each a time point.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum TimePoint {
    Origin,
    Scene {
        document: DocId,
    },
    Event {
        id: EventId,
    },
    /// A named time point the author or a frontend introduced ("the siege").
    Named {
        label: String,
    },
}

impl TimePoint {
    #[must_use]
    pub fn label(&self) -> String {
        match self {
            TimePoint::Origin => "the story's origin".into(),
            TimePoint::Scene { document } => format!("scene {document}"),
            TimePoint::Event { id } => format!("event {id}"),
            TimePoint::Named { label } => label.clone(),
        }
    }
}

/// `lo ≤ to − from ≤ hi`, in ticks, with provenance (§9.6).
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct TemporalConstraint {
    pub from: TimePoint,
    pub to: TimePoint,
    pub lo: Option<Tick>,
    pub hi: Option<Tick>,
    pub label: String,
    #[serde(default)]
    pub provenance: Provenance,
    pub status: Status,
    #[serde(default = "one")]
    pub confidence: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub anchor: Option<Anchor>,
}

// ───────────────────────── evidence ─────────────────────────

/// Any perceptual output. Reasoning never reads raw text; it reads these.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum Atom {
    Mention(Mention),
    Observation(Observation),
    Event(Event),
    Constraint(TemporalConstraint),
    /// A span that looks like a name but links to no entity.
    UnknownName {
        anchor: Anchor,
        surface: String,
        reading_pos: i64,
    },
    /// A clause's context, when it differs from the scene's.
    ContextSpan {
        anchor: Anchor,
        context: Context,
    },
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Evidence {
    pub atom: Atom,
    pub confidence: f32,
    pub provenance: Provenance,
    pub status: Status,
}

// ───────────────────────── invariants ─────────────────────────

/// An atomic proposition over a scene's state, for author invariants (§9.10).
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum Proposition {
    /// A fluent holds at the scene's time.
    Holds { fluent: Fluent },
    /// An event of this kind, with this subject, happens in the scene.
    Happens {
        subject: Option<EntityId>,
        event: EventKindPattern,
    },
    /// The entity is mentioned in the scene.
    Mentioned { entity: EntityId },
    /// The scene has this context.
    InContext { context: Context },
}

/// A coarse pattern over event kinds so authors can say "dies" without arguments.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EventKindPattern {
    Born,
    Dies,
    Travels,
    Acquires,
    Loses,
    Learns,
    Tells,
    Reveals,
    Marries,
    Acts,
    Any,
    Other(String),
}

impl EventKindPattern {
    #[must_use]
    pub fn matches(&self, kind: &EventKind) -> bool {
        match (self, kind) {
            (EventKindPattern::Any, _) => true,
            (EventKindPattern::Born, EventKind::Born)
            | (EventKindPattern::Dies, EventKind::Dies)
            | (EventKindPattern::Travels, EventKind::Travels { .. } | EventKind::Arrives { .. })
            | (EventKindPattern::Acquires, EventKind::Acquires { .. })
            | (EventKindPattern::Loses, EventKind::Loses { .. })
            | (EventKindPattern::Learns, EventKind::Learns { .. })
            | (EventKindPattern::Tells, EventKind::Tells { .. })
            | (EventKindPattern::Reveals, EventKind::Reveals { .. })
            | (EventKindPattern::Marries, EventKind::Marries { .. })
            | (EventKindPattern::Acts, EventKind::Acts) => true,
            (EventKindPattern::Other(label), EventKind::Other { label: other }) => label == other,
            _ => false,
        }
    }
}

/// The Dwyer, Avrunin and Corbett specification patterns authors see instead of LTLf.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", tag = "pattern")]
pub enum InvariantPattern {
    /// Once `after` holds, `never` never holds again. "Once Oren dies, he never speaks."
    AbsenceAfter {
        after: Proposition,
        never: Proposition,
    },
    /// Every `trigger` is eventually followed by `response`. "Every clue is paid off."
    Response {
        trigger: Proposition,
        response: Proposition,
    },
    /// `later` never happens before `first`. "The reveal never comes before the clue."
    Precedence {
        first: Proposition,
        later: Proposition,
    },
    /// Between `from` and `until`, `always` holds in every scene.
    UniversalityBetween {
        from: Proposition,
        until: Proposition,
        always: Proposition,
    },
    /// `event` happens at most `max` times.
    BoundedExistence { event: Proposition, max: u32 },
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TraceClock {
    #[default]
    StoryTime,
    ReadingOrder,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Invariant {
    pub id: String,
    pub label: String,
    pub pattern: InvariantPattern,
    #[serde(default)]
    pub clock: TraceClock,
    #[serde(default)]
    pub enabled: bool,
}

// ───────────────────────── places and routes ─────────────────────────

/// A route between two places with a travel duration range, in ticks.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Route {
    pub from: EntityId,
    pub to: EntityId,
    pub min_ticks: Tick,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_ticks: Option<Tick>,
    #[serde(default)]
    pub mode: String,
    #[serde(default)]
    pub bidirectional: bool,
    /// Ticks during which the route exists (a bridge that burns).
    #[serde(default)]
    pub available: TimeBounds,
}

// ───────────────────────── project view ─────────────────────────

/// Project-wide analysis settings.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Settings {
    pub strictness: Strictness,
    /// Narration projects into world truth unless the author marks it unreliable.
    pub narrator_reliable: bool,
    /// Ticks per day in this project's base unit (1 for days, 24 for hours, 1440 for minutes).
    pub ticks_per_day: Tick,
    /// Sorts for which resurrection is allowed by a world rule.
    #[serde(default)]
    pub resurrection_allowed: Vec<Sort>,
    /// Minimum parent age at a child's birth, in ticks. None disables the check.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_parent_age_ticks: Option<Tick>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            strictness: Strictness::Standard,
            narrator_reliable: true,
            ticks_per_day: 1,
            resurrection_allowed: Vec::new(),
            min_parent_age_ticks: None,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Strictness {
    /// Only settled-versus-settled conflicts.
    Quiet,
    /// Plus settled-versus-observed above a confidence threshold.
    #[default]
    Standard,
    /// Plus proposed-versus-proposed, ranked, for revision passes.
    Thorough,
}

/// The full IR for one project revision. Built by the engine from the stored
/// canon and the perception of every block; consumed by every analysis.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct StoryIr {
    pub entities: Vec<Entity>,
    pub predicates: Vec<PredicateSig>,
    pub scenes: Vec<Scene>,
    pub facts: Vec<Fact>,
    pub observations: Vec<Observation>,
    pub events: Vec<Event>,
    pub mentions: Vec<Mention>,
    pub constraints: Vec<TemporalConstraint>,
    pub invariants: Vec<Invariant>,
    pub routes: Vec<Route>,
    pub unknown_names: Vec<(Anchor, String, i64)>,
    pub settings: Settings,
    /// Lattices by name.
    pub lattices: BTreeMap<String, ConceptLattice>,
}

impl StoryIr {
    #[must_use]
    pub fn predicate(&self, id: &PredicateId) -> Option<&PredicateSig> {
        self.predicates.iter().find(|sig| &sig.id == id)
    }

    #[must_use]
    pub fn entity(&self, id: &EntityId) -> Option<&Entity> {
        self.entities.iter().find(|entity| &entity.id == id)
    }

    #[must_use]
    pub fn scene(&self, document: &DocId) -> Option<&Scene> {
        self.scenes.iter().find(|scene| &scene.document == document)
    }

    #[must_use]
    pub fn lattice_for(&self, predicate: &PredicateId) -> Option<&ConceptLattice> {
        match &self.predicate(predicate)?.domain {
            ValueDomain::Concept { lattice } => self.lattices.get(lattice),
            _ => None,
        }
    }

    /// Whether two values of a predicate are compatible (their meet is not bottom).
    #[must_use]
    pub fn compatible(&self, predicate: &PredicateId, left: &Value, right: &Value) -> bool {
        match (left, right) {
            (Value::Concept { id: a, .. }, Value::Concept { id: b, .. }) => {
                match self.lattice_for(predicate) {
                    Some(lattice) => lattice.compatible(a, b),
                    None => a == b,
                }
            }
            (
                Value::Quantity {
                    lo: a_lo, hi: a_hi, ..
                },
                Value::Quantity {
                    lo: b_lo, hi: b_hi, ..
                },
            ) => a_lo <= b_hi && b_lo <= a_hi,
            (Value::Entity { id: a }, Value::Entity { id: b }) => a == b,
            (Value::Text { norm: a, .. }, Value::Text { norm: b, .. }) => a == b,
            (Value::Bool { value: a }, Value::Bool { value: b }) => a == b,
            _ => false,
        }
    }
}

// ───────────────────────── fingerprints ─────────────────────────

/// Lowercase hex SHA-256 of a byte string. Used for block fingerprints,
/// evidence fingerprints, and stable diagnostic ids.
#[must_use]
pub fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut out = String::with_capacity(64);
    for byte in digest {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

/// A fingerprint of a set of statement identities, order-independent.
#[must_use]
pub fn evidence_fingerprint<'a>(parts: impl IntoIterator<Item = &'a str>) -> String {
    let mut items: Vec<&str> = parts.into_iter().collect();
    items.sort_unstable();
    items.dedup();
    sha256_hex(items.join("\u{1f}").as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn anchors_quote_in_scalars() {
        let anchor = Anchor::new("doc", "block", 5, 10).with_text("মিরা পুশড her hair");
        assert_eq!(anchor.quote, "পুশড ");
        assert_eq!(anchor.context_before, "মিরা ");
        assert_eq!(anchor.context_after, "her hair");
    }

    #[test]
    fn contexts_project_by_reliability() {
        assert!(Context::Narration.projects_to_world(true));
        assert!(!Context::Narration.projects_to_world(false));
        assert!(!Context::Says { speaker: None }.projects_to_world(true));
        assert_eq!(Context::Memory.comparison_key(true), Context::World);
    }

    #[test]
    fn fault_priors_follow_status_and_context() {
        let settled = Provenance::Author {
            action: String::new(),
        }
        .fault_prior(Status::Settled, &Context::World, 1.0);
        let spoken = Provenance::Text {
            anchor: Anchor::default(),
        }
        .fault_prior(Status::Proposed, &Context::Says { speaker: None }, 0.9);
        assert!(settled < spoken);
        assert!(spoken >= 0.3);
    }

    #[test]
    fn evidence_fingerprint_is_order_independent() {
        assert_eq!(
            evidence_fingerprint(["a", "b"]),
            evidence_fingerprint(["b", "a", "a"])
        );
    }
}
