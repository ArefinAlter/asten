use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::{ApplicationError, Result};
use asten_domain::{
    CanonEntity, Comment, CommentId, DocumentId, DocumentSnapshot, EntityId, EntityKind,
    MethodPack, MethodPackId, ProjectId, SnapshotId, Suppression, SuppressionId, TimelineEvent,
    TimelineEventId, TrackedChange, TrackedChangeId, TrackedKind, entity_key_from_title,
    normalize_key,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CreateCanonEntity {
    #[serde(default)]
    pub project_id: ProjectId,
    pub kind: EntityKind,
    pub name: String,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub notes: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct UpdateCanonEntity {
    pub kind: Option<EntityKind>,
    pub name: Option<String>,
    pub aliases: Option<Vec<String>>,
    pub notes: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CreateTimelineEvent {
    #[serde(default)]
    pub project_id: ProjectId,
    pub title: String,
    pub entity_id: Option<EntityId>,
    pub document_id: Option<DocumentId>,
    pub story_time: Option<String>,
    pub reading_order: Option<i64>,
    pub duration: Option<String>,
    #[serde(default)]
    pub notes: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct UpdateTimelineEvent {
    pub title: Option<String>,
    pub entity_id: Option<Option<EntityId>>,
    pub document_id: Option<Option<DocumentId>>,
    pub story_time: Option<Option<String>>,
    pub reading_order: Option<Option<i64>>,
    pub duration: Option<Option<String>>,
    pub notes: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CreateComment {
    #[serde(default)]
    pub document_id: DocumentId,
    pub block_id: Option<String>,
    #[serde(default)]
    pub from_offset: i64,
    #[serde(default)]
    pub to_offset: i64,
    pub body: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct UpdateComment {
    pub body: Option<String>,
    pub resolved: Option<bool>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CreateSnapshot {
    #[serde(default)]
    pub document_id: DocumentId,
    pub label: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CreateSuppression {
    #[serde(default)]
    pub project_id: ProjectId,
    pub rule_code: String,
    pub document_id: Option<DocumentId>,
    pub entity_key: Option<String>,
    #[serde(default)]
    pub reason: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UpdateMethodPack {
    pub enabled: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CreateTrackedChange {
    #[serde(default)]
    pub document_id: DocumentId,
    pub kind: TrackedKind,
    pub block_id: Option<String>,
    pub text: String,
    #[serde(default)]
    pub actor: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ResolveTrackedChange {
    pub accepted: bool,
}

impl crate::Application {
    pub async fn list_entities(&self, project_id: ProjectId) -> Result<Vec<CanonEntity>> {
        self.get_project(project_id).await?;
        self.repository.list_entities(project_id).await
    }

    pub async fn create_entity(&self, input: CreateCanonEntity) -> Result<CanonEntity> {
        self.get_project(input.project_id).await?;
        let name = input.name.trim();
        if name.is_empty() {
            return Err(ApplicationError::Validation("a name is required".into()));
        }
        let now = Utc::now();
        let entity = CanonEntity {
            id: EntityId::new(),
            project_id: input.project_id,
            kind: input.kind,
            name: name.to_owned(),
            aliases: input
                .aliases
                .into_iter()
                .map(|alias| alias.trim().to_owned())
                .filter(|alias| !alias.is_empty())
                .collect(),
            notes: input.notes.trim().to_owned(),
            created_at: now,
            updated_at: now,
        };
        self.repository.insert_entity(&entity).await?;
        if entity.kind == EntityKind::Character {
            let key = entity_key_from_title(&entity.name);
            if !key.is_empty() {
                let facts = self.repository.list_facts(entity.project_id).await?;
                if !facts
                    .iter()
                    .any(|fact| fact.entity_key == key && fact.predicate == "kind")
                {
                    let _ = self
                        .create_fact(crate::CreateFact {
                            project_id: entity.project_id,
                            entity_key: key,
                            predicate: "kind".into(),
                            value: "character".into(),
                            modality: Default::default(),
                            status: Default::default(),
                            valid_from: None,
                            valid_to: None,
                            source: None,
                        })
                        .await;
                }
            }
        }
        Ok(entity)
    }

    pub async fn update_entity(
        &self,
        id: EntityId,
        input: UpdateCanonEntity,
    ) -> Result<CanonEntity> {
        let mut entity = self
            .repository
            .find_entity(id)
            .await?
            .ok_or(ApplicationError::NotFound { entity: "entity" })?;
        if let Some(kind) = input.kind {
            entity.kind = kind;
        }
        if let Some(name) = input.name {
            let name = name.trim();
            if name.is_empty() {
                return Err(ApplicationError::Validation("a name is required".into()));
            }
            entity.name = name.to_owned();
        }
        if let Some(aliases) = input.aliases {
            entity.aliases = aliases;
        }
        if let Some(notes) = input.notes {
            entity.notes = notes;
        }
        entity.updated_at = Utc::now();
        self.repository.update_entity(&entity).await?;
        Ok(entity)
    }

    pub async fn list_events(&self, project_id: ProjectId) -> Result<Vec<TimelineEvent>> {
        self.get_project(project_id).await?;
        self.repository.list_events(project_id).await
    }

    pub async fn create_event(&self, input: CreateTimelineEvent) -> Result<TimelineEvent> {
        self.get_project(input.project_id).await?;
        let title = input.title.trim();
        if title.is_empty() {
            return Err(ApplicationError::Validation("a beat needs a title".into()));
        }
        if let Some(document_id) = input.document_id {
            self.get_document(document_id).await?;
        }
        let now = Utc::now();
        let event = TimelineEvent {
            id: TimelineEventId::new(),
            project_id: input.project_id,
            entity_id: input.entity_id,
            document_id: input.document_id,
            title: title.to_owned(),
            story_time: empty_to_none(input.story_time),
            reading_order: input.reading_order,
            duration: empty_to_none(input.duration),
            notes: input.notes.trim().to_owned(),
            created_at: now,
            updated_at: now,
        };
        self.repository.insert_event(&event).await?;
        Ok(event)
    }

    pub async fn update_event(
        &self,
        id: TimelineEventId,
        input: UpdateTimelineEvent,
    ) -> Result<TimelineEvent> {
        let mut event = self
            .repository
            .find_event(id)
            .await?
            .ok_or(ApplicationError::NotFound { entity: "event" })?;
        if let Some(title) = input.title {
            let title = title.trim();
            if title.is_empty() {
                return Err(ApplicationError::Validation("a beat needs a title".into()));
            }
            event.title = title.to_owned();
        }
        if let Some(entity_id) = input.entity_id {
            event.entity_id = entity_id;
        }
        if let Some(document_id) = input.document_id {
            event.document_id = document_id;
        }
        if let Some(story_time) = input.story_time {
            event.story_time = empty_to_none(story_time);
        }
        if let Some(reading_order) = input.reading_order {
            event.reading_order = reading_order;
        }
        if let Some(duration) = input.duration {
            event.duration = empty_to_none(duration);
        }
        if let Some(notes) = input.notes {
            event.notes = notes;
        }
        event.updated_at = Utc::now();
        self.repository.update_event(&event).await?;
        Ok(event)
    }

    pub async fn list_comments(&self, document_id: DocumentId) -> Result<Vec<Comment>> {
        self.get_document(document_id).await?;
        self.repository.list_comments(document_id).await
    }

    pub async fn create_comment(&self, input: CreateComment) -> Result<Comment> {
        self.get_document(input.document_id).await?;
        let body = input.body.trim();
        if body.is_empty() {
            return Err(ApplicationError::Validation("write the note first".into()));
        }
        let now = Utc::now();
        let comment = Comment {
            id: CommentId::new(),
            document_id: input.document_id,
            block_id: input.block_id.filter(|value| !value.is_empty()),
            from_offset: input.from_offset,
            to_offset: input.to_offset,
            body: body.to_owned(),
            resolved: false,
            created_at: now,
            updated_at: now,
        };
        self.repository.insert_comment(&comment).await?;
        Ok(comment)
    }

    pub async fn update_comment(&self, id: CommentId, input: UpdateComment) -> Result<Comment> {
        let mut comment = self
            .repository
            .find_comment(id)
            .await?
            .ok_or(ApplicationError::NotFound { entity: "comment" })?;
        if let Some(body) = input.body {
            let body = body.trim();
            if body.is_empty() {
                return Err(ApplicationError::Validation("write the note first".into()));
            }
            comment.body = body.to_owned();
        }
        if let Some(resolved) = input.resolved {
            comment.resolved = resolved;
        }
        comment.updated_at = Utc::now();
        self.repository.update_comment(&comment).await?;
        Ok(comment)
    }

    pub async fn list_ops(&self, document_id: DocumentId) -> Result<Vec<asten_domain::DocumentOp>> {
        self.get_document(document_id).await?;
        self.repository.list_ops(document_id).await
    }

    pub async fn list_snapshots(&self, document_id: DocumentId) -> Result<Vec<DocumentSnapshot>> {
        self.get_document(document_id).await?;
        self.repository.list_snapshots(document_id).await
    }

    pub async fn create_snapshot(&self, input: CreateSnapshot) -> Result<DocumentSnapshot> {
        let document = self.get_document(input.document_id).await?;
        let snapshot = DocumentSnapshot {
            id: SnapshotId::new(),
            document_id: document.id,
            revision: document.revision,
            label: input
                .label
                .map(|value| value.trim().to_owned())
                .filter(|value| !value.is_empty()),
            content_json: document.content_json.clone(),
            created_at: Utc::now(),
        };
        self.repository.insert_snapshot(&snapshot).await?;
        Ok(snapshot)
    }

    pub async fn restore_snapshot(
        &self,
        snapshot_id: SnapshotId,
    ) -> Result<asten_domain::Document> {
        let snapshot = self
            .repository
            .find_snapshot(snapshot_id)
            .await?
            .ok_or(ApplicationError::NotFound { entity: "snapshot" })?;
        self.update_document(
            snapshot.document_id,
            crate::UpdateDocument {
                title: None,
                content_json: Some(snapshot.content_json),
                parent_id: None,
                position: None,
                exclude_from_compile: None,
                actor: Some("restore".into()),
            },
        )
        .await
    }

    pub async fn list_suppressions(&self, project_id: ProjectId) -> Result<Vec<Suppression>> {
        self.get_project(project_id).await?;
        self.repository.list_suppressions(project_id).await
    }

    pub async fn create_suppression(&self, input: CreateSuppression) -> Result<Suppression> {
        self.get_project(input.project_id).await?;
        let rule_code = input.rule_code.trim();
        if rule_code.is_empty() {
            return Err(ApplicationError::Validation("which rule?".into()));
        }
        let suppression = Suppression {
            id: SuppressionId::new(),
            project_id: input.project_id,
            rule_code: rule_code.to_owned(),
            document_id: input.document_id,
            entity_key: input.entity_key.map(|value| normalize_key(&value)),
            reason: input.reason.trim().to_owned(),
            created_at: Utc::now(),
        };
        self.repository.insert_suppression(&suppression).await?;
        Ok(suppression)
    }

    pub async fn list_packs(&self) -> Result<Vec<MethodPack>> {
        self.repository.list_packs().await
    }

    pub async fn update_pack(
        &self,
        id: MethodPackId,
        input: UpdateMethodPack,
    ) -> Result<MethodPack> {
        let mut pack = self
            .repository
            .find_pack(id)
            .await?
            .ok_or(ApplicationError::NotFound { entity: "pack" })?;
        if pack.pack_key == "scribe" && !input.enabled {
            return Err(ApplicationError::Validation(
                "the scribe pack stays on".into(),
            ));
        }
        pack.enabled = input.enabled;
        self.repository.update_pack(&pack).await?;
        Ok(pack)
    }

    pub async fn list_tracked_changes(
        &self,
        document_id: DocumentId,
    ) -> Result<Vec<TrackedChange>> {
        self.get_document(document_id).await?;
        self.repository.list_tracked_changes(document_id).await
    }

    pub async fn create_tracked_change(&self, input: CreateTrackedChange) -> Result<TrackedChange> {
        self.get_document(input.document_id).await?;
        let text = input.text.trim();
        if text.is_empty() {
            return Err(ApplicationError::Validation("nothing to mark".into()));
        }
        let change = TrackedChange {
            id: TrackedChangeId::new(),
            document_id: input.document_id,
            kind: input.kind,
            block_id: input.block_id,
            text: text.to_owned(),
            actor: input
                .actor
                .unwrap_or_else(|| "author".into())
                .trim()
                .to_owned(),
            accepted: None,
            created_at: Utc::now(),
        };
        self.repository.insert_tracked_change(&change).await?;
        Ok(change)
    }

    pub async fn resolve_tracked_change(
        &self,
        id: TrackedChangeId,
        input: ResolveTrackedChange,
    ) -> Result<TrackedChange> {
        let mut change =
            self.repository
                .find_tracked_change(id)
                .await?
                .ok_or(ApplicationError::NotFound {
                    entity: "tracked change",
                })?;
        change.accepted = Some(input.accepted);
        self.repository.update_tracked_change(&change).await?;
        Ok(change)
    }
}

fn empty_to_none(value: Option<String>) -> Option<String> {
    value.and_then(|item| {
        let trimmed = item.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_owned())
        }
    })
}
