use std::path::Path;

use asten_application::{ApplicationError, Repository, Result};
use asten_domain::{
    CanonEntity, Comment, CommentId, Document, DocumentId, DocumentKind, DocumentOp, DocumentOpId,
    DocumentSnapshot, EntityId, EntityKind, Fact, FactId, FactModality, FactStatus, IdeaPrompt,
    MethodPack, MethodPackId, Project, ProjectId, ProjectKind, SnapshotId, SourceAnchor,
    Suppression, SuppressionId, TimelineEvent, TimelineEventId, TrackedChange, TrackedChangeId,
    TrackedKind, WorldEntry, WorldEntryId, WorldKind, WorldStatus,
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{
    Row, SqlitePool,
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions},
};
use uuid::Uuid;

#[derive(Clone)]
pub struct SqliteRepository {
    pool: SqlitePool,
}

impl SqliteRepository {
    pub async fn open(path: impl AsRef<Path>) -> Result<Self> {
        let options = SqliteConnectOptions::new()
            .filename(path)
            .create_if_missing(true)
            .foreign_keys(true)
            .journal_mode(SqliteJournalMode::Wal);
        let pool = SqlitePoolOptions::new()
            .max_connections(8)
            .connect_with(options)
            .await
            .map_err(storage_error)?;
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .map_err(storage_error)?;
        Ok(Self { pool })
    }

    #[must_use]
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}

#[async_trait]
impl Repository for SqliteRepository {
    async fn list_projects(&self) -> Result<Vec<Project>> {
        sqlx::query(
            "SELECT id, name, kind, description, created_at, updated_at
             FROM projects ORDER BY updated_at DESC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(storage_error)?
        .into_iter()
        .map(project_from_row)
        .collect()
    }

    async fn find_project(&self, id: ProjectId) -> Result<Option<Project>> {
        sqlx::query(
            "SELECT id, name, kind, description, created_at, updated_at
             FROM projects WHERE id = ?",
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(storage_error)?
        .map(project_from_row)
        .transpose()
    }

    async fn insert_project(&self, project: &Project) -> Result<()> {
        sqlx::query(
            "INSERT INTO projects
             (id, name, kind, description, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(project.id.to_string())
        .bind(&project.name)
        .bind(project_kind(&project.kind))
        .bind(&project.description)
        .bind(project.created_at.to_rfc3339())
        .bind(project.updated_at.to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(storage_error)?;
        Ok(())
    }

    async fn list_documents(&self, project_id: ProjectId) -> Result<Vec<Document>> {
        sqlx::query(
            "SELECT id, project_id, parent_id, kind, title, content_json,
                    position, revision, exclude_from_compile, created_at, updated_at
             FROM documents WHERE project_id = ?
             ORDER BY parent_id, position, created_at",
        )
        .bind(project_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(storage_error)?
        .into_iter()
        .map(document_from_row)
        .collect()
    }

    async fn find_document(&self, id: DocumentId) -> Result<Option<Document>> {
        sqlx::query(
            "SELECT id, project_id, parent_id, kind, title, content_json,
                    position, revision, exclude_from_compile, created_at, updated_at
             FROM documents WHERE id = ?",
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(storage_error)?
        .map(document_from_row)
        .transpose()
    }

    async fn insert_document(&self, document: &Document) -> Result<()> {
        sqlx::query(
            "INSERT INTO documents
             (id, project_id, parent_id, kind, title, content_json, position,
              revision, exclude_from_compile, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(document.id.to_string())
        .bind(document.project_id.to_string())
        .bind(document.parent_id.map(|id| id.to_string()))
        .bind(document_kind(&document.kind))
        .bind(&document.title)
        .bind(document.content_json.to_string())
        .bind(document.position)
        .bind(document.revision)
        .bind(if document.exclude_from_compile { 1 } else { 0 })
        .bind(document.created_at.to_rfc3339())
        .bind(document.updated_at.to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(storage_error)?;
        Ok(())
    }

    async fn update_document(&self, document: &Document) -> Result<()> {
        let result = sqlx::query(
            "UPDATE documents
             SET title = ?, content_json = ?, parent_id = ?, position = ?,
                 revision = ?, exclude_from_compile = ?, updated_at = ?
             WHERE id = ?",
        )
        .bind(&document.title)
        .bind(document.content_json.to_string())
        .bind(document.parent_id.map(|id| id.to_string()))
        .bind(document.position)
        .bind(document.revision)
        .bind(if document.exclude_from_compile { 1 } else { 0 })
        .bind(document.updated_at.to_rfc3339())
        .bind(document.id.to_string())
        .execute(&self.pool)
        .await
        .map_err(storage_error)?;
        if result.rows_affected() == 0 {
            return Err(ApplicationError::NotFound { entity: "document" });
        }
        Ok(())
    }

    async fn list_facts(&self, project_id: ProjectId) -> Result<Vec<Fact>> {
        sqlx::query(
            "SELECT id, project_id, entity_key, predicate, value, modality, status,
                    valid_from, valid_to, source_document_id, source_start, source_end,
                    created_at, updated_at
             FROM facts WHERE project_id = ?
             ORDER BY entity_key, predicate, created_at",
        )
        .bind(project_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(storage_error)?
        .into_iter()
        .map(fact_from_row)
        .collect()
    }

    async fn insert_fact(&self, fact: &Fact) -> Result<()> {
        sqlx::query(
            "INSERT INTO facts
             (id, project_id, entity_key, predicate, value, modality, status,
              valid_from, valid_to, source_document_id, source_start, source_end,
              created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(fact.id.to_string())
        .bind(fact.project_id.to_string())
        .bind(&fact.entity_key)
        .bind(&fact.predicate)
        .bind(&fact.value)
        .bind(fact_modality(&fact.modality))
        .bind(fact_status(&fact.status))
        .bind(&fact.valid_from)
        .bind(&fact.valid_to)
        .bind(
            fact.source
                .as_ref()
                .map(|source| source.document_id.to_string()),
        )
        .bind(fact.source.as_ref().map(|source| i64::from(source.start)))
        .bind(fact.source.as_ref().map(|source| i64::from(source.end)))
        .bind(fact.created_at.to_rfc3339())
        .bind(fact.updated_at.to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(storage_error)?;
        Ok(())
    }

    async fn list_world_entries(&self, project_id: ProjectId) -> Result<Vec<WorldEntry>> {
        sqlx::query(
            "SELECT id, project_id, kind, title, summary, prompt, status,
                    board_x, board_y, map_x, map_y, hue, created_at, updated_at
             FROM world_entries WHERE project_id = ?
             ORDER BY created_at",
        )
        .bind(project_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(storage_error)?
        .into_iter()
        .map(world_entry_from_row)
        .collect()
    }

    async fn find_world_entry(&self, id: WorldEntryId) -> Result<Option<WorldEntry>> {
        sqlx::query(
            "SELECT id, project_id, kind, title, summary, prompt, status,
                    board_x, board_y, map_x, map_y, hue, created_at, updated_at
             FROM world_entries WHERE id = ?",
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(storage_error)?
        .map(world_entry_from_row)
        .transpose()
    }

    async fn insert_world_entry(&self, entry: &WorldEntry) -> Result<()> {
        sqlx::query(
            "INSERT INTO world_entries
             (id, project_id, kind, title, summary, prompt, status,
              board_x, board_y, map_x, map_y, hue, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(entry.id.to_string())
        .bind(entry.project_id.to_string())
        .bind(world_kind(&entry.kind))
        .bind(&entry.title)
        .bind(&entry.summary)
        .bind(entry.prompt.as_ref().map(idea_prompt))
        .bind(world_status(&entry.status))
        .bind(entry.board_x)
        .bind(entry.board_y)
        .bind(entry.map_x)
        .bind(entry.map_y)
        .bind(&entry.hue)
        .bind(entry.created_at.to_rfc3339())
        .bind(entry.updated_at.to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(storage_error)?;
        Ok(())
    }

    async fn update_world_entry(&self, entry: &WorldEntry) -> Result<()> {
        let result = sqlx::query(
            "UPDATE world_entries
             SET kind = ?, title = ?, summary = ?, prompt = ?, status = ?,
                 board_x = ?, board_y = ?, map_x = ?, map_y = ?, hue = ?,
                 updated_at = ?
             WHERE id = ?",
        )
        .bind(world_kind(&entry.kind))
        .bind(&entry.title)
        .bind(&entry.summary)
        .bind(entry.prompt.as_ref().map(idea_prompt))
        .bind(world_status(&entry.status))
        .bind(entry.board_x)
        .bind(entry.board_y)
        .bind(entry.map_x)
        .bind(entry.map_y)
        .bind(&entry.hue)
        .bind(entry.updated_at.to_rfc3339())
        .bind(entry.id.to_string())
        .execute(&self.pool)
        .await
        .map_err(storage_error)?;
        if result.rows_affected() == 0 {
            return Err(ApplicationError::NotFound {
                entity: "world entry",
            });
        }
        Ok(())
    }

    async fn list_entities(&self, project_id: ProjectId) -> Result<Vec<CanonEntity>> {
        sqlx::query(
            "SELECT id, project_id, kind, name, aliases, notes, created_at, updated_at
             FROM entities WHERE project_id = ? ORDER BY kind, name",
        )
        .bind(project_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(storage_error)?
        .into_iter()
        .map(entity_from_row)
        .collect()
    }

    async fn find_entity(&self, id: EntityId) -> Result<Option<CanonEntity>> {
        sqlx::query(
            "SELECT id, project_id, kind, name, aliases, notes, created_at, updated_at
             FROM entities WHERE id = ?",
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(storage_error)?
        .map(entity_from_row)
        .transpose()
    }

    async fn insert_entity(&self, entity: &CanonEntity) -> Result<()> {
        sqlx::query(
            "INSERT INTO entities
             (id, project_id, kind, name, aliases, notes, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(entity.id.to_string())
        .bind(entity.project_id.to_string())
        .bind(entity_kind(&entity.kind))
        .bind(&entity.name)
        .bind(serde_json::to_string(&entity.aliases).map_err(storage_error)?)
        .bind(&entity.notes)
        .bind(entity.created_at.to_rfc3339())
        .bind(entity.updated_at.to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(storage_error)?;
        Ok(())
    }

    async fn update_entity(&self, entity: &CanonEntity) -> Result<()> {
        let result = sqlx::query(
            "UPDATE entities SET kind = ?, name = ?, aliases = ?, notes = ?, updated_at = ?
             WHERE id = ?",
        )
        .bind(entity_kind(&entity.kind))
        .bind(&entity.name)
        .bind(serde_json::to_string(&entity.aliases).map_err(storage_error)?)
        .bind(&entity.notes)
        .bind(entity.updated_at.to_rfc3339())
        .bind(entity.id.to_string())
        .execute(&self.pool)
        .await
        .map_err(storage_error)?;
        if result.rows_affected() == 0 {
            return Err(ApplicationError::NotFound { entity: "entity" });
        }
        Ok(())
    }

    async fn list_events(&self, project_id: ProjectId) -> Result<Vec<TimelineEvent>> {
        sqlx::query(
            "SELECT id, project_id, entity_id, document_id, title, story_time,
                    reading_order, duration, notes, created_at, updated_at
             FROM timeline_events WHERE project_id = ?
             ORDER BY reading_order, created_at",
        )
        .bind(project_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(storage_error)?
        .into_iter()
        .map(event_from_row)
        .collect()
    }

    async fn find_event(&self, id: TimelineEventId) -> Result<Option<TimelineEvent>> {
        sqlx::query(
            "SELECT id, project_id, entity_id, document_id, title, story_time,
                    reading_order, duration, notes, created_at, updated_at
             FROM timeline_events WHERE id = ?",
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(storage_error)?
        .map(event_from_row)
        .transpose()
    }

    async fn insert_event(&self, event: &TimelineEvent) -> Result<()> {
        sqlx::query(
            "INSERT INTO timeline_events
             (id, project_id, entity_id, document_id, title, story_time,
              reading_order, duration, notes, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(event.id.to_string())
        .bind(event.project_id.to_string())
        .bind(event.entity_id.map(|id| id.to_string()))
        .bind(event.document_id.map(|id| id.to_string()))
        .bind(&event.title)
        .bind(&event.story_time)
        .bind(event.reading_order)
        .bind(&event.duration)
        .bind(&event.notes)
        .bind(event.created_at.to_rfc3339())
        .bind(event.updated_at.to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(storage_error)?;
        Ok(())
    }

    async fn update_event(&self, event: &TimelineEvent) -> Result<()> {
        let result = sqlx::query(
            "UPDATE timeline_events
             SET entity_id = ?, document_id = ?, title = ?, story_time = ?,
                 reading_order = ?, duration = ?, notes = ?, updated_at = ?
             WHERE id = ?",
        )
        .bind(event.entity_id.map(|id| id.to_string()))
        .bind(event.document_id.map(|id| id.to_string()))
        .bind(&event.title)
        .bind(&event.story_time)
        .bind(event.reading_order)
        .bind(&event.duration)
        .bind(&event.notes)
        .bind(event.updated_at.to_rfc3339())
        .bind(event.id.to_string())
        .execute(&self.pool)
        .await
        .map_err(storage_error)?;
        if result.rows_affected() == 0 {
            return Err(ApplicationError::NotFound { entity: "event" });
        }
        Ok(())
    }

    async fn list_comments(&self, document_id: DocumentId) -> Result<Vec<Comment>> {
        sqlx::query(
            "SELECT id, document_id, block_id, from_offset, to_offset, body, resolved,
                    created_at, updated_at
             FROM comments WHERE document_id = ? ORDER BY created_at",
        )
        .bind(document_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(storage_error)?
        .into_iter()
        .map(comment_from_row)
        .collect()
    }

    async fn find_comment(&self, id: CommentId) -> Result<Option<Comment>> {
        sqlx::query(
            "SELECT id, document_id, block_id, from_offset, to_offset, body, resolved,
                    created_at, updated_at
             FROM comments WHERE id = ?",
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(storage_error)?
        .map(comment_from_row)
        .transpose()
    }

    async fn insert_comment(&self, comment: &Comment) -> Result<()> {
        sqlx::query(
            "INSERT INTO comments
             (id, document_id, block_id, from_offset, to_offset, body, resolved,
              created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(comment.id.to_string())
        .bind(comment.document_id.to_string())
        .bind(&comment.block_id)
        .bind(comment.from_offset)
        .bind(comment.to_offset)
        .bind(&comment.body)
        .bind(if comment.resolved { 1 } else { 0 })
        .bind(comment.created_at.to_rfc3339())
        .bind(comment.updated_at.to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(storage_error)?;
        Ok(())
    }

    async fn update_comment(&self, comment: &Comment) -> Result<()> {
        let result =
            sqlx::query("UPDATE comments SET body = ?, resolved = ?, updated_at = ? WHERE id = ?")
                .bind(&comment.body)
                .bind(if comment.resolved { 1 } else { 0 })
                .bind(comment.updated_at.to_rfc3339())
                .bind(comment.id.to_string())
                .execute(&self.pool)
                .await
                .map_err(storage_error)?;
        if result.rows_affected() == 0 {
            return Err(ApplicationError::NotFound { entity: "comment" });
        }
        Ok(())
    }

    async fn next_op_seq(&self, document_id: DocumentId) -> Result<i64> {
        let row = sqlx::query(
            "SELECT COALESCE(MAX(seq), 0) AS seq FROM document_ops WHERE document_id = ?",
        )
        .bind(document_id.to_string())
        .fetch_one(&self.pool)
        .await
        .map_err(storage_error)?;
        let seq: i64 = row.try_get("seq").map_err(storage_error)?;
        Ok(seq + 1)
    }

    async fn insert_op(&self, op: &DocumentOp) -> Result<()> {
        sqlx::query(
            "INSERT INTO document_ops (id, document_id, seq, kind, op_json, actor, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(op.id.to_string())
        .bind(op.document_id.to_string())
        .bind(op.seq)
        .bind(&op.kind)
        .bind(op.op_json.to_string())
        .bind(&op.actor)
        .bind(op.created_at.to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(storage_error)?;
        Ok(())
    }

    async fn list_ops(&self, document_id: DocumentId) -> Result<Vec<DocumentOp>> {
        sqlx::query(
            "SELECT id, document_id, seq, kind, op_json, actor, created_at
             FROM document_ops WHERE document_id = ? ORDER BY seq",
        )
        .bind(document_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(storage_error)?
        .into_iter()
        .map(op_from_row)
        .collect()
    }

    async fn insert_snapshot(&self, snapshot: &DocumentSnapshot) -> Result<()> {
        sqlx::query(
            "INSERT INTO document_snapshots
             (id, document_id, revision, label, content_json, created_at)
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(snapshot.id.to_string())
        .bind(snapshot.document_id.to_string())
        .bind(snapshot.revision)
        .bind(&snapshot.label)
        .bind(snapshot.content_json.to_string())
        .bind(snapshot.created_at.to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(storage_error)?;
        Ok(())
    }

    async fn list_snapshots(&self, document_id: DocumentId) -> Result<Vec<DocumentSnapshot>> {
        sqlx::query(
            "SELECT id, document_id, revision, label, content_json, created_at
             FROM document_snapshots WHERE document_id = ? ORDER BY created_at DESC",
        )
        .bind(document_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(storage_error)?
        .into_iter()
        .map(snapshot_from_row)
        .collect()
    }

    async fn find_snapshot(&self, id: SnapshotId) -> Result<Option<DocumentSnapshot>> {
        sqlx::query(
            "SELECT id, document_id, revision, label, content_json, created_at
             FROM document_snapshots WHERE id = ?",
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(storage_error)?
        .map(snapshot_from_row)
        .transpose()
    }

    async fn list_suppressions(&self, project_id: ProjectId) -> Result<Vec<Suppression>> {
        sqlx::query(
            "SELECT id, project_id, rule_code, document_id, entity_key, reason, created_at
             FROM suppressions WHERE project_id = ? ORDER BY created_at DESC",
        )
        .bind(project_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(storage_error)?
        .into_iter()
        .map(suppression_from_row)
        .collect()
    }

    async fn insert_suppression(&self, suppression: &Suppression) -> Result<()> {
        sqlx::query(
            "INSERT INTO suppressions
             (id, project_id, rule_code, document_id, entity_key, reason, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(suppression.id.to_string())
        .bind(suppression.project_id.to_string())
        .bind(&suppression.rule_code)
        .bind(suppression.document_id.map(|id| id.to_string()))
        .bind(&suppression.entity_key)
        .bind(&suppression.reason)
        .bind(suppression.created_at.to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(storage_error)?;
        Ok(())
    }

    async fn list_packs(&self) -> Result<Vec<MethodPack>> {
        sqlx::query(
            "SELECT id, pack_key, name, version, summary, enabled, manifest_json, created_at
             FROM method_packs ORDER BY name",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(storage_error)?
        .into_iter()
        .map(pack_from_row)
        .collect()
    }

    async fn update_pack(&self, pack: &MethodPack) -> Result<()> {
        sqlx::query("UPDATE method_packs SET enabled = ? WHERE id = ?")
            .bind(if pack.enabled { 1 } else { 0 })
            .bind(pack.id.to_string())
            .execute(&self.pool)
            .await
            .map_err(storage_error)?;
        Ok(())
    }

    async fn find_pack(&self, id: MethodPackId) -> Result<Option<MethodPack>> {
        sqlx::query(
            "SELECT id, pack_key, name, version, summary, enabled, manifest_json, created_at
             FROM method_packs WHERE id = ?",
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(storage_error)?
        .map(pack_from_row)
        .transpose()
    }

    async fn list_tracked_changes(&self, document_id: DocumentId) -> Result<Vec<TrackedChange>> {
        sqlx::query(
            "SELECT id, document_id, kind, block_id, text, actor, accepted, created_at
             FROM tracked_changes WHERE document_id = ? ORDER BY created_at",
        )
        .bind(document_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(storage_error)?
        .into_iter()
        .map(tracked_from_row)
        .collect()
    }

    async fn insert_tracked_change(&self, change: &TrackedChange) -> Result<()> {
        sqlx::query(
            "INSERT INTO tracked_changes
             (id, document_id, kind, block_id, text, actor, accepted, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(change.id.to_string())
        .bind(change.document_id.to_string())
        .bind(tracked_kind(&change.kind))
        .bind(&change.block_id)
        .bind(&change.text)
        .bind(&change.actor)
        .bind(change.accepted.map(|value| if value { 1 } else { 0 }))
        .bind(change.created_at.to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(storage_error)?;
        Ok(())
    }

    async fn update_tracked_change(&self, change: &TrackedChange) -> Result<()> {
        sqlx::query("UPDATE tracked_changes SET accepted = ? WHERE id = ?")
            .bind(change.accepted.map(|value| if value { 1 } else { 0 }))
            .bind(change.id.to_string())
            .execute(&self.pool)
            .await
            .map_err(storage_error)?;
        Ok(())
    }

    async fn find_tracked_change(&self, id: TrackedChangeId) -> Result<Option<TrackedChange>> {
        sqlx::query(
            "SELECT id, document_id, kind, block_id, text, actor, accepted, created_at
             FROM tracked_changes WHERE id = ?",
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(storage_error)?
        .map(tracked_from_row)
        .transpose()
    }
}

fn project_from_row(row: sqlx::sqlite::SqliteRow) -> Result<Project> {
    Ok(Project {
        id: ProjectId(parse_uuid(row.try_get("id").map_err(storage_error)?)?),
        name: row.try_get("name").map_err(storage_error)?,
        kind: parse_project_kind(row.try_get("kind").map_err(storage_error)?)?,
        description: row.try_get("description").map_err(storage_error)?,
        created_at: parse_time(row.try_get("created_at").map_err(storage_error)?)?,
        updated_at: parse_time(row.try_get("updated_at").map_err(storage_error)?)?,
    })
}

fn document_from_row(row: sqlx::sqlite::SqliteRow) -> Result<Document> {
    let parent_id: Option<String> = row.try_get("parent_id").map_err(storage_error)?;
    let content: String = row.try_get("content_json").map_err(storage_error)?;
    Ok(Document {
        id: DocumentId(parse_uuid(row.try_get("id").map_err(storage_error)?)?),
        project_id: ProjectId(parse_uuid(
            row.try_get("project_id").map_err(storage_error)?,
        )?),
        parent_id: parent_id.map(parse_uuid).transpose()?.map(DocumentId),
        kind: parse_document_kind(row.try_get("kind").map_err(storage_error)?)?,
        title: row.try_get("title").map_err(storage_error)?,
        content_json: serde_json::from_str(&content).map_err(storage_error)?,
        position: row.try_get("position").map_err(storage_error)?,
        revision: row.try_get("revision").map_err(storage_error)?,
        exclude_from_compile: row.try_get::<i64, _>("exclude_from_compile").unwrap_or(0) != 0,
        created_at: parse_time(row.try_get("created_at").map_err(storage_error)?)?,
        updated_at: parse_time(row.try_get("updated_at").map_err(storage_error)?)?,
    })
}

fn storage_error(error: impl std::fmt::Display) -> ApplicationError {
    ApplicationError::Storage(error.to_string())
}

fn parse_uuid(value: String) -> Result<Uuid> {
    Uuid::parse_str(&value).map_err(storage_error)
}

fn parse_time(value: String) -> Result<DateTime<Utc>> {
    value.parse().map_err(storage_error)
}

const fn project_kind(kind: &ProjectKind) -> &'static str {
    match kind {
        ProjectKind::Novel => "novel",
        ProjectKind::Screenplay => "screenplay",
        ProjectKind::NonFiction => "non_fiction",
        ProjectKind::Other => "other",
    }
}

fn parse_project_kind(value: String) -> Result<ProjectKind> {
    match value.as_str() {
        "novel" => Ok(ProjectKind::Novel),
        "screenplay" => Ok(ProjectKind::Screenplay),
        "non_fiction" => Ok(ProjectKind::NonFiction),
        "other" => Ok(ProjectKind::Other),
        _ => Err(ApplicationError::Storage(format!(
            "unknown project kind: {value}"
        ))),
    }
}

const fn document_kind(kind: &DocumentKind) -> &'static str {
    match kind {
        DocumentKind::Manuscript => "manuscript",
        DocumentKind::Chapter => "chapter",
        DocumentKind::Scene => "scene",
        DocumentKind::Wiki => "wiki",
        DocumentKind::Note => "note",
        DocumentKind::Source => "source",
    }
}

fn parse_document_kind(value: String) -> Result<DocumentKind> {
    match value.as_str() {
        "manuscript" => Ok(DocumentKind::Manuscript),
        "chapter" => Ok(DocumentKind::Chapter),
        "scene" => Ok(DocumentKind::Scene),
        "wiki" => Ok(DocumentKind::Wiki),
        "note" => Ok(DocumentKind::Note),
        "source" => Ok(DocumentKind::Source),
        _ => Err(ApplicationError::Storage(format!(
            "unknown document kind: {value}"
        ))),
    }
}

fn fact_from_row(row: sqlx::sqlite::SqliteRow) -> Result<Fact> {
    let source_document_id: Option<String> =
        row.try_get("source_document_id").map_err(storage_error)?;
    let source_start: Option<i64> = row.try_get("source_start").map_err(storage_error)?;
    let source_end: Option<i64> = row.try_get("source_end").map_err(storage_error)?;
    let source = match source_document_id {
        Some(document_id) => Some(SourceAnchor {
            document_id: DocumentId(parse_uuid(document_id)?),
            start: source_start.unwrap_or(0) as u32,
            end: source_end.unwrap_or(0) as u32,
        }),
        None => None,
    };
    Ok(Fact {
        id: FactId(parse_uuid(row.try_get("id").map_err(storage_error)?)?),
        project_id: ProjectId(parse_uuid(
            row.try_get("project_id").map_err(storage_error)?,
        )?),
        entity_key: row.try_get("entity_key").map_err(storage_error)?,
        predicate: row.try_get("predicate").map_err(storage_error)?,
        value: row.try_get("value").map_err(storage_error)?,
        modality: parse_fact_modality(row.try_get("modality").map_err(storage_error)?)?,
        status: parse_fact_status(row.try_get("status").map_err(storage_error)?)?,
        valid_from: row.try_get("valid_from").map_err(storage_error)?,
        valid_to: row.try_get("valid_to").map_err(storage_error)?,
        source,
        created_at: parse_time(row.try_get("created_at").map_err(storage_error)?)?,
        updated_at: parse_time(row.try_get("updated_at").map_err(storage_error)?)?,
    })
}

const fn fact_modality(modality: &FactModality) -> &'static str {
    match modality {
        FactModality::Established => "established",
        FactModality::Inferred => "inferred",
        FactModality::Contested => "contested",
        FactModality::AuthorOverride => "author_override",
    }
}

fn parse_fact_modality(value: String) -> Result<FactModality> {
    match value.as_str() {
        "established" => Ok(FactModality::Established),
        "inferred" => Ok(FactModality::Inferred),
        "contested" => Ok(FactModality::Contested),
        "author_override" => Ok(FactModality::AuthorOverride),
        _ => Err(ApplicationError::Storage(format!(
            "unknown fact modality: {value}"
        ))),
    }
}

const fn fact_status(status: &FactStatus) -> &'static str {
    match status {
        FactStatus::Proposed => "proposed",
        FactStatus::Confirmed => "confirmed",
        FactStatus::Deprecated => "deprecated",
    }
}

fn parse_fact_status(value: String) -> Result<FactStatus> {
    match value.as_str() {
        "proposed" => Ok(FactStatus::Proposed),
        "confirmed" => Ok(FactStatus::Confirmed),
        "deprecated" => Ok(FactStatus::Deprecated),
        _ => Err(ApplicationError::Storage(format!(
            "unknown fact status: {value}"
        ))),
    }
}

fn world_entry_from_row(row: sqlx::sqlite::SqliteRow) -> Result<WorldEntry> {
    let prompt: Option<String> = row.try_get("prompt").map_err(storage_error)?;
    Ok(WorldEntry {
        id: WorldEntryId(parse_uuid(row.try_get("id").map_err(storage_error)?)?),
        project_id: ProjectId(parse_uuid(
            row.try_get("project_id").map_err(storage_error)?,
        )?),
        kind: parse_world_kind(row.try_get("kind").map_err(storage_error)?)?,
        title: row.try_get("title").map_err(storage_error)?,
        summary: row.try_get("summary").map_err(storage_error)?,
        prompt: prompt.map(parse_idea_prompt).transpose()?,
        status: parse_world_status(row.try_get("status").map_err(storage_error)?)?,
        board_x: row.try_get("board_x").map_err(storage_error)?,
        board_y: row.try_get("board_y").map_err(storage_error)?,
        map_x: row.try_get("map_x").map_err(storage_error)?,
        map_y: row.try_get("map_y").map_err(storage_error)?,
        hue: row.try_get("hue").map_err(storage_error)?,
        created_at: parse_time(row.try_get("created_at").map_err(storage_error)?)?,
        updated_at: parse_time(row.try_get("updated_at").map_err(storage_error)?)?,
    })
}

const fn world_kind(kind: &WorldKind) -> &'static str {
    match kind {
        WorldKind::Place => "place",
        WorldKind::Culture => "culture",
        WorldKind::System => "system",
        WorldKind::Creature => "creature",
        WorldKind::Item => "item",
        WorldKind::Question => "question",
        WorldKind::Fragment => "fragment",
    }
}

fn parse_world_kind(value: String) -> Result<WorldKind> {
    match value.as_str() {
        "place" => Ok(WorldKind::Place),
        "culture" => Ok(WorldKind::Culture),
        "system" => Ok(WorldKind::System),
        "creature" => Ok(WorldKind::Creature),
        "item" => Ok(WorldKind::Item),
        "question" => Ok(WorldKind::Question),
        "fragment" => Ok(WorldKind::Fragment),
        _ => Err(ApplicationError::Storage(format!(
            "unknown world kind: {value}"
        ))),
    }
}

const fn world_status(status: &WorldStatus) -> &'static str {
    match status {
        WorldStatus::Spark => "spark",
        WorldStatus::Sketched => "sketched",
        WorldStatus::Settled => "settled",
    }
}

fn parse_world_status(value: String) -> Result<WorldStatus> {
    match value.as_str() {
        "spark" => Ok(WorldStatus::Spark),
        "sketched" => Ok(WorldStatus::Sketched),
        "settled" => Ok(WorldStatus::Settled),
        _ => Err(ApplicationError::Storage(format!(
            "unknown world status: {value}"
        ))),
    }
}

const fn idea_prompt(prompt: &IdeaPrompt) -> &'static str {
    match prompt {
        IdeaPrompt::Why => "why",
        IdeaPrompt::How => "how",
        IdeaPrompt::WhatResult => "what_result",
    }
}

fn parse_idea_prompt(value: String) -> Result<IdeaPrompt> {
    match value.as_str() {
        "why" => Ok(IdeaPrompt::Why),
        "how" => Ok(IdeaPrompt::How),
        "what_result" => Ok(IdeaPrompt::WhatResult),
        _ => Err(ApplicationError::Storage(format!(
            "unknown idea prompt: {value}"
        ))),
    }
}

fn entity_from_row(row: sqlx::sqlite::SqliteRow) -> Result<CanonEntity> {
    let aliases: String = row.try_get("aliases").map_err(storage_error)?;
    Ok(CanonEntity {
        id: EntityId(parse_uuid(row.try_get("id").map_err(storage_error)?)?),
        project_id: ProjectId(parse_uuid(
            row.try_get("project_id").map_err(storage_error)?,
        )?),
        kind: parse_entity_kind(row.try_get("kind").map_err(storage_error)?)?,
        name: row.try_get("name").map_err(storage_error)?,
        aliases: serde_json::from_str(&aliases).map_err(storage_error)?,
        notes: row.try_get("notes").map_err(storage_error)?,
        created_at: parse_time(row.try_get("created_at").map_err(storage_error)?)?,
        updated_at: parse_time(row.try_get("updated_at").map_err(storage_error)?)?,
    })
}

const fn entity_kind(kind: &EntityKind) -> &'static str {
    match kind {
        EntityKind::Character => "character",
        EntityKind::Place => "place",
        EntityKind::Event => "event",
        EntityKind::Item => "item",
        EntityKind::Rule => "rule",
        EntityKind::Source => "source",
        EntityKind::Other => "other",
    }
}

fn parse_entity_kind(value: String) -> Result<EntityKind> {
    match value.as_str() {
        "character" => Ok(EntityKind::Character),
        "place" => Ok(EntityKind::Place),
        "event" => Ok(EntityKind::Event),
        "item" => Ok(EntityKind::Item),
        "rule" => Ok(EntityKind::Rule),
        "source" => Ok(EntityKind::Source),
        "other" => Ok(EntityKind::Other),
        _ => Err(ApplicationError::Storage(format!(
            "unknown entity kind: {value}"
        ))),
    }
}

fn event_from_row(row: sqlx::sqlite::SqliteRow) -> Result<TimelineEvent> {
    let entity_id: Option<String> = row.try_get("entity_id").map_err(storage_error)?;
    let document_id: Option<String> = row.try_get("document_id").map_err(storage_error)?;
    Ok(TimelineEvent {
        id: TimelineEventId(parse_uuid(row.try_get("id").map_err(storage_error)?)?),
        project_id: ProjectId(parse_uuid(
            row.try_get("project_id").map_err(storage_error)?,
        )?),
        entity_id: entity_id.map(parse_uuid).transpose()?.map(EntityId),
        document_id: document_id.map(parse_uuid).transpose()?.map(DocumentId),
        title: row.try_get("title").map_err(storage_error)?,
        story_time: row.try_get("story_time").map_err(storage_error)?,
        reading_order: row.try_get("reading_order").map_err(storage_error)?,
        duration: row.try_get("duration").map_err(storage_error)?,
        notes: row.try_get("notes").map_err(storage_error)?,
        created_at: parse_time(row.try_get("created_at").map_err(storage_error)?)?,
        updated_at: parse_time(row.try_get("updated_at").map_err(storage_error)?)?,
    })
}

fn comment_from_row(row: sqlx::sqlite::SqliteRow) -> Result<Comment> {
    let resolved: i64 = row.try_get("resolved").map_err(storage_error)?;
    Ok(Comment {
        id: CommentId(parse_uuid(row.try_get("id").map_err(storage_error)?)?),
        document_id: DocumentId(parse_uuid(
            row.try_get("document_id").map_err(storage_error)?,
        )?),
        block_id: row.try_get("block_id").map_err(storage_error)?,
        from_offset: row.try_get("from_offset").map_err(storage_error)?,
        to_offset: row.try_get("to_offset").map_err(storage_error)?,
        body: row.try_get("body").map_err(storage_error)?,
        resolved: resolved != 0,
        created_at: parse_time(row.try_get("created_at").map_err(storage_error)?)?,
        updated_at: parse_time(row.try_get("updated_at").map_err(storage_error)?)?,
    })
}

fn op_from_row(row: sqlx::sqlite::SqliteRow) -> Result<DocumentOp> {
    let op_json: String = row.try_get("op_json").map_err(storage_error)?;
    Ok(DocumentOp {
        id: DocumentOpId(parse_uuid(row.try_get("id").map_err(storage_error)?)?),
        document_id: DocumentId(parse_uuid(
            row.try_get("document_id").map_err(storage_error)?,
        )?),
        seq: row.try_get("seq").map_err(storage_error)?,
        kind: row.try_get("kind").map_err(storage_error)?,
        op_json: serde_json::from_str(&op_json).map_err(storage_error)?,
        actor: row.try_get("actor").map_err(storage_error)?,
        created_at: parse_time(row.try_get("created_at").map_err(storage_error)?)?,
    })
}

fn snapshot_from_row(row: sqlx::sqlite::SqliteRow) -> Result<DocumentSnapshot> {
    let content: String = row.try_get("content_json").map_err(storage_error)?;
    Ok(DocumentSnapshot {
        id: SnapshotId(parse_uuid(row.try_get("id").map_err(storage_error)?)?),
        document_id: DocumentId(parse_uuid(
            row.try_get("document_id").map_err(storage_error)?,
        )?),
        revision: row.try_get("revision").map_err(storage_error)?,
        label: row.try_get("label").map_err(storage_error)?,
        content_json: serde_json::from_str(&content).map_err(storage_error)?,
        created_at: parse_time(row.try_get("created_at").map_err(storage_error)?)?,
    })
}

fn suppression_from_row(row: sqlx::sqlite::SqliteRow) -> Result<Suppression> {
    let document_id: Option<String> = row.try_get("document_id").map_err(storage_error)?;
    Ok(Suppression {
        id: SuppressionId(parse_uuid(row.try_get("id").map_err(storage_error)?)?),
        project_id: ProjectId(parse_uuid(
            row.try_get("project_id").map_err(storage_error)?,
        )?),
        rule_code: row.try_get("rule_code").map_err(storage_error)?,
        document_id: document_id.map(parse_uuid).transpose()?.map(DocumentId),
        entity_key: row.try_get("entity_key").map_err(storage_error)?,
        reason: row.try_get("reason").map_err(storage_error)?,
        created_at: parse_time(row.try_get("created_at").map_err(storage_error)?)?,
    })
}

fn pack_from_row(row: sqlx::sqlite::SqliteRow) -> Result<MethodPack> {
    let manifest: String = row.try_get("manifest_json").map_err(storage_error)?;
    let enabled: i64 = row.try_get("enabled").map_err(storage_error)?;
    Ok(MethodPack {
        id: MethodPackId(parse_uuid(row.try_get("id").map_err(storage_error)?)?),
        pack_key: row.try_get("pack_key").map_err(storage_error)?,
        name: row.try_get("name").map_err(storage_error)?,
        version: row.try_get("version").map_err(storage_error)?,
        summary: row.try_get("summary").map_err(storage_error)?,
        enabled: enabled != 0,
        manifest: serde_json::from_str(&manifest).map_err(storage_error)?,
        created_at: parse_time(row.try_get("created_at").map_err(storage_error)?)?,
    })
}

fn tracked_from_row(row: sqlx::sqlite::SqliteRow) -> Result<TrackedChange> {
    let accepted: Option<i64> = row.try_get("accepted").map_err(storage_error)?;
    Ok(TrackedChange {
        id: TrackedChangeId(parse_uuid(row.try_get("id").map_err(storage_error)?)?),
        document_id: DocumentId(parse_uuid(
            row.try_get("document_id").map_err(storage_error)?,
        )?),
        kind: parse_tracked_kind(row.try_get("kind").map_err(storage_error)?)?,
        block_id: row.try_get("block_id").map_err(storage_error)?,
        text: row.try_get("text").map_err(storage_error)?,
        actor: row.try_get("actor").map_err(storage_error)?,
        accepted: accepted.map(|value| value != 0),
        created_at: parse_time(row.try_get("created_at").map_err(storage_error)?)?,
    })
}

const fn tracked_kind(kind: &TrackedKind) -> &'static str {
    match kind {
        TrackedKind::Insert => "insert",
        TrackedKind::Delete => "delete",
    }
}

fn parse_tracked_kind(value: String) -> Result<TrackedKind> {
    match value.as_str() {
        "insert" => Ok(TrackedKind::Insert),
        "delete" => Ok(TrackedKind::Delete),
        _ => Err(ApplicationError::Storage(format!(
            "unknown tracked kind: {value}"
        ))),
    }
}
