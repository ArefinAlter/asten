use std::path::Path;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use margin_application::{ApplicationError, Repository, Result};
use margin_domain::{
    Document, DocumentId, DocumentKind, Fact, FactId, FactModality, FactStatus, IdeaPrompt, Project,
    ProjectId, ProjectKind, SourceAnchor, WorldEntry, WorldEntryId, WorldKind, WorldStatus,
};
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
                    position, revision, created_at, updated_at
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
                    position, revision, created_at, updated_at
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
              revision, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(document.id.to_string())
        .bind(document.project_id.to_string())
        .bind(document.parent_id.map(|id| id.to_string()))
        .bind(document_kind(&document.kind))
        .bind(&document.title)
        .bind(document.content_json.to_string())
        .bind(document.position)
        .bind(document.revision)
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
             SET title = ?, content_json = ?, revision = ?, updated_at = ?
             WHERE id = ?",
        )
        .bind(&document.title)
        .bind(document.content_json.to_string())
        .bind(document.revision)
        .bind(document.updated_at.to_rfc3339())
        .bind(document.id.to_string())
        .execute(&self.pool)
        .await
        .map_err(storage_error)?;
        if result.rows_affected() == 0 {
            return Err(ApplicationError::NotFound {
                entity: "document",
            });
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
        .bind(fact.source.as_ref().map(|source| source.document_id.to_string()))
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
