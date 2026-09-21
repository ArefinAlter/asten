use std::path::Path;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use margin_application::{ApplicationError, Repository, Result};
use margin_domain::{Document, DocumentId, DocumentKind, Project, ProjectId, ProjectKind};
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
