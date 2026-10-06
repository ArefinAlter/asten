use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{DocumentId, ProjectId};

entity_id!(EntityId);
entity_id!(TimelineEventId);
entity_id!(CommentId);
entity_id!(DocumentOpId);
entity_id!(SnapshotId);
entity_id!(SuppressionId);
entity_id!(MethodPackId);
entity_id!(TrackedChangeId);

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

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct CanonEntity {
    pub id: EntityId,
    pub project_id: ProjectId,
    pub kind: EntityKind,
    pub name: String,
    pub aliases: Vec<String>,
    pub notes: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct TimelineEvent {
    pub id: TimelineEventId,
    pub project_id: ProjectId,
    pub entity_id: Option<EntityId>,
    pub document_id: Option<DocumentId>,
    pub title: String,
    pub story_time: Option<String>,
    pub reading_order: Option<i64>,
    pub duration: Option<String>,
    pub notes: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
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
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Suppression {
    pub id: SuppressionId,
    pub project_id: ProjectId,
    pub rule_code: String,
    pub document_id: Option<DocumentId>,
    pub entity_key: Option<String>,
    pub reason: String,
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

impl Suppression {
    #[must_use]
    pub fn hides(&self, code: &str, document_id: DocumentId, entity_key: Option<&str>) -> bool {
        if self.rule_code != code {
            return false;
        }
        if let Some(target) = self.document_id {
            if target != document_id {
                return false;
            }
        }
        if let Some(key) = &self.entity_key {
            if entity_key.is_none_or(|value| value != key) {
                return false;
            }
        }
        true
    }
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
    comparable_time(earlier).zip(comparable_time(later)).is_some_and(|(left, right)| left > right)
}

fn comparable_time(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    Some(trimmed.to_ascii_lowercase())
}
