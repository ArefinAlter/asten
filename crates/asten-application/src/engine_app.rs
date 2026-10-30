use asten_blocks::merkle_root;
use asten_ir::{Anchor, Context, EventKind, Provenance};
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::{ApplicationError, Result};
use asten_domain::{
    CanonEntity, Comment, CommentId, DocumentId, DocumentSnapshot, EntityId, EntityKind,
    FactStatus, MethodPack, MethodPackId, ProjectId, SnapshotId, Suppression, SuppressionId,
    SuppressionScope, TimelineEvent, TimelineEventId, TrackedChange, TrackedChangeId, TrackedKind,
    normalize_key,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(
    feature = "ts",
    derive(ts_rs::TS),
    ts(export, export_to = "application/")
)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "ts", ts(optional_fields = nullable))]
pub struct CreateCanonEntity {
    #[serde(default)]
    #[cfg_attr(feature = "ts", ts(as = "Option<_>", optional))]
    pub project_id: ProjectId,
    pub kind: EntityKind,
    pub name: String,
    #[serde(default)]
    #[cfg_attr(feature = "ts", ts(as = "Option<_>", optional))]
    pub aliases: Vec<String>,
    #[serde(default)]
    #[cfg_attr(feature = "ts", ts(as = "Option<_>", optional))]
    pub notes: String,
    #[serde(default)]
    pub introduced_at: Option<i64>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(
    feature = "ts",
    derive(ts_rs::TS),
    ts(export, export_to = "application/")
)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "ts", ts(optional_fields = nullable))]
pub struct UpdateCanonEntity {
    pub kind: Option<EntityKind>,
    pub name: Option<String>,
    pub aliases: Option<Vec<String>>,
    pub notes: Option<String>,
    pub introduced_at: Option<Option<i64>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(
    feature = "ts",
    derive(ts_rs::TS),
    ts(export, export_to = "application/")
)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "ts", ts(optional_fields = nullable))]
pub struct CreateTimelineEvent {
    #[serde(default)]
    #[cfg_attr(feature = "ts", ts(as = "Option<_>", optional))]
    pub project_id: ProjectId,
    pub title: String,
    pub entity_id: Option<EntityId>,
    pub document_id: Option<DocumentId>,
    pub story_time: Option<String>,
    pub reading_order: Option<i64>,
    pub duration: Option<String>,
    #[serde(default)]
    #[cfg_attr(feature = "ts", ts(as = "Option<_>", optional))]
    pub notes: String,
    #[serde(default)]
    pub kind: Option<EventKind>,
    #[serde(default)]
    #[cfg_attr(feature = "ts", ts(as = "Option<_>", optional))]
    pub participants: Vec<String>,
    #[serde(default)]
    pub context: Option<Context>,
    #[serde(default)]
    pub status: Option<FactStatus>,
    #[serde(default)]
    pub tick: Option<i64>,
    #[serde(default)]
    pub tick_lo: Option<i64>,
    #[serde(default)]
    pub tick_hi: Option<i64>,
    #[serde(default)]
    pub anchor: Option<Anchor>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(
    feature = "ts",
    derive(ts_rs::TS),
    ts(export, export_to = "application/")
)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "ts", ts(optional_fields = nullable))]
pub struct UpdateTimelineEvent {
    pub title: Option<String>,
    pub entity_id: Option<Option<EntityId>>,
    pub document_id: Option<Option<DocumentId>>,
    pub story_time: Option<Option<String>>,
    pub reading_order: Option<Option<i64>>,
    pub duration: Option<Option<String>>,
    pub notes: Option<String>,
    pub kind: Option<EventKind>,
    pub participants: Option<Vec<String>>,
    pub context: Option<Context>,
    pub status: Option<FactStatus>,
    pub tick: Option<Option<i64>>,
    pub tick_lo: Option<Option<i64>>,
    pub tick_hi: Option<Option<i64>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(
    feature = "ts",
    derive(ts_rs::TS),
    ts(export, export_to = "application/")
)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "ts", ts(optional_fields = nullable))]
pub struct CreateComment {
    #[serde(default)]
    #[cfg_attr(feature = "ts", ts(as = "Option<_>", optional))]
    pub document_id: DocumentId,
    pub block_id: Option<String>,
    #[serde(default)]
    #[cfg_attr(feature = "ts", ts(as = "Option<_>", optional))]
    pub from_offset: i64,
    #[serde(default)]
    #[cfg_attr(feature = "ts", ts(as = "Option<_>", optional))]
    pub to_offset: i64,
    pub body: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(
    feature = "ts",
    derive(ts_rs::TS),
    ts(export, export_to = "application/")
)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "ts", ts(optional_fields = nullable))]
pub struct UpdateComment {
    pub body: Option<String>,
    pub resolved: Option<bool>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(
    feature = "ts",
    derive(ts_rs::TS),
    ts(export, export_to = "application/")
)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "ts", ts(optional_fields = nullable))]
pub struct CreateSnapshot {
    #[serde(default)]
    #[cfg_attr(feature = "ts", ts(as = "Option<_>", optional))]
    pub document_id: DocumentId,
    pub label: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(
    feature = "ts",
    derive(ts_rs::TS),
    ts(export, export_to = "application/")
)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "ts", ts(optional_fields = nullable))]
pub struct CreateSuppression {
    #[serde(default)]
    #[cfg_attr(feature = "ts", ts(as = "Option<_>", optional))]
    pub project_id: ProjectId,
    pub rule_code: String,
    pub document_id: Option<DocumentId>,
    pub entity_key: Option<String>,
    #[serde(default)]
    #[cfg_attr(feature = "ts", ts(as = "Option<_>", optional))]
    pub reason: String,
    #[serde(default)]
    pub scope: Option<SuppressionScope>,
    #[serde(default)]
    pub diagnostic_id: Option<String>,
    #[serde(default)]
    pub evidence_fingerprint: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(
    feature = "ts",
    derive(ts_rs::TS),
    ts(export, export_to = "application/")
)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "ts", ts(optional_fields = nullable))]
pub struct UpdateMethodPack {
    pub enabled: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(
    feature = "ts",
    derive(ts_rs::TS),
    ts(export, export_to = "application/")
)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "ts", ts(optional_fields = nullable))]
pub struct CreateTrackedChange {
    #[serde(default)]
    #[cfg_attr(feature = "ts", ts(as = "Option<_>", optional))]
    pub document_id: DocumentId,
    pub kind: TrackedKind,
    pub block_id: Option<String>,
    pub text: String,
    #[serde(default)]
    pub actor: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(
    feature = "ts",
    derive(ts_rs::TS),
    ts(export, export_to = "application/")
)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "ts", ts(optional_fields = nullable))]
pub struct ResolveTrackedChange {
    pub accepted: bool,
}

impl crate::Application {
    /// Read at most `limit` rows from the selected project.
    pub async fn list_entities_limited(
        &self,
        project_id: ProjectId,
        limit: u32,
    ) -> Result<Vec<CanonEntity>> {
        self.get_project(project_id).await?;
        self.repository
            .list_entities_limited(project_id, limit.min(100))
            .await
    }

    pub async fn list_entities(&self, project_id: ProjectId) -> Result<Vec<CanonEntity>> {
        self.get_project(project_id).await?;
        self.repository.list_entities(project_id).await
    }

    pub async fn get_entity(&self, id: EntityId) -> Result<CanonEntity> {
        self.repository
            .find_entity(id)
            .await?
            .ok_or(ApplicationError::NotFound { entity: "entity" })
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
            introduced_at: input.introduced_at,
            created_at: now,
            updated_at: now,
        };
        self.repository.insert_entity(&entity).await?;
        self.schedule_refresh(entity.project_id);
        Ok(entity)
    }

    pub async fn update_entity(
        &self,
        id: EntityId,
        input: UpdateCanonEntity,
    ) -> Result<CanonEntity> {
        let mut entity = self.get_entity(id).await?;
        let old_key = entity.key();
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
            entity.aliases = aliases
                .into_iter()
                .map(|alias| alias.trim().to_owned())
                .filter(|alias| !alias.is_empty())
                .collect();
        }
        if let Some(notes) = input.notes {
            entity.notes = notes;
        }
        if let Some(introduced_at) = input.introduced_at {
            entity.introduced_at = introduced_at;
        }
        entity.updated_at = Utc::now();
        self.repository.update_entity(&entity).await?;
        // A renamed entity keeps its facts: move them to the new key.
        let new_key = entity.key();
        if new_key != old_key {
            for mut fact in self.repository.list_facts(entity.project_id).await? {
                if fact.entity_key == old_key {
                    fact.entity_key = new_key.clone();
                    fact.updated_at = Utc::now();
                    self.repository.update_fact(&fact).await?;
                }
            }
        }
        self.schedule_refresh(entity.project_id);
        Ok(entity)
    }

    pub async fn delete_entity(&self, id: EntityId) -> Result<()> {
        let entity = self.get_entity(id).await?;
        self.repository.delete_entity(id).await?;
        self.schedule_refresh(entity.project_id);
        Ok(())
    }

    /// Merge `source` into `target`: the source's name becomes an alias, its
    /// facts and events move over, and the source is removed.
    pub async fn merge_entities(&self, target: EntityId, source: EntityId) -> Result<CanonEntity> {
        if target == source {
            return Err(ApplicationError::Validation(
                "choose two different entities".into(),
            ));
        }
        let mut target_entity = self.get_entity(target).await?;
        let source_entity = self.get_entity(source).await?;
        if target_entity.project_id != source_entity.project_id {
            return Err(ApplicationError::Validation(
                "entities belong to different projects".into(),
            ));
        }
        let (target_key, source_key) = (target_entity.key(), source_entity.key());
        for alias in
            std::iter::once(source_entity.name.clone()).chain(source_entity.aliases.iter().cloned())
        {
            if !target_entity
                .aliases
                .iter()
                .any(|existing| normalize_key(existing) == normalize_key(&alias))
                && normalize_key(&alias) != normalize_key(&target_entity.name)
            {
                target_entity.aliases.push(alias);
            }
        }
        target_entity.updated_at = Utc::now();
        self.repository.update_entity(&target_entity).await?;
        for mut fact in self.repository.list_facts(target_entity.project_id).await? {
            if fact.entity_key == source_key {
                fact.entity_key = target_key.clone();
                fact.updated_at = Utc::now();
                self.repository.update_fact(&fact).await?;
            }
        }
        for mut event in self
            .repository
            .list_events(target_entity.project_id)
            .await?
        {
            let mut changed = false;
            if event.entity_id == Some(source) {
                event.entity_id = Some(target);
                changed = true;
            }
            if event.participants.contains(&source_key) {
                for key in &mut event.participants {
                    if *key == source_key {
                        *key = target_key.clone();
                    }
                }
                changed = true;
            }
            if changed {
                event.updated_at = Utc::now();
                self.repository.update_event(&event).await?;
            }
        }
        self.repository.delete_entity(source).await?;
        self.schedule_refresh(target_entity.project_id);
        Ok(target_entity)
    }

    /// Read at most `limit` rows from the selected project.
    pub async fn list_events_limited(
        &self,
        project_id: ProjectId,
        limit: u32,
    ) -> Result<Vec<TimelineEvent>> {
        self.get_project(project_id).await?;
        self.repository
            .list_events_limited(project_id, limit.min(100))
            .await
    }

    pub async fn list_events(&self, project_id: ProjectId) -> Result<Vec<TimelineEvent>> {
        self.get_project(project_id).await?;
        self.repository.list_events(project_id).await
    }

    pub async fn get_event(&self, id: TimelineEventId) -> Result<TimelineEvent> {
        self.repository
            .find_event(id)
            .await?
            .ok_or(ApplicationError::NotFound { entity: "event" })
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
            story_time: crate::empty_to_none(input.story_time),
            reading_order: input.reading_order,
            duration: crate::empty_to_none(input.duration),
            notes: input.notes.trim().to_owned(),
            kind: input.kind.unwrap_or(EventKind::Acts),
            participants: input
                .participants
                .into_iter()
                .map(|key| asten_domain::entity_key_from_title(&key))
                .filter(|key| !key.is_empty())
                .collect(),
            context: input.context.unwrap_or_default(),
            status: input.status.unwrap_or(FactStatus::Confirmed),
            confidence: 1.0,
            anchor: input.anchor,
            tick: input.tick,
            tick_lo: input.tick_lo,
            tick_hi: input.tick_hi,
            provenance: Provenance::Author {
                action: "create".into(),
            },
            created_at: now,
            updated_at: now,
        };
        self.repository.insert_event(&event).await?;
        self.schedule_refresh(event.project_id);
        Ok(event)
    }

    pub async fn update_event(
        &self,
        id: TimelineEventId,
        input: UpdateTimelineEvent,
    ) -> Result<TimelineEvent> {
        let mut event = self.get_event(id).await?;
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
            event.story_time = crate::empty_to_none(story_time);
        }
        if let Some(reading_order) = input.reading_order {
            event.reading_order = reading_order;
        }
        if let Some(duration) = input.duration {
            event.duration = crate::empty_to_none(duration);
        }
        if let Some(notes) = input.notes {
            event.notes = notes;
        }
        if let Some(kind) = input.kind {
            event.kind = kind;
        }
        if let Some(participants) = input.participants {
            event.participants = participants
                .into_iter()
                .map(|key| asten_domain::entity_key_from_title(&key))
                .filter(|key| !key.is_empty())
                .collect();
        }
        if let Some(context) = input.context {
            event.context = context;
        }
        if let Some(status) = input.status {
            event.status = status;
        }
        if let Some(tick) = input.tick {
            event.tick = tick;
        }
        if let Some(tick_lo) = input.tick_lo {
            event.tick_lo = tick_lo;
        }
        if let Some(tick_hi) = input.tick_hi {
            event.tick_hi = tick_hi;
        }
        event.updated_at = Utc::now();
        self.repository.update_event(&event).await?;
        self.schedule_refresh(event.project_id);
        Ok(event)
    }

    pub async fn delete_event(&self, id: TimelineEventId) -> Result<()> {
        let event = self.get_event(id).await?;
        self.repository.delete_event(id).await?;
        self.schedule_refresh(event.project_id);
        Ok(())
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

    /// A named snapshot: the content plus the event head and Merkle root it commits to.
    pub async fn create_snapshot(&self, input: CreateSnapshot) -> Result<DocumentSnapshot> {
        let document = self.get_document(input.document_id).await?;
        let state = self
            .repository
            .document_state(document.id)
            .await?
            .unwrap_or_default();
        let snapshot = DocumentSnapshot {
            id: SnapshotId::new(),
            document_id: document.id,
            revision: document.revision,
            label: input
                .label
                .map(|value| value.trim().to_owned())
                .filter(|value| !value.is_empty()),
            content_json: document.content_json.clone(),
            seq: document.head_seq,
            chain_hash: document.chain_hash.clone(),
            merkle_root: merkle_root(&state),
            created_at: Utc::now(),
        };
        self.repository.insert_snapshot(&snapshot).await?;
        Ok(snapshot)
    }

    /// Restore appends events; it never rewrites history.
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
                content_json: Some(snapshot.content_json),
                actor: Some("restore".into()),
                ..Default::default()
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
        let scope = input.scope.unwrap_or(
            match (&input.diagnostic_id, &input.entity_key, &input.document_id) {
                (Some(_), _, _) => SuppressionScope::Occurrence,
                (_, Some(_), _) => SuppressionScope::Entity,
                (_, _, Some(_)) => SuppressionScope::Document,
                _ => SuppressionScope::Project,
            },
        );
        let suppression = Suppression {
            id: SuppressionId::new(),
            project_id: input.project_id,
            rule_code: rule_code.to_owned(),
            document_id: input.document_id,
            entity_key: input
                .entity_key
                .map(|value| asten_domain::entity_key_from_title(&value)),
            reason: input.reason.trim().to_owned(),
            scope,
            diagnostic_id: input.diagnostic_id,
            evidence_fingerprint: input.evidence_fingerprint,
            stale: false,
            created_at: Utc::now(),
        };
        self.repository.insert_suppression(&suppression).await?;
        self.schedule_refresh(suppression.project_id);
        Ok(suppression)
    }

    pub async fn delete_suppression(&self, id: SuppressionId) -> Result<()> {
        let suppression =
            self.repository
                .find_suppression(id)
                .await?
                .ok_or(ApplicationError::NotFound {
                    entity: "suppression",
                })?;
        self.repository.delete_suppression(id).await?;
        self.schedule_refresh(suppression.project_id);
        Ok(())
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
        // Packs are global: every open project re-analyses.
        for project in self.repository.list_projects().await? {
            self.schedule_refresh(project.id);
        }
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
