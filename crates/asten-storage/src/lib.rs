//! The SQLite adapter: one file on the writer's machine, migrations applied on
//! open, and every table the application reads and writes.

use std::{path::Path, time::Duration};

use asten_application::{ApplicationError, Repository, Result};
use asten_blocks::{DocState, EventPayload, EventRecord};
use asten_domain::{
    AuthToken, AuthTokenId, Baseline, BaselineId, BlockSnapshot, CanonEntity, Comment, CommentId,
    Document, DocumentId, DocumentKind, DocumentOp, DocumentOpId, DocumentSnapshot, Draft, DraftId,
    EntityId, EntityKind, Fact, FactId, FactModality, FactStatus, IdeaPrompt, InvariantId,
    MethodPack, MethodPackId, OwnerCredential, PairingCode, PairingCodeId, Project, ProjectId,
    ProjectKind, ProjectSettings, RouteId, ScenePlacement, SnapshotId, SourceAnchor,
    StoredInvariant, StoredRoute, Suppression, SuppressionId, SuppressionScope, TimelineEvent,
    TimelineEventId, TokenKind, TrackedChange, TrackedChangeId, TrackedKind, WorldEntry,
    WorldEntryId, WorldKind, WorldStatus,
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{
    Row, SqlitePool,
    sqlite::{
        SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteRow, SqliteSynchronous,
    },
};
use uuid::Uuid;

#[derive(Clone)]
pub struct SqliteRepository {
    pool: SqlitePool,
}

impl SqliteRepository {
    /// Open (or create) the database file. Write-ahead logging lets the desk
    /// read while a commit is being written; `synchronous=NORMAL` is durable
    /// across application crashes under WAL and costs one fsync per checkpoint
    /// rather than per commit; a five-second busy timeout outlasts any single
    /// commit so a concurrent writer waits instead of failing.
    pub async fn open(path: impl AsRef<Path>) -> Result<Self> {
        let options = SqliteConnectOptions::new()
            .filename(path)
            .create_if_missing(true)
            .foreign_keys(true)
            .journal_mode(SqliteJournalMode::Wal)
            .synchronous(SqliteSynchronous::Normal)
            .busy_timeout(Duration::from_secs(5));
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

    /// An in-memory database, for tests and previews.
    pub async fn in_memory() -> Result<Self> {
        let options = SqliteConnectOptions::new()
            .in_memory(true)
            .foreign_keys(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
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

const DOCUMENT_COLUMNS: &str = "id, project_id, parent_id, kind, title, content_json, position, revision, exclude_from_compile, scene_json, head_seq, chain_hash, created_at, updated_at";
const FACT_COLUMNS: &str = "id, project_id, entity_key, predicate, value, modality, status, valid_from, valid_to, source_document_id, source_start, source_end, context_json, confidence, anchor_json, provenance_json, settled_by, settled_at, created_at, updated_at";
const EVENT_COLUMNS: &str = "id, project_id, entity_id, document_id, title, story_time, reading_order, duration, notes, kind_json, participants_json, context_json, status, confidence, anchor_json, tick, tick_lo, tick_hi, provenance_json, created_at, updated_at";
const SUPPRESSION_COLUMNS: &str = "id, project_id, rule_code, document_id, entity_key, reason, scope, diagnostic_id, evidence_fingerprint, stale, created_at";
const SNAPSHOT_COLUMNS: &str =
    "id, document_id, revision, label, content_json, seq, chain_hash, merkle_root, created_at";
const ENTITY_COLUMNS: &str =
    "id, project_id, kind, name, aliases, notes, introduced_at, created_at, updated_at";
const TOKEN_COLUMNS: &str =
    "id, hash, kind, label, created_at, expires_at, revoked_at, last_used_at";
const PAIRING_COLUMNS: &str =
    "id, code_hash, salt, label, grants, expires_at, consumed_at, attempts, created_at";

/// The newest schema version this build knows how to open.
#[must_use]
pub fn latest_migration() -> i64 {
    sqlx::migrate!("./migrations")
        .iter()
        .map(|migration| migration.version)
        .max()
        .unwrap_or(0)
}

#[async_trait]
impl Repository for SqliteRepository {
    // ───────────────────────── projects ─────────────────────────

    async fn list_projects(&self) -> Result<Vec<Project>> {
        sqlx::query("SELECT id, name, kind, description, created_at, updated_at FROM projects ORDER BY updated_at DESC")
            .fetch_all(&self.pool)
            .await
            .map_err(storage_error)?
            .into_iter()
            .map(project_from_row)
            .collect()
    }

    async fn find_project(&self, id: ProjectId) -> Result<Option<Project>> {
        sqlx::query(
            "SELECT id, name, kind, description, created_at, updated_at FROM projects WHERE id = ?",
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(storage_error)?
        .map(project_from_row)
        .transpose()
    }

    async fn insert_project(&self, project: &Project) -> Result<()> {
        sqlx::query("INSERT INTO projects (id, name, kind, description, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)")
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

    async fn update_project(&self, project: &Project) -> Result<()> {
        sqlx::query(
            "UPDATE projects SET name = ?, kind = ?, description = ?, updated_at = ? WHERE id = ?",
        )
        .bind(&project.name)
        .bind(project_kind(&project.kind))
        .bind(&project.description)
        .bind(project.updated_at.to_rfc3339())
        .bind(project.id.to_string())
        .execute(&self.pool)
        .await
        .map_err(storage_error)?;
        Ok(())
    }

    async fn delete_project(&self, id: ProjectId) -> Result<()> {
        sqlx::query("DELETE FROM projects WHERE id = ?")
            .bind(id.to_string())
            .execute(&self.pool)
            .await
            .map_err(storage_error)?;
        Ok(())
    }

    // ───────────────────────── documents ─────────────────────────

    async fn list_documents(&self, project_id: ProjectId) -> Result<Vec<Document>> {
        sqlx::query(&format!("SELECT {DOCUMENT_COLUMNS} FROM documents WHERE project_id = ? ORDER BY parent_id, position, created_at"))
            .bind(project_id.to_string())
            .fetch_all(&self.pool)
            .await
            .map_err(storage_error)?
            .into_iter()
            .map(document_from_row)
            .collect()
    }

    async fn find_document(&self, id: DocumentId) -> Result<Option<Document>> {
        sqlx::query(&format!(
            "SELECT {DOCUMENT_COLUMNS} FROM documents WHERE id = ?"
        ))
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(storage_error)?
        .map(document_from_row)
        .transpose()
    }

    async fn insert_document(&self, document: &Document) -> Result<()> {
        sqlx::query(
            "INSERT INTO documents (id, project_id, parent_id, kind, title, content_json, position, revision, exclude_from_compile, scene_json, state_json, head_seq, chain_hash, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, '', ?, ?, ?, ?)",
        )
        .bind(document.id.to_string())
        .bind(document.project_id.to_string())
        .bind(document.parent_id.map(|id| id.to_string()))
        .bind(document_kind(&document.kind))
        .bind(&document.title)
        .bind(document.content_json.to_string())
        .bind(document.position)
        .bind(document.revision)
        .bind(i64::from(document.exclude_from_compile))
        .bind(json_string(&document.scene))
        .bind(document.head_seq)
        .bind(&document.chain_hash)
        .bind(document.created_at.to_rfc3339())
        .bind(document.updated_at.to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(storage_error)?;
        Ok(())
    }

    async fn update_document(&self, document: &Document) -> Result<()> {
        let result = sqlx::query(
            "UPDATE documents SET title = ?, content_json = ?, parent_id = ?, position = ?, revision = ?, exclude_from_compile = ?, scene_json = ?, head_seq = ?, chain_hash = ?, updated_at = ?
             WHERE id = ?",
        )
        .bind(&document.title)
        .bind(document.content_json.to_string())
        .bind(document.parent_id.map(|id| id.to_string()))
        .bind(document.position)
        .bind(document.revision)
        .bind(i64::from(document.exclude_from_compile))
        .bind(json_string(&document.scene))
        .bind(document.head_seq)
        .bind(&document.chain_hash)
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

    async fn delete_document(&self, id: DocumentId) -> Result<()> {
        sqlx::query("DELETE FROM documents WHERE id = ?")
            .bind(id.to_string())
            .execute(&self.pool)
            .await
            .map_err(storage_error)?;
        Ok(())
    }

    async fn commit_document(
        &self,
        document: &Document,
        state: &DocState,
        events: &[EventRecord],
    ) -> Result<()> {
        let mut tx = self.pool.begin().await.map_err(storage_error)?;
        let result = sqlx::query(
            "UPDATE documents SET title = ?, content_json = ?, parent_id = ?, position = ?, revision = ?, exclude_from_compile = ?, scene_json = ?, state_json = ?, head_seq = ?, chain_hash = ?, updated_at = ?
             WHERE id = ? AND head_seq = ?",
        )
        .bind(&document.title)
        .bind(document.content_json.to_string())
        .bind(document.parent_id.map(|id| id.to_string()))
        .bind(document.position)
        .bind(document.revision)
        .bind(i64::from(document.exclude_from_compile))
        .bind(json_string(&document.scene))
        .bind(json_string(state))
        .bind(document.head_seq)
        .bind(&document.chain_hash)
        .bind(document.updated_at.to_rfc3339())
        .bind(document.id.to_string())
        .bind(events.first().map_or(document.head_seq, |first| first.seq - 1))
        .execute(&mut *tx)
        .await
        .map_err(storage_error)?;
        if result.rows_affected() == 0 {
            return Err(ApplicationError::Conflict(
                "the document changed while saving; reload and try again".into(),
            ));
        }
        for record in events {
            sqlx::query(
                "INSERT INTO block_events (document_id, seq, event_id, actor, created_at, payload_json, content_hash, prev_chain_hash, chain_hash)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(document.id.to_string())
            .bind(record.seq)
            .bind(&record.id)
            .bind(&record.actor)
            .bind(&record.at)
            .bind(json_string(&record.payload))
            .bind(&record.content_hash)
            .bind(&record.prev_chain_hash)
            .bind(&record.chain_hash)
            .execute(&mut *tx)
            .await
            .map_err(storage_error)?;
        }
        tx.commit().await.map_err(storage_error)?;
        Ok(())
    }

    async fn document_state(&self, id: DocumentId) -> Result<Option<DocState>> {
        let row = sqlx::query("SELECT state_json FROM documents WHERE id = ?")
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(storage_error)?;
        let Some(row) = row else {
            return Ok(None);
        };
        let text: String = row.try_get("state_json").map_err(storage_error)?;
        if text.trim().is_empty() {
            return Ok(None);
        }
        serde_json::from_str(&text).map(Some).map_err(storage_error)
    }

    async fn list_block_events(
        &self,
        document_id: DocumentId,
        from_seq: i64,
    ) -> Result<Vec<EventRecord>> {
        sqlx::query("SELECT seq, event_id, actor, created_at, payload_json, content_hash, prev_chain_hash, chain_hash FROM block_events WHERE document_id = ? AND seq >= ? ORDER BY seq")
            .bind(document_id.to_string())
            .bind(from_seq)
            .fetch_all(&self.pool)
            .await
            .map_err(storage_error)?
            .into_iter()
            .map(event_record_from_row)
            .collect()
    }

    async fn insert_block_snapshot(&self, snapshot: &BlockSnapshot) -> Result<()> {
        sqlx::query("INSERT OR REPLACE INTO block_snapshots (id, document_id, seq, chain_hash, merkle_root, state_json, created_at) VALUES (?, ?, ?, ?, ?, ?, ?)")
            .bind(snapshot.id.to_string())
            .bind(snapshot.document_id.to_string())
            .bind(snapshot.seq)
            .bind(&snapshot.chain_hash)
            .bind(&snapshot.merkle_root)
            .bind(snapshot.state_json.to_string())
            .bind(snapshot.created_at.to_rfc3339())
            .execute(&self.pool)
            .await
            .map_err(storage_error)?;
        Ok(())
    }

    async fn latest_block_snapshot(
        &self,
        document_id: DocumentId,
        through_seq: Option<i64>,
    ) -> Result<Option<BlockSnapshot>> {
        let row = match through_seq {
            Some(seq) => {
                sqlx::query("SELECT id, document_id, seq, chain_hash, merkle_root, state_json, created_at FROM block_snapshots WHERE document_id = ? AND seq <= ? ORDER BY seq DESC LIMIT 1")
                    .bind(document_id.to_string())
                    .bind(seq)
                    .fetch_optional(&self.pool)
                    .await
            }
            None => {
                sqlx::query("SELECT id, document_id, seq, chain_hash, merkle_root, state_json, created_at FROM block_snapshots WHERE document_id = ? ORDER BY seq DESC LIMIT 1")
                    .bind(document_id.to_string())
                    .fetch_optional(&self.pool)
                    .await
            }
        }
        .map_err(storage_error)?;
        row.map(block_snapshot_from_row).transpose()
    }

    // ───────────────────────── facts ─────────────────────────

    async fn list_facts(&self, project_id: ProjectId) -> Result<Vec<Fact>> {
        sqlx::query(&format!("SELECT {FACT_COLUMNS} FROM facts WHERE project_id = ? ORDER BY entity_key, predicate, created_at"))
            .bind(project_id.to_string())
            .fetch_all(&self.pool)
            .await
            .map_err(storage_error)?
            .into_iter()
            .map(fact_from_row)
            .collect()
    }

    async fn find_fact(&self, id: FactId) -> Result<Option<Fact>> {
        sqlx::query(&format!("SELECT {FACT_COLUMNS} FROM facts WHERE id = ?"))
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(storage_error)?
            .map(fact_from_row)
            .transpose()
    }

    async fn insert_fact(&self, fact: &Fact) -> Result<()> {
        sqlx::query(&format!(
            "INSERT INTO facts ({FACT_COLUMNS}) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        ))
        .bind(fact.id.to_string())
        .bind(fact.project_id.to_string())
        .bind(&fact.entity_key)
        .bind(&fact.predicate)
        .bind(&fact.value)
        .bind(fact_modality(&fact.modality))
        .bind(fact_status(fact.status))
        .bind(&fact.valid_from)
        .bind(&fact.valid_to)
        .bind(fact.source.as_ref().map(|source| source.document_id.to_string()))
        .bind(fact.source.as_ref().map(|source| i64::from(source.start)))
        .bind(fact.source.as_ref().map(|source| i64::from(source.end)))
        .bind(json_string(&fact.context))
        .bind(f64::from(fact.confidence))
        .bind(fact.anchor.as_ref().map(json_string))
        .bind(json_string(&fact.provenance))
        .bind(&fact.settled_by)
        .bind(fact.settled_at.map(|at| at.to_rfc3339()))
        .bind(fact.created_at.to_rfc3339())
        .bind(fact.updated_at.to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(storage_error)?;
        Ok(())
    }

    async fn update_fact(&self, fact: &Fact) -> Result<()> {
        let result = sqlx::query(
            "UPDATE facts SET entity_key = ?, predicate = ?, value = ?, modality = ?, status = ?, valid_from = ?, valid_to = ?, context_json = ?, confidence = ?, anchor_json = ?, provenance_json = ?, settled_by = ?, settled_at = ?, updated_at = ?
             WHERE id = ?",
        )
        .bind(&fact.entity_key)
        .bind(&fact.predicate)
        .bind(&fact.value)
        .bind(fact_modality(&fact.modality))
        .bind(fact_status(fact.status))
        .bind(&fact.valid_from)
        .bind(&fact.valid_to)
        .bind(json_string(&fact.context))
        .bind(f64::from(fact.confidence))
        .bind(fact.anchor.as_ref().map(json_string))
        .bind(json_string(&fact.provenance))
        .bind(&fact.settled_by)
        .bind(fact.settled_at.map(|at| at.to_rfc3339()))
        .bind(fact.updated_at.to_rfc3339())
        .bind(fact.id.to_string())
        .execute(&self.pool)
        .await
        .map_err(storage_error)?;
        if result.rows_affected() == 0 {
            return Err(ApplicationError::NotFound { entity: "fact" });
        }
        Ok(())
    }

    async fn delete_fact(&self, id: FactId) -> Result<()> {
        sqlx::query("DELETE FROM facts WHERE id = ?")
            .bind(id.to_string())
            .execute(&self.pool)
            .await
            .map_err(storage_error)?;
        Ok(())
    }

    // ───────────────────────── world ─────────────────────────

    async fn list_world_entries(&self, project_id: ProjectId) -> Result<Vec<WorldEntry>> {
        sqlx::query("SELECT id, project_id, kind, title, summary, prompt, status, board_x, board_y, map_x, map_y, hue, created_at, updated_at FROM world_entries WHERE project_id = ? ORDER BY created_at")
            .bind(project_id.to_string())
            .fetch_all(&self.pool)
            .await
            .map_err(storage_error)?
            .into_iter()
            .map(world_entry_from_row)
            .collect()
    }

    async fn find_world_entry(&self, id: WorldEntryId) -> Result<Option<WorldEntry>> {
        sqlx::query("SELECT id, project_id, kind, title, summary, prompt, status, board_x, board_y, map_x, map_y, hue, created_at, updated_at FROM world_entries WHERE id = ?")
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(storage_error)?
            .map(world_entry_from_row)
            .transpose()
    }

    async fn insert_world_entry(&self, entry: &WorldEntry) -> Result<()> {
        sqlx::query("INSERT INTO world_entries (id, project_id, kind, title, summary, prompt, status, board_x, board_y, map_x, map_y, hue, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")
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
        let result = sqlx::query("UPDATE world_entries SET kind = ?, title = ?, summary = ?, prompt = ?, status = ?, board_x = ?, board_y = ?, map_x = ?, map_y = ?, hue = ?, updated_at = ? WHERE id = ?")
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

    // ───────────────────────── entities ─────────────────────────

    async fn list_entities(&self, project_id: ProjectId) -> Result<Vec<CanonEntity>> {
        sqlx::query(&format!(
            "SELECT {ENTITY_COLUMNS} FROM entities WHERE project_id = ? ORDER BY kind, name"
        ))
        .bind(project_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(storage_error)?
        .into_iter()
        .map(entity_from_row)
        .collect()
    }

    async fn find_entity(&self, id: EntityId) -> Result<Option<CanonEntity>> {
        sqlx::query(&format!(
            "SELECT {ENTITY_COLUMNS} FROM entities WHERE id = ?"
        ))
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(storage_error)?
        .map(entity_from_row)
        .transpose()
    }

    async fn insert_entity(&self, entity: &CanonEntity) -> Result<()> {
        sqlx::query(&format!(
            "INSERT INTO entities ({ENTITY_COLUMNS}) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
        ))
        .bind(entity.id.to_string())
        .bind(entity.project_id.to_string())
        .bind(entity_kind(&entity.kind))
        .bind(&entity.name)
        .bind(json_string(&entity.aliases))
        .bind(&entity.notes)
        .bind(entity.introduced_at)
        .bind(entity.created_at.to_rfc3339())
        .bind(entity.updated_at.to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(storage_error)?;
        Ok(())
    }

    async fn update_entity(&self, entity: &CanonEntity) -> Result<()> {
        let result = sqlx::query("UPDATE entities SET kind = ?, name = ?, aliases = ?, notes = ?, introduced_at = ?, updated_at = ? WHERE id = ?")
            .bind(entity_kind(&entity.kind))
            .bind(&entity.name)
            .bind(json_string(&entity.aliases))
            .bind(&entity.notes)
            .bind(entity.introduced_at)
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

    async fn delete_entity(&self, id: EntityId) -> Result<()> {
        sqlx::query("DELETE FROM entities WHERE id = ?")
            .bind(id.to_string())
            .execute(&self.pool)
            .await
            .map_err(storage_error)?;
        Ok(())
    }

    // ───────────────────────── events ─────────────────────────

    async fn list_events(&self, project_id: ProjectId) -> Result<Vec<TimelineEvent>> {
        sqlx::query(&format!("SELECT {EVENT_COLUMNS} FROM timeline_events WHERE project_id = ? ORDER BY reading_order, created_at"))
            .bind(project_id.to_string())
            .fetch_all(&self.pool)
            .await
            .map_err(storage_error)?
            .into_iter()
            .map(event_from_row)
            .collect()
    }

    async fn find_event(&self, id: TimelineEventId) -> Result<Option<TimelineEvent>> {
        sqlx::query(&format!(
            "SELECT {EVENT_COLUMNS} FROM timeline_events WHERE id = ?"
        ))
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(storage_error)?
        .map(event_from_row)
        .transpose()
    }

    async fn insert_event(&self, event: &TimelineEvent) -> Result<()> {
        sqlx::query(&format!(
            "INSERT INTO timeline_events ({EVENT_COLUMNS}) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        ))
        .bind(event.id.to_string())
        .bind(event.project_id.to_string())
        .bind(event.entity_id.map(|id| id.to_string()))
        .bind(event.document_id.map(|id| id.to_string()))
        .bind(&event.title)
        .bind(&event.story_time)
        .bind(event.reading_order)
        .bind(&event.duration)
        .bind(&event.notes)
        .bind(json_string(&event.kind))
        .bind(json_string(&event.participants))
        .bind(json_string(&event.context))
        .bind(fact_status(event.status))
        .bind(f64::from(event.confidence))
        .bind(event.anchor.as_ref().map(json_string))
        .bind(event.tick)
        .bind(event.tick_lo)
        .bind(event.tick_hi)
        .bind(json_string(&event.provenance))
        .bind(event.created_at.to_rfc3339())
        .bind(event.updated_at.to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(storage_error)?;
        Ok(())
    }

    async fn update_event(&self, event: &TimelineEvent) -> Result<()> {
        let result = sqlx::query(
            "UPDATE timeline_events SET entity_id = ?, document_id = ?, title = ?, story_time = ?, reading_order = ?, duration = ?, notes = ?, kind_json = ?, participants_json = ?, context_json = ?, status = ?, confidence = ?, anchor_json = ?, tick = ?, tick_lo = ?, tick_hi = ?, provenance_json = ?, updated_at = ?
             WHERE id = ?",
        )
        .bind(event.entity_id.map(|id| id.to_string()))
        .bind(event.document_id.map(|id| id.to_string()))
        .bind(&event.title)
        .bind(&event.story_time)
        .bind(event.reading_order)
        .bind(&event.duration)
        .bind(&event.notes)
        .bind(json_string(&event.kind))
        .bind(json_string(&event.participants))
        .bind(json_string(&event.context))
        .bind(fact_status(event.status))
        .bind(f64::from(event.confidence))
        .bind(event.anchor.as_ref().map(json_string))
        .bind(event.tick)
        .bind(event.tick_lo)
        .bind(event.tick_hi)
        .bind(json_string(&event.provenance))
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

    async fn delete_event(&self, id: TimelineEventId) -> Result<()> {
        sqlx::query("DELETE FROM timeline_events WHERE id = ?")
            .bind(id.to_string())
            .execute(&self.pool)
            .await
            .map_err(storage_error)?;
        Ok(())
    }

    // ───────────────────────── comments ─────────────────────────

    async fn list_comments(&self, document_id: DocumentId) -> Result<Vec<Comment>> {
        sqlx::query("SELECT id, document_id, block_id, from_offset, to_offset, body, resolved, created_at, updated_at FROM comments WHERE document_id = ? ORDER BY created_at")
            .bind(document_id.to_string())
            .fetch_all(&self.pool)
            .await
            .map_err(storage_error)?
            .into_iter()
            .map(comment_from_row)
            .collect()
    }

    async fn find_comment(&self, id: CommentId) -> Result<Option<Comment>> {
        sqlx::query("SELECT id, document_id, block_id, from_offset, to_offset, body, resolved, created_at, updated_at FROM comments WHERE id = ?")
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(storage_error)?
            .map(comment_from_row)
            .transpose()
    }

    async fn insert_comment(&self, comment: &Comment) -> Result<()> {
        sqlx::query("INSERT INTO comments (id, document_id, block_id, from_offset, to_offset, body, resolved, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)")
            .bind(comment.id.to_string())
            .bind(comment.document_id.to_string())
            .bind(&comment.block_id)
            .bind(comment.from_offset)
            .bind(comment.to_offset)
            .bind(&comment.body)
            .bind(i64::from(comment.resolved))
            .bind(comment.created_at.to_rfc3339())
            .bind(comment.updated_at.to_rfc3339())
            .execute(&self.pool)
            .await
            .map_err(storage_error)?;
        Ok(())
    }

    async fn update_comment(&self, comment: &Comment) -> Result<()> {
        let result = sqlx::query("UPDATE comments SET body = ?, resolved = ?, from_offset = ?, to_offset = ?, updated_at = ? WHERE id = ?")
            .bind(&comment.body)
            .bind(i64::from(comment.resolved))
            .bind(comment.from_offset)
            .bind(comment.to_offset)
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

    // ───────────────────────── legacy ops ─────────────────────────

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
        sqlx::query("INSERT INTO document_ops (id, document_id, seq, kind, op_json, actor, created_at) VALUES (?, ?, ?, ?, ?, ?, ?)")
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
        sqlx::query("SELECT id, document_id, seq, kind, op_json, actor, created_at FROM document_ops WHERE document_id = ? ORDER BY seq")
            .bind(document_id.to_string())
            .fetch_all(&self.pool)
            .await
            .map_err(storage_error)?
            .into_iter()
            .map(op_from_row)
            .collect()
    }

    // ───────────────────────── snapshots ─────────────────────────

    async fn insert_snapshot(&self, snapshot: &DocumentSnapshot) -> Result<()> {
        sqlx::query(&format!(
            "INSERT INTO document_snapshots ({SNAPSHOT_COLUMNS}) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
        ))
        .bind(snapshot.id.to_string())
        .bind(snapshot.document_id.to_string())
        .bind(snapshot.revision)
        .bind(&snapshot.label)
        .bind(snapshot.content_json.to_string())
        .bind(snapshot.seq)
        .bind(&snapshot.chain_hash)
        .bind(&snapshot.merkle_root)
        .bind(snapshot.created_at.to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(storage_error)?;
        Ok(())
    }

    async fn list_snapshots(&self, document_id: DocumentId) -> Result<Vec<DocumentSnapshot>> {
        sqlx::query(&format!("SELECT {SNAPSHOT_COLUMNS} FROM document_snapshots WHERE document_id = ? ORDER BY created_at DESC"))
            .bind(document_id.to_string())
            .fetch_all(&self.pool)
            .await
            .map_err(storage_error)?
            .into_iter()
            .map(snapshot_from_row)
            .collect()
    }

    async fn find_snapshot(&self, id: SnapshotId) -> Result<Option<DocumentSnapshot>> {
        sqlx::query(&format!(
            "SELECT {SNAPSHOT_COLUMNS} FROM document_snapshots WHERE id = ?"
        ))
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(storage_error)?
        .map(snapshot_from_row)
        .transpose()
    }

    // ───────────────────────── suppressions ─────────────────────────

    async fn list_suppressions(&self, project_id: ProjectId) -> Result<Vec<Suppression>> {
        sqlx::query(&format!("SELECT {SUPPRESSION_COLUMNS} FROM suppressions WHERE project_id = ? ORDER BY created_at DESC"))
            .bind(project_id.to_string())
            .fetch_all(&self.pool)
            .await
            .map_err(storage_error)?
            .into_iter()
            .map(suppression_from_row)
            .collect()
    }

    async fn find_suppression(&self, id: SuppressionId) -> Result<Option<Suppression>> {
        sqlx::query(&format!(
            "SELECT {SUPPRESSION_COLUMNS} FROM suppressions WHERE id = ?"
        ))
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(storage_error)?
        .map(suppression_from_row)
        .transpose()
    }

    async fn insert_suppression(&self, suppression: &Suppression) -> Result<()> {
        sqlx::query(&format!("INSERT INTO suppressions ({SUPPRESSION_COLUMNS}) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"))
            .bind(suppression.id.to_string())
            .bind(suppression.project_id.to_string())
            .bind(&suppression.rule_code)
            .bind(suppression.document_id.map(|id| id.to_string()))
            .bind(&suppression.entity_key)
            .bind(&suppression.reason)
            .bind(suppression_scope(&suppression.scope))
            .bind(&suppression.diagnostic_id)
            .bind(&suppression.evidence_fingerprint)
            .bind(i64::from(suppression.stale))
            .bind(suppression.created_at.to_rfc3339())
            .execute(&self.pool)
            .await
            .map_err(storage_error)?;
        Ok(())
    }

    async fn update_suppression(&self, suppression: &Suppression) -> Result<()> {
        sqlx::query("UPDATE suppressions SET reason = ?, scope = ?, diagnostic_id = ?, evidence_fingerprint = ?, stale = ? WHERE id = ?")
            .bind(&suppression.reason)
            .bind(suppression_scope(&suppression.scope))
            .bind(&suppression.diagnostic_id)
            .bind(&suppression.evidence_fingerprint)
            .bind(i64::from(suppression.stale))
            .bind(suppression.id.to_string())
            .execute(&self.pool)
            .await
            .map_err(storage_error)?;
        Ok(())
    }

    async fn delete_suppression(&self, id: SuppressionId) -> Result<()> {
        sqlx::query("DELETE FROM suppressions WHERE id = ?")
            .bind(id.to_string())
            .execute(&self.pool)
            .await
            .map_err(storage_error)?;
        Ok(())
    }

    // ───────────────────────── packs ─────────────────────────

    async fn list_packs(&self) -> Result<Vec<MethodPack>> {
        sqlx::query("SELECT id, pack_key, name, version, summary, enabled, manifest_json, created_at FROM method_packs ORDER BY name")
            .fetch_all(&self.pool)
            .await
            .map_err(storage_error)?
            .into_iter()
            .map(pack_from_row)
            .collect()
    }

    async fn update_pack(&self, pack: &MethodPack) -> Result<()> {
        sqlx::query("UPDATE method_packs SET enabled = ? WHERE id = ?")
            .bind(i64::from(pack.enabled))
            .bind(pack.id.to_string())
            .execute(&self.pool)
            .await
            .map_err(storage_error)?;
        Ok(())
    }

    async fn find_pack(&self, id: MethodPackId) -> Result<Option<MethodPack>> {
        sqlx::query("SELECT id, pack_key, name, version, summary, enabled, manifest_json, created_at FROM method_packs WHERE id = ?")
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(storage_error)?
            .map(pack_from_row)
            .transpose()
    }

    // ───────────────────────── tracked changes ─────────────────────────

    async fn list_tracked_changes(&self, document_id: DocumentId) -> Result<Vec<TrackedChange>> {
        sqlx::query("SELECT id, document_id, kind, block_id, text, actor, accepted, created_at FROM tracked_changes WHERE document_id = ? ORDER BY created_at")
            .bind(document_id.to_string())
            .fetch_all(&self.pool)
            .await
            .map_err(storage_error)?
            .into_iter()
            .map(tracked_from_row)
            .collect()
    }

    async fn insert_tracked_change(&self, change: &TrackedChange) -> Result<()> {
        sqlx::query("INSERT INTO tracked_changes (id, document_id, kind, block_id, text, actor, accepted, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)")
            .bind(change.id.to_string())
            .bind(change.document_id.to_string())
            .bind(tracked_kind(&change.kind))
            .bind(&change.block_id)
            .bind(&change.text)
            .bind(&change.actor)
            .bind(change.accepted.map(i64::from))
            .bind(change.created_at.to_rfc3339())
            .execute(&self.pool)
            .await
            .map_err(storage_error)?;
        Ok(())
    }

    async fn update_tracked_change(&self, change: &TrackedChange) -> Result<()> {
        sqlx::query("UPDATE tracked_changes SET accepted = ? WHERE id = ?")
            .bind(change.accepted.map(i64::from))
            .bind(change.id.to_string())
            .execute(&self.pool)
            .await
            .map_err(storage_error)?;
        Ok(())
    }

    async fn find_tracked_change(&self, id: TrackedChangeId) -> Result<Option<TrackedChange>> {
        sqlx::query("SELECT id, document_id, kind, block_id, text, actor, accepted, created_at FROM tracked_changes WHERE id = ?")
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(storage_error)?
            .map(tracked_from_row)
            .transpose()
    }

    // ───────────────────────── drafts ─────────────────────────

    async fn list_drafts(&self, document_id: DocumentId) -> Result<Vec<Draft>> {
        sqlx::query("SELECT id, document_id, name, base_seq, base_state_json, content_json, created_at, updated_at, merged_at FROM drafts WHERE document_id = ? ORDER BY created_at DESC")
            .bind(document_id.to_string())
            .fetch_all(&self.pool)
            .await
            .map_err(storage_error)?
            .into_iter()
            .map(draft_from_row)
            .collect()
    }

    async fn find_draft(&self, id: DraftId) -> Result<Option<Draft>> {
        sqlx::query("SELECT id, document_id, name, base_seq, base_state_json, content_json, created_at, updated_at, merged_at FROM drafts WHERE id = ?")
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(storage_error)?
            .map(draft_from_row)
            .transpose()
    }

    async fn insert_draft(&self, draft: &Draft) -> Result<()> {
        sqlx::query("INSERT INTO drafts (id, document_id, name, base_seq, base_state_json, content_json, created_at, updated_at, merged_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)")
            .bind(draft.id.to_string())
            .bind(draft.document_id.to_string())
            .bind(&draft.name)
            .bind(draft.base_seq)
            .bind(draft.base_state_json.to_string())
            .bind(draft.content_json.to_string())
            .bind(draft.created_at.to_rfc3339())
            .bind(draft.updated_at.to_rfc3339())
            .bind(draft.merged_at.map(|at| at.to_rfc3339()))
            .execute(&self.pool)
            .await
            .map_err(storage_error)?;
        Ok(())
    }

    async fn update_draft(&self, draft: &Draft) -> Result<()> {
        sqlx::query("UPDATE drafts SET name = ?, content_json = ?, updated_at = ?, merged_at = ? WHERE id = ?")
            .bind(&draft.name)
            .bind(draft.content_json.to_string())
            .bind(draft.updated_at.to_rfc3339())
            .bind(draft.merged_at.map(|at| at.to_rfc3339()))
            .bind(draft.id.to_string())
            .execute(&self.pool)
            .await
            .map_err(storage_error)?;
        Ok(())
    }

    async fn delete_draft(&self, id: DraftId) -> Result<()> {
        sqlx::query("DELETE FROM drafts WHERE id = ?")
            .bind(id.to_string())
            .execute(&self.pool)
            .await
            .map_err(storage_error)?;
        Ok(())
    }

    // ───────────────────────── settings ─────────────────────────

    async fn get_settings(&self, project_id: ProjectId) -> Result<Option<ProjectSettings>> {
        let row = sqlx::query("SELECT settings_json FROM project_settings WHERE project_id = ?")
            .bind(project_id.to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(storage_error)?;
        row.map(|row| {
            let text: String = row.try_get("settings_json").map_err(storage_error)?;
            serde_json::from_str(&text).map_err(storage_error)
        })
        .transpose()
    }

    async fn put_settings(&self, settings: &ProjectSettings) -> Result<()> {
        sqlx::query("INSERT INTO project_settings (project_id, settings_json, updated_at) VALUES (?, ?, ?) ON CONFLICT(project_id) DO UPDATE SET settings_json = excluded.settings_json, updated_at = excluded.updated_at")
            .bind(settings.project_id.to_string())
            .bind(json_string(settings))
            .bind(settings.updated_at.to_rfc3339())
            .execute(&self.pool)
            .await
            .map_err(storage_error)?;
        Ok(())
    }

    // ───────────────────────── invariants, routes, baselines ─────────────────────────

    async fn list_invariants(&self, project_id: ProjectId) -> Result<Vec<StoredInvariant>> {
        sqlx::query("SELECT id, project_id, invariant_json, created_at FROM invariants WHERE project_id = ? ORDER BY created_at")
            .bind(project_id.to_string())
            .fetch_all(&self.pool)
            .await
            .map_err(storage_error)?
            .into_iter()
            .map(|row| {
                Ok(StoredInvariant {
                    id: InvariantId(parse_uuid(row.try_get("id").map_err(storage_error)?)?),
                    project_id: ProjectId(parse_uuid(row.try_get("project_id").map_err(storage_error)?)?),
                    invariant: parse_json(row.try_get("invariant_json").map_err(storage_error)?)?,
                    created_at: parse_time(row.try_get("created_at").map_err(storage_error)?)?,
                })
            })
            .collect()
    }

    async fn insert_invariant(&self, invariant: &StoredInvariant) -> Result<()> {
        sqlx::query("INSERT INTO invariants (id, project_id, invariant_json, created_at) VALUES (?, ?, ?, ?)")
            .bind(invariant.id.to_string())
            .bind(invariant.project_id.to_string())
            .bind(json_string(&invariant.invariant))
            .bind(invariant.created_at.to_rfc3339())
            .execute(&self.pool)
            .await
            .map_err(storage_error)?;
        Ok(())
    }

    async fn update_invariant(&self, invariant: &StoredInvariant) -> Result<()> {
        sqlx::query("UPDATE invariants SET invariant_json = ? WHERE id = ?")
            .bind(json_string(&invariant.invariant))
            .bind(invariant.id.to_string())
            .execute(&self.pool)
            .await
            .map_err(storage_error)?;
        Ok(())
    }

    async fn delete_invariant(&self, id: InvariantId) -> Result<()> {
        sqlx::query("DELETE FROM invariants WHERE id = ?")
            .bind(id.to_string())
            .execute(&self.pool)
            .await
            .map_err(storage_error)?;
        Ok(())
    }

    async fn list_routes(&self, project_id: ProjectId) -> Result<Vec<StoredRoute>> {
        sqlx::query("SELECT id, project_id, route_json, created_at FROM routes WHERE project_id = ? ORDER BY created_at")
            .bind(project_id.to_string())
            .fetch_all(&self.pool)
            .await
            .map_err(storage_error)?
            .into_iter()
            .map(|row| {
                Ok(StoredRoute {
                    id: RouteId(parse_uuid(row.try_get("id").map_err(storage_error)?)?),
                    project_id: ProjectId(parse_uuid(row.try_get("project_id").map_err(storage_error)?)?),
                    route: parse_json(row.try_get("route_json").map_err(storage_error)?)?,
                    created_at: parse_time(row.try_get("created_at").map_err(storage_error)?)?,
                })
            })
            .collect()
    }

    async fn insert_route(&self, route: &StoredRoute) -> Result<()> {
        sqlx::query(
            "INSERT INTO routes (id, project_id, route_json, created_at) VALUES (?, ?, ?, ?)",
        )
        .bind(route.id.to_string())
        .bind(route.project_id.to_string())
        .bind(json_string(&route.route))
        .bind(route.created_at.to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(storage_error)?;
        Ok(())
    }

    async fn delete_route(&self, id: RouteId) -> Result<()> {
        sqlx::query("DELETE FROM routes WHERE id = ?")
            .bind(id.to_string())
            .execute(&self.pool)
            .await
            .map_err(storage_error)?;
        Ok(())
    }

    async fn list_baselines(&self, project_id: ProjectId) -> Result<Vec<Baseline>> {
        sqlx::query("SELECT id, project_id, name, revision, diagnostic_ids_json, active, created_at FROM baselines WHERE project_id = ? ORDER BY created_at DESC")
            .bind(project_id.to_string())
            .fetch_all(&self.pool)
            .await
            .map_err(storage_error)?
            .into_iter()
            .map(|row| {
                Ok(Baseline {
                    id: BaselineId(parse_uuid(row.try_get("id").map_err(storage_error)?)?),
                    project_id: ProjectId(parse_uuid(row.try_get("project_id").map_err(storage_error)?)?),
                    name: row.try_get("name").map_err(storage_error)?,
                    revision: row.try_get("revision").map_err(storage_error)?,
                    diagnostic_ids: parse_json(row.try_get("diagnostic_ids_json").map_err(storage_error)?)?,
                    active: row.try_get::<i64, _>("active").map_err(storage_error)? != 0,
                    created_at: parse_time(row.try_get("created_at").map_err(storage_error)?)?,
                })
            })
            .collect()
    }

    async fn insert_baseline(&self, baseline: &Baseline) -> Result<()> {
        sqlx::query("INSERT INTO baselines (id, project_id, name, revision, diagnostic_ids_json, active, created_at) VALUES (?, ?, ?, ?, ?, ?, ?)")
            .bind(baseline.id.to_string())
            .bind(baseline.project_id.to_string())
            .bind(&baseline.name)
            .bind(&baseline.revision)
            .bind(json_string(&baseline.diagnostic_ids))
            .bind(i64::from(baseline.active))
            .bind(baseline.created_at.to_rfc3339())
            .execute(&self.pool)
            .await
            .map_err(storage_error)?;
        Ok(())
    }

    async fn update_baseline(&self, baseline: &Baseline) -> Result<()> {
        sqlx::query("UPDATE baselines SET name = ?, active = ? WHERE id = ?")
            .bind(&baseline.name)
            .bind(i64::from(baseline.active))
            .bind(baseline.id.to_string())
            .execute(&self.pool)
            .await
            .map_err(storage_error)?;
        Ok(())
    }

    async fn delete_baseline(&self, id: BaselineId) -> Result<()> {
        sqlx::query("DELETE FROM baselines WHERE id = ?")
            .bind(id.to_string())
            .execute(&self.pool)
            .await
            .map_err(storage_error)?;
        Ok(())
    }

    // ───────────────────────── auth ─────────────────────────

    async fn list_tokens(&self) -> Result<Vec<AuthToken>> {
        sqlx::query(&format!(
            "SELECT {TOKEN_COLUMNS} FROM auth_tokens ORDER BY created_at DESC"
        ))
        .fetch_all(&self.pool)
        .await
        .map_err(storage_error)?
        .into_iter()
        .map(token_from_row)
        .collect()
    }

    async fn find_token_by_hash(&self, hash: &str) -> Result<Option<AuthToken>> {
        sqlx::query(&format!(
            "SELECT {TOKEN_COLUMNS} FROM auth_tokens WHERE hash = ?"
        ))
        .bind(hash)
        .fetch_optional(&self.pool)
        .await
        .map_err(storage_error)?
        .map(token_from_row)
        .transpose()
    }

    async fn insert_token(&self, token: &AuthToken) -> Result<()> {
        sqlx::query(&format!(
            "INSERT INTO auth_tokens ({TOKEN_COLUMNS}) VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
        ))
        .bind(token.id.to_string())
        .bind(&token.hash)
        .bind(token_kind(token.kind))
        .bind(&token.label)
        .bind(token.created_at.to_rfc3339())
        .bind(token.expires_at.map(|at| at.to_rfc3339()))
        .bind(token.revoked_at.map(|at| at.to_rfc3339()))
        .bind(token.last_used_at.map(|at| at.to_rfc3339()))
        .execute(&self.pool)
        .await
        .map_err(storage_error)?;
        Ok(())
    }

    async fn update_token(&self, token: &AuthToken) -> Result<()> {
        sqlx::query("UPDATE auth_tokens SET label = ?, expires_at = ?, revoked_at = ?, last_used_at = ? WHERE id = ?")
            .bind(&token.label)
            .bind(token.expires_at.map(|at| at.to_rfc3339()))
            .bind(token.revoked_at.map(|at| at.to_rfc3339()))
            .bind(token.last_used_at.map(|at| at.to_rfc3339()))
            .bind(token.id.to_string())
            .execute(&self.pool)
            .await
            .map_err(storage_error)?;
        Ok(())
    }

    async fn delete_token(&self, id: AuthTokenId) -> Result<()> {
        sqlx::query("DELETE FROM auth_tokens WHERE id = ?")
            .bind(id.to_string())
            .execute(&self.pool)
            .await
            .map_err(storage_error)?;
        Ok(())
    }

    async fn list_pairing_codes(&self) -> Result<Vec<PairingCode>> {
        sqlx::query(&format!(
            "SELECT {PAIRING_COLUMNS} FROM pairing_codes ORDER BY created_at DESC"
        ))
        .fetch_all(&self.pool)
        .await
        .map_err(storage_error)?
        .into_iter()
        .map(pairing_from_row)
        .collect()
    }

    async fn insert_pairing_code(&self, code: &PairingCode) -> Result<()> {
        sqlx::query(&format!(
            "INSERT INTO pairing_codes ({PAIRING_COLUMNS}) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
        ))
        .bind(code.id.to_string())
        .bind(&code.code_hash)
        .bind(&code.salt)
        .bind(&code.label)
        .bind(code.grants.as_str())
        .bind(code.expires_at.to_rfc3339())
        .bind(code.consumed_at.map(|at| at.to_rfc3339()))
        .bind(code.attempts)
        .bind(code.created_at.to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(storage_error)?;
        Ok(())
    }

    async fn update_pairing_code(&self, code: &PairingCode) -> Result<()> {
        sqlx::query("UPDATE pairing_codes SET consumed_at = ?, attempts = ? WHERE id = ?")
            .bind(code.consumed_at.map(|at| at.to_rfc3339()))
            .bind(code.attempts)
            .bind(code.id.to_string())
            .execute(&self.pool)
            .await
            .map_err(storage_error)?;
        Ok(())
    }

    async fn record_pairing_failure(&self, id: PairingCodeId) -> Result<()> {
        sqlx::query("UPDATE pairing_codes SET attempts = attempts + 1 WHERE id = ?")
            .bind(id.to_string())
            .execute(&self.pool)
            .await
            .map_err(storage_error)?;
        Ok(())
    }

    async fn consume_pairing_code(&self, id: PairingCodeId, at: DateTime<Utc>) -> Result<bool> {
        let result = sqlx::query(
            "UPDATE pairing_codes SET consumed_at = ? WHERE id = ? AND consumed_at IS NULL AND attempts < 5",
        )
        .bind(at.to_rfc3339())
        .bind(id.to_string())
        .execute(&self.pool)
        .await
        .map_err(storage_error)?;
        Ok(result.rows_affected() == 1)
    }

    async fn delete_pairing_code(&self, id: PairingCodeId) -> Result<()> {
        sqlx::query("DELETE FROM pairing_codes WHERE id = ?")
            .bind(id.to_string())
            .execute(&self.pool)
            .await
            .map_err(storage_error)?;
        Ok(())
    }

    async fn owner_credential(&self) -> Result<Option<OwnerCredential>> {
        let row = sqlx::query("SELECT phc, created_at FROM owner_credential WHERE id = 1")
            .fetch_optional(&self.pool)
            .await
            .map_err(storage_error)?;
        row.map(|row| {
            Ok(OwnerCredential {
                phc: row.try_get("phc").map_err(storage_error)?,
                created_at: parse_time(row.try_get("created_at").map_err(storage_error)?)?,
            })
        })
        .transpose()
    }

    async fn create_owner_credential(&self, credential: &OwnerCredential) -> Result<bool> {
        let result = sqlx::query(
            "INSERT OR IGNORE INTO owner_credential (id, phc, created_at) VALUES (1, ?, ?)",
        )
        .bind(&credential.phc)
        .bind(credential.created_at.to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(storage_error)?;
        Ok(result.rows_affected() == 1)
    }

    async fn set_owner_credential(&self, credential: &OwnerCredential) -> Result<()> {
        sqlx::query("INSERT INTO owner_credential (id, phc, created_at) VALUES (1, ?, ?) ON CONFLICT(id) DO UPDATE SET phc = excluded.phc, created_at = excluded.created_at")
            .bind(&credential.phc)
            .bind(credential.created_at.to_rfc3339())
            .execute(&self.pool)
            .await
            .map_err(storage_error)?;
        Ok(())
    }
}

// ───────────────────────── helpers ─────────────────────────

fn storage_error(error: impl std::fmt::Display) -> ApplicationError {
    ApplicationError::Storage(error.to_string())
}

fn json_string<T: serde::Serialize>(value: &T) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "null".into())
}

fn parse_json<T: serde::de::DeserializeOwned>(text: String) -> Result<T> {
    serde_json::from_str(&text).map_err(storage_error)
}

fn parse_json_or<T: serde::de::DeserializeOwned + Default>(text: Option<String>) -> T {
    text.and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

fn parse_uuid(value: String) -> Result<Uuid> {
    Uuid::parse_str(&value).map_err(storage_error)
}

fn parse_time(value: String) -> Result<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(&value)
        .map(|time| time.with_timezone(&Utc))
        .map_err(storage_error)
}

fn parse_time_opt(value: Option<String>) -> Result<Option<DateTime<Utc>>> {
    value.map(parse_time).transpose()
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
    Ok(match value.as_str() {
        "novel" => ProjectKind::Novel,
        "screenplay" => ProjectKind::Screenplay,
        "non_fiction" => ProjectKind::NonFiction,
        "other" => ProjectKind::Other,
        other => return Err(storage_error(format!("unknown project kind {other}"))),
    })
}

fn project_from_row(row: SqliteRow) -> Result<Project> {
    Ok(Project {
        id: ProjectId(parse_uuid(row.try_get("id").map_err(storage_error)?)?),
        name: row.try_get("name").map_err(storage_error)?,
        kind: parse_project_kind(row.try_get("kind").map_err(storage_error)?)?,
        description: row.try_get("description").map_err(storage_error)?,
        created_at: parse_time(row.try_get("created_at").map_err(storage_error)?)?,
        updated_at: parse_time(row.try_get("updated_at").map_err(storage_error)?)?,
    })
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
    Ok(match value.as_str() {
        "manuscript" => DocumentKind::Manuscript,
        "chapter" => DocumentKind::Chapter,
        "scene" => DocumentKind::Scene,
        "wiki" => DocumentKind::Wiki,
        "note" => DocumentKind::Note,
        "source" => DocumentKind::Source,
        other => return Err(storage_error(format!("unknown document kind {other}"))),
    })
}

fn document_from_row(row: SqliteRow) -> Result<Document> {
    let content: String = row.try_get("content_json").map_err(storage_error)?;
    let scene: Option<String> = row.try_get("scene_json").map_err(storage_error)?;
    let parent: Option<String> = row.try_get("parent_id").map_err(storage_error)?;
    Ok(Document {
        id: DocumentId(parse_uuid(row.try_get("id").map_err(storage_error)?)?),
        project_id: ProjectId(parse_uuid(
            row.try_get("project_id").map_err(storage_error)?,
        )?),
        parent_id: parent.map(parse_uuid).transpose()?.map(DocumentId),
        kind: parse_document_kind(row.try_get("kind").map_err(storage_error)?)?,
        title: row.try_get("title").map_err(storage_error)?,
        content_json: serde_json::from_str(&content).unwrap_or(serde_json::Value::Null),
        position: row.try_get("position").map_err(storage_error)?,
        revision: row.try_get("revision").map_err(storage_error)?,
        exclude_from_compile: row
            .try_get::<i64, _>("exclude_from_compile")
            .map_err(storage_error)?
            != 0,
        scene: parse_json_or::<ScenePlacement>(scene),
        head_seq: row.try_get("head_seq").map_err(storage_error)?,
        chain_hash: row.try_get("chain_hash").map_err(storage_error)?,
        created_at: parse_time(row.try_get("created_at").map_err(storage_error)?)?,
        updated_at: parse_time(row.try_get("updated_at").map_err(storage_error)?)?,
    })
}

fn event_record_from_row(row: SqliteRow) -> Result<EventRecord> {
    let payload: String = row.try_get("payload_json").map_err(storage_error)?;
    let payload: EventPayload = serde_json::from_str(&payload).map_err(storage_error)?;
    Ok(EventRecord {
        id: row.try_get("event_id").map_err(storage_error)?,
        seq: row.try_get("seq").map_err(storage_error)?,
        actor: row.try_get("actor").map_err(storage_error)?,
        at: row.try_get("created_at").map_err(storage_error)?,
        payload,
        content_hash: row.try_get("content_hash").map_err(storage_error)?,
        prev_chain_hash: row.try_get("prev_chain_hash").map_err(storage_error)?,
        chain_hash: row.try_get("chain_hash").map_err(storage_error)?,
    })
}

fn block_snapshot_from_row(row: SqliteRow) -> Result<BlockSnapshot> {
    let state: String = row.try_get("state_json").map_err(storage_error)?;
    Ok(BlockSnapshot {
        id: SnapshotId(parse_uuid(row.try_get("id").map_err(storage_error)?)?),
        document_id: DocumentId(parse_uuid(
            row.try_get("document_id").map_err(storage_error)?,
        )?),
        seq: row.try_get("seq").map_err(storage_error)?,
        chain_hash: row.try_get("chain_hash").map_err(storage_error)?,
        merkle_root: row.try_get("merkle_root").map_err(storage_error)?,
        state_json: serde_json::from_str(&state).map_err(storage_error)?,
        created_at: parse_time(row.try_get("created_at").map_err(storage_error)?)?,
    })
}

fn fact_from_row(row: SqliteRow) -> Result<Fact> {
    let source_document: Option<String> =
        row.try_get("source_document_id").map_err(storage_error)?;
    let source_start: Option<i64> = row.try_get("source_start").map_err(storage_error)?;
    let source_end: Option<i64> = row.try_get("source_end").map_err(storage_error)?;
    let source = match (source_document, source_start, source_end) {
        (Some(document), Some(start), Some(end)) => Some(SourceAnchor {
            document_id: DocumentId(parse_uuid(document)?),
            start: u32::try_from(start).unwrap_or(0),
            end: u32::try_from(end).unwrap_or(0),
        }),
        _ => None,
    };
    let context: Option<String> = row.try_get("context_json").map_err(storage_error)?;
    let anchor: Option<String> = row.try_get("anchor_json").map_err(storage_error)?;
    let provenance: Option<String> = row.try_get("provenance_json").map_err(storage_error)?;
    let confidence: f64 = row.try_get("confidence").map_err(storage_error)?;
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
        context: parse_json_or(context),
        confidence: confidence as f32,
        anchor: anchor.and_then(|text| serde_json::from_str(&text).ok()),
        provenance: parse_json_or(provenance),
        settled_by: row.try_get("settled_by").map_err(storage_error)?,
        settled_at: parse_time_opt(row.try_get("settled_at").map_err(storage_error)?)?,
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
    Ok(match value.as_str() {
        "established" => FactModality::Established,
        "inferred" => FactModality::Inferred,
        "contested" => FactModality::Contested,
        "author_override" => FactModality::AuthorOverride,
        other => return Err(storage_error(format!("unknown fact modality {other}"))),
    })
}

const fn fact_status(status: FactStatus) -> &'static str {
    match status {
        FactStatus::Proposed => "proposed",
        FactStatus::Confirmed => "confirmed",
        FactStatus::Deprecated => "deprecated",
        FactStatus::Rejected => "rejected",
    }
}

fn parse_fact_status(value: String) -> Result<FactStatus> {
    Ok(match value.as_str() {
        "proposed" => FactStatus::Proposed,
        "confirmed" => FactStatus::Confirmed,
        "deprecated" => FactStatus::Deprecated,
        "rejected" => FactStatus::Rejected,
        other => return Err(storage_error(format!("unknown fact status {other}"))),
    })
}

fn world_entry_from_row(row: SqliteRow) -> Result<WorldEntry> {
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
    Ok(match value.as_str() {
        "place" => WorldKind::Place,
        "culture" => WorldKind::Culture,
        "system" => WorldKind::System,
        "creature" => WorldKind::Creature,
        "item" => WorldKind::Item,
        "question" => WorldKind::Question,
        "fragment" => WorldKind::Fragment,
        other => return Err(storage_error(format!("unknown world kind {other}"))),
    })
}

const fn world_status(status: &WorldStatus) -> &'static str {
    match status {
        WorldStatus::Spark => "spark",
        WorldStatus::Sketched => "sketched",
        WorldStatus::Settled => "settled",
    }
}

fn parse_world_status(value: String) -> Result<WorldStatus> {
    Ok(match value.as_str() {
        "spark" => WorldStatus::Spark,
        "sketched" => WorldStatus::Sketched,
        "settled" => WorldStatus::Settled,
        other => return Err(storage_error(format!("unknown world status {other}"))),
    })
}

const fn idea_prompt(prompt: &IdeaPrompt) -> &'static str {
    match prompt {
        IdeaPrompt::Why => "why",
        IdeaPrompt::How => "how",
        IdeaPrompt::WhatResult => "what_result",
    }
}

fn parse_idea_prompt(value: String) -> Result<IdeaPrompt> {
    Ok(match value.as_str() {
        "why" => IdeaPrompt::Why,
        "how" => IdeaPrompt::How,
        "what_result" => IdeaPrompt::WhatResult,
        other => return Err(storage_error(format!("unknown prompt {other}"))),
    })
}

fn entity_from_row(row: SqliteRow) -> Result<CanonEntity> {
    let aliases: String = row.try_get("aliases").map_err(storage_error)?;
    Ok(CanonEntity {
        id: EntityId(parse_uuid(row.try_get("id").map_err(storage_error)?)?),
        project_id: ProjectId(parse_uuid(
            row.try_get("project_id").map_err(storage_error)?,
        )?),
        kind: parse_entity_kind(row.try_get("kind").map_err(storage_error)?)?,
        name: row.try_get("name").map_err(storage_error)?,
        aliases: serde_json::from_str(&aliases).unwrap_or_default(),
        notes: row.try_get("notes").map_err(storage_error)?,
        introduced_at: row.try_get("introduced_at").map_err(storage_error)?,
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
    Ok(match value.as_str() {
        "character" => EntityKind::Character,
        "place" => EntityKind::Place,
        "event" => EntityKind::Event,
        "item" => EntityKind::Item,
        "rule" => EntityKind::Rule,
        "source" => EntityKind::Source,
        "other" => EntityKind::Other,
        other => return Err(storage_error(format!("unknown entity kind {other}"))),
    })
}

fn event_from_row(row: SqliteRow) -> Result<TimelineEvent> {
    let entity: Option<String> = row.try_get("entity_id").map_err(storage_error)?;
    let document: Option<String> = row.try_get("document_id").map_err(storage_error)?;
    let kind: Option<String> = row.try_get("kind_json").map_err(storage_error)?;
    let participants: Option<String> = row.try_get("participants_json").map_err(storage_error)?;
    let context: Option<String> = row.try_get("context_json").map_err(storage_error)?;
    let anchor: Option<String> = row.try_get("anchor_json").map_err(storage_error)?;
    let provenance: Option<String> = row.try_get("provenance_json").map_err(storage_error)?;
    let confidence: f64 = row.try_get("confidence").map_err(storage_error)?;
    Ok(TimelineEvent {
        id: TimelineEventId(parse_uuid(row.try_get("id").map_err(storage_error)?)?),
        project_id: ProjectId(parse_uuid(
            row.try_get("project_id").map_err(storage_error)?,
        )?),
        entity_id: entity.map(parse_uuid).transpose()?.map(EntityId),
        document_id: document.map(parse_uuid).transpose()?.map(DocumentId),
        title: row.try_get("title").map_err(storage_error)?,
        story_time: row.try_get("story_time").map_err(storage_error)?,
        reading_order: row.try_get("reading_order").map_err(storage_error)?,
        duration: row.try_get("duration").map_err(storage_error)?,
        notes: row.try_get("notes").map_err(storage_error)?,
        kind: kind
            .and_then(|text| serde_json::from_str(&text).ok())
            .unwrap_or(asten_ir::EventKind::Acts),
        participants: parse_json_or(participants),
        context: parse_json_or(context),
        status: parse_fact_status(row.try_get("status").map_err(storage_error)?)?,
        confidence: confidence as f32,
        anchor: anchor.and_then(|text| serde_json::from_str(&text).ok()),
        tick: row.try_get("tick").map_err(storage_error)?,
        tick_lo: row.try_get("tick_lo").map_err(storage_error)?,
        tick_hi: row.try_get("tick_hi").map_err(storage_error)?,
        provenance: parse_json_or(provenance),
        created_at: parse_time(row.try_get("created_at").map_err(storage_error)?)?,
        updated_at: parse_time(row.try_get("updated_at").map_err(storage_error)?)?,
    })
}

fn comment_from_row(row: SqliteRow) -> Result<Comment> {
    Ok(Comment {
        id: CommentId(parse_uuid(row.try_get("id").map_err(storage_error)?)?),
        document_id: DocumentId(parse_uuid(
            row.try_get("document_id").map_err(storage_error)?,
        )?),
        block_id: row.try_get("block_id").map_err(storage_error)?,
        from_offset: row.try_get("from_offset").map_err(storage_error)?,
        to_offset: row.try_get("to_offset").map_err(storage_error)?,
        body: row.try_get("body").map_err(storage_error)?,
        resolved: row.try_get::<i64, _>("resolved").map_err(storage_error)? != 0,
        created_at: parse_time(row.try_get("created_at").map_err(storage_error)?)?,
        updated_at: parse_time(row.try_get("updated_at").map_err(storage_error)?)?,
    })
}

fn op_from_row(row: SqliteRow) -> Result<DocumentOp> {
    let op: String = row.try_get("op_json").map_err(storage_error)?;
    Ok(DocumentOp {
        id: DocumentOpId(parse_uuid(row.try_get("id").map_err(storage_error)?)?),
        document_id: DocumentId(parse_uuid(
            row.try_get("document_id").map_err(storage_error)?,
        )?),
        seq: row.try_get("seq").map_err(storage_error)?,
        kind: row.try_get("kind").map_err(storage_error)?,
        op_json: serde_json::from_str(&op).unwrap_or(serde_json::Value::Null),
        actor: row.try_get("actor").map_err(storage_error)?,
        created_at: parse_time(row.try_get("created_at").map_err(storage_error)?)?,
    })
}

fn snapshot_from_row(row: SqliteRow) -> Result<DocumentSnapshot> {
    let content: String = row.try_get("content_json").map_err(storage_error)?;
    Ok(DocumentSnapshot {
        id: SnapshotId(parse_uuid(row.try_get("id").map_err(storage_error)?)?),
        document_id: DocumentId(parse_uuid(
            row.try_get("document_id").map_err(storage_error)?,
        )?),
        revision: row.try_get("revision").map_err(storage_error)?,
        label: row.try_get("label").map_err(storage_error)?,
        content_json: serde_json::from_str(&content).unwrap_or(serde_json::Value::Null),
        seq: row.try_get("seq").map_err(storage_error)?,
        chain_hash: row.try_get("chain_hash").map_err(storage_error)?,
        merkle_root: row.try_get("merkle_root").map_err(storage_error)?,
        created_at: parse_time(row.try_get("created_at").map_err(storage_error)?)?,
    })
}

fn suppression_from_row(row: SqliteRow) -> Result<Suppression> {
    let document: Option<String> = row.try_get("document_id").map_err(storage_error)?;
    let scope: String = row.try_get("scope").map_err(storage_error)?;
    Ok(Suppression {
        id: SuppressionId(parse_uuid(row.try_get("id").map_err(storage_error)?)?),
        project_id: ProjectId(parse_uuid(
            row.try_get("project_id").map_err(storage_error)?,
        )?),
        rule_code: row.try_get("rule_code").map_err(storage_error)?,
        document_id: document.map(parse_uuid).transpose()?.map(DocumentId),
        entity_key: row.try_get("entity_key").map_err(storage_error)?,
        reason: row.try_get("reason").map_err(storage_error)?,
        scope: parse_suppression_scope(&scope),
        diagnostic_id: row.try_get("diagnostic_id").map_err(storage_error)?,
        evidence_fingerprint: row.try_get("evidence_fingerprint").map_err(storage_error)?,
        stale: row.try_get::<i64, _>("stale").map_err(storage_error)? != 0,
        created_at: parse_time(row.try_get("created_at").map_err(storage_error)?)?,
    })
}

const fn suppression_scope(scope: &SuppressionScope) -> &'static str {
    match scope {
        SuppressionScope::Occurrence => "occurrence",
        SuppressionScope::Entity => "entity",
        SuppressionScope::Document => "document",
        SuppressionScope::Project => "project",
    }
}

fn parse_suppression_scope(value: &str) -> SuppressionScope {
    match value {
        "occurrence" => SuppressionScope::Occurrence,
        "entity" => SuppressionScope::Entity,
        "project" => SuppressionScope::Project,
        _ => SuppressionScope::Document,
    }
}

fn pack_from_row(row: SqliteRow) -> Result<MethodPack> {
    let manifest: String = row.try_get("manifest_json").map_err(storage_error)?;
    Ok(MethodPack {
        id: MethodPackId(parse_uuid(row.try_get("id").map_err(storage_error)?)?),
        pack_key: row.try_get("pack_key").map_err(storage_error)?,
        name: row.try_get("name").map_err(storage_error)?,
        version: row.try_get("version").map_err(storage_error)?,
        summary: row.try_get("summary").map_err(storage_error)?,
        enabled: row.try_get::<i64, _>("enabled").map_err(storage_error)? != 0,
        manifest: serde_json::from_str(&manifest).unwrap_or(serde_json::Value::Null),
        created_at: parse_time(row.try_get("created_at").map_err(storage_error)?)?,
    })
}

fn tracked_from_row(row: SqliteRow) -> Result<TrackedChange> {
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
    Ok(match value.as_str() {
        "insert" => TrackedKind::Insert,
        "delete" => TrackedKind::Delete,
        other => return Err(storage_error(format!("unknown tracked kind {other}"))),
    })
}

fn draft_from_row(row: SqliteRow) -> Result<Draft> {
    let base: String = row.try_get("base_state_json").map_err(storage_error)?;
    let content: String = row.try_get("content_json").map_err(storage_error)?;
    Ok(Draft {
        id: DraftId(parse_uuid(row.try_get("id").map_err(storage_error)?)?),
        document_id: DocumentId(parse_uuid(
            row.try_get("document_id").map_err(storage_error)?,
        )?),
        name: row.try_get("name").map_err(storage_error)?,
        base_seq: row.try_get("base_seq").map_err(storage_error)?,
        base_state_json: serde_json::from_str(&base).unwrap_or(serde_json::Value::Null),
        content_json: serde_json::from_str(&content).unwrap_or(serde_json::Value::Null),
        created_at: parse_time(row.try_get("created_at").map_err(storage_error)?)?,
        updated_at: parse_time(row.try_get("updated_at").map_err(storage_error)?)?,
        merged_at: parse_time_opt(row.try_get("merged_at").map_err(storage_error)?)?,
    })
}

const fn token_kind(kind: TokenKind) -> &'static str {
    match kind {
        TokenKind::Desk => "desk",
        TokenKind::Guest => "guest",
        TokenKind::Session => "session",
    }
}

fn token_from_row(row: SqliteRow) -> Result<AuthToken> {
    let kind: String = row.try_get("kind").map_err(storage_error)?;
    Ok(AuthToken {
        id: AuthTokenId(parse_uuid(row.try_get("id").map_err(storage_error)?)?),
        hash: row.try_get("hash").map_err(storage_error)?,
        kind: match kind.as_str() {
            "desk" => TokenKind::Desk,
            "session" => TokenKind::Session,
            _ => TokenKind::Guest,
        },
        label: row.try_get("label").map_err(storage_error)?,
        created_at: parse_time(row.try_get("created_at").map_err(storage_error)?)?,
        expires_at: parse_time_opt(row.try_get("expires_at").map_err(storage_error)?)?,
        revoked_at: parse_time_opt(row.try_get("revoked_at").map_err(storage_error)?)?,
        last_used_at: parse_time_opt(row.try_get("last_used_at").map_err(storage_error)?)?,
    })
}

fn pairing_from_row(row: SqliteRow) -> Result<PairingCode> {
    Ok(PairingCode {
        id: PairingCodeId(parse_uuid(row.try_get("id").map_err(storage_error)?)?),
        code_hash: row.try_get("code_hash").map_err(storage_error)?,
        salt: row.try_get("salt").map_err(storage_error)?,
        label: row.try_get("label").map_err(storage_error)?,
        grants: TokenKind::parse(&row.try_get::<String, _>("grants").map_err(storage_error)?),
        expires_at: parse_time(row.try_get("expires_at").map_err(storage_error)?)?,
        consumed_at: parse_time_opt(row.try_get("consumed_at").map_err(storage_error)?)?,
        attempts: row.try_get("attempts").map_err(storage_error)?,
        created_at: parse_time(row.try_get("created_at").map_err(storage_error)?)?,
    })
}
