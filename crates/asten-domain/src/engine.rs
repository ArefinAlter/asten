use asten_ir::{Anchor, Context, EventKind, Provenance};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{DocumentId, FactStatus, ProjectId};

entity_id!(EntityId);
entity_id!(TimelineEventId);
entity_id!(CommentId);
entity_id!(DocumentOpId);
entity_id!(SnapshotId);
entity_id!(SuppressionId);
entity_id!(MethodPackId);
entity_id!(TrackedChangeId);
entity_id!(DraftId);
entity_id!(InvariantId);
entity_id!(RouteId);
entity_id!(BaselineId);
entity_id!(AuthTokenId);
entity_id!(PairingCodeId);

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityKind {
    Character,
    Place,
    Event,
    Item,
    Rule,
    Source,
    Other,
}

impl EntityKind {
    #[must_use]
    pub fn to_sort(&self) -> asten_ir::Sort {
        match self {
            EntityKind::Character => asten_ir::Sort::Character,
            EntityKind::Place => asten_ir::Sort::Place,
            EntityKind::Event => asten_ir::Sort::Event,
            EntityKind::Item => asten_ir::Sort::Item,
            EntityKind::Rule => asten_ir::Sort::Rule,
            EntityKind::Source => asten_ir::Sort::Source,
            EntityKind::Other => asten_ir::Sort::Other,
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct CanonEntity {
    pub id: EntityId,
    pub project_id: ProjectId,
    pub kind: EntityKind,
    pub name: String,
    pub aliases: Vec<String>,
    pub notes: String,
    /// Reading position of the scene that introduces the entity, when marked.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub introduced_at: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl CanonEntity {
    /// The language-independent key the engine uses for this entity.
    #[must_use]
    pub fn key(&self) -> String {
        crate::entity_key_from_title(&self.name)
    }
}

/// A beat on the timeline: something that happens, with a kind the engine
/// understands and a placement in story time.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct TimelineEvent {
    pub id: TimelineEventId,
    pub project_id: ProjectId,
    pub entity_id: Option<EntityId>,
    pub document_id: Option<DocumentId>,
    pub title: String,
    /// The label the author typed for story time.
    pub story_time: Option<String>,
    pub reading_order: Option<i64>,
    pub duration: Option<String>,
    pub notes: String,
    #[serde(default = "acts")]
    pub kind: EventKind,
    /// Other participants, by entity key.
    #[serde(default)]
    pub participants: Vec<String>,
    #[serde(default)]
    pub context: Context,
    #[serde(default)]
    pub status: FactStatus,
    #[serde(default = "one")]
    pub confidence: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub anchor: Option<Anchor>,
    /// Structured placement; wins over the label when set.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tick: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tick_lo: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tick_hi: Option<i64>,
    #[serde(default)]
    pub provenance: Provenance,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

fn acts() -> EventKind {
    EventKind::Acts
}

fn one() -> f32 {
    1.0
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Comment {
    pub id: CommentId,
    pub document_id: DocumentId,
    pub block_id: Option<String>,
    pub from_offset: i64,
    pub to_offset: i64,
    pub body: String,
    pub resolved: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Legacy coarse ops, kept for older history views. New history is the
/// hash-chained block event log.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct DocumentOp {
    pub id: DocumentOpId,
    pub document_id: DocumentId,
    pub seq: i64,
    pub kind: String,
    pub op_json: serde_json::Value,
    pub actor: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct DocumentSnapshot {
    pub id: SnapshotId,
    pub document_id: DocumentId,
    pub revision: i64,
    pub label: Option<String>,
    pub content_json: serde_json::Value,
    /// The event sequence and chain hash the snapshot commits to.
    #[serde(default)]
    pub seq: i64,
    #[serde(default)]
    pub chain_hash: String,
    #[serde(default)]
    pub merkle_root: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SuppressionScope {
    /// One mark, bound to its evidence.
    Occurrence,
    Entity,
    #[default]
    Document,
    Project,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Suppression {
    pub id: SuppressionId,
    pub project_id: ProjectId,
    pub rule_code: String,
    pub document_id: Option<DocumentId>,
    pub entity_key: Option<String>,
    pub reason: String,
    #[serde(default)]
    pub scope: SuppressionScope,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub diagnostic_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence_fingerprint: Option<String>,
    #[serde(default)]
    pub stale: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct MethodPack {
    pub id: MethodPackId,
    pub pack_key: String,
    pub name: String,
    pub version: String,
    pub summary: String,
    pub enabled: bool,
    pub manifest: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TrackedKind {
    Insert,
    Delete,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct TrackedChange {
    pub id: TrackedChangeId,
    pub document_id: DocumentId,
    pub kind: TrackedKind,
    pub block_id: Option<String>,
    pub text: String,
    pub actor: String,
    pub accepted: Option<bool>,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct CompileArtifact {
    pub filename: String,
    pub mime: String,
    pub encoding: String,
    pub body: String,
}

/// A materialised checkpoint of a document's event log with a Merkle commitment.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct BlockSnapshot {
    pub id: SnapshotId,
    pub document_id: DocumentId,
    pub seq: i64,
    pub chain_hash: String,
    pub merkle_root: String,
    pub state_json: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

/// A personal branch of a document, merged back three-way.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Draft {
    pub id: DraftId,
    pub document_id: DocumentId,
    pub name: String,
    /// The main document's event sequence the draft branched from.
    pub base_seq: i64,
    /// Materialised state at the base, for the three-way merge.
    pub base_state_json: serde_json::Value,
    /// The draft's current content.
    pub content_json: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub merged_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct StoredInvariant {
    pub id: InvariantId,
    pub project_id: ProjectId,
    pub invariant: asten_ir::Invariant,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct StoredRoute {
    pub id: RouteId,
    pub project_id: ProjectId,
    pub route: asten_ir::Route,
    pub created_at: DateTime<Utc>,
}

/// A frozen set of accepted marks; only marks new since it are highlighted.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Baseline {
    pub id: BaselineId,
    pub project_id: ProjectId,
    pub name: String,
    pub revision: String,
    pub diagnostic_ids: Vec<String>,
    pub active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TokenKind {
    Desk,
    Guest,
    Session,
}

fn default_grants() -> TokenKind {
    TokenKind::Guest
}

impl TokenKind {
    /// Desk and owner-session tokens may manage pairing, tokens and the
    /// passphrase. Guests (Word, Docs, a paired browser pane) may not.
    #[must_use]
    pub fn can_administer(self) -> bool {
        matches!(self, TokenKind::Desk | TokenKind::Session)
    }

    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            TokenKind::Desk => "desk",
            TokenKind::Guest => "guest",
            TokenKind::Session => "session",
        }
    }

    #[must_use]
    pub fn parse(value: &str) -> Self {
        match value {
            "desk" => TokenKind::Desk,
            "session" => TokenKind::Session,
            _ => TokenKind::Guest,
        }
    }
}

/// A paired client. Only the hash of its token is stored.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AuthToken {
    pub id: AuthTokenId,
    pub hash: String,
    pub kind: TokenKind,
    pub label: String,
    pub created_at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub revoked_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_used_at: Option<DateTime<Utc>>,
}

impl AuthToken {
    #[must_use]
    pub fn is_valid_at(&self, now: DateTime<Utc>) -> bool {
        self.revoked_at.is_none() && self.expires_at.is_none_or(|expires| expires > now)
    }
}

/// A short code the desk shows so a guest can pair once.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct PairingCode {
    pub id: PairingCodeId,
    pub code_hash: String,
    pub salt: String,
    pub label: String,
    /// The kind of token the code grants when redeemed. Codes minted for
    /// guests grant guest tokens; the bootstrap code grants a desk token.
    #[serde(default = "default_grants")]
    pub grants: TokenKind,
    pub expires_at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub consumed_at: Option<DateTime<Utc>>,
    pub attempts: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct OwnerCredential {
    pub phc: String,
    pub created_at: DateTime<Utc>,
}

#[must_use]
pub fn parse_story_time(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_owned())
    }
}

#[must_use]
pub fn story_times_out_of_order(earlier: &str, later: &str) -> bool {
    comparable_time(earlier)
        .zip(comparable_time(later))
        .is_some_and(|(left, right)| left > right)
}

fn comparable_time(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    Some(trimmed.to_ascii_lowercase())
}
