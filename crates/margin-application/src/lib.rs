use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use margin_domain::{Document, DocumentId, DocumentKind, Project, ProjectId, ProjectKind};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApplicationError {
    #[error("{entity} was not found")]
    NotFound { entity: &'static str },
    #[error("invalid input: {0}")]
    Validation(String),
    #[error("storage operation failed: {0}")]
    Storage(String),
}

pub type Result<T> = std::result::Result<T, ApplicationError>;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CreateProject {
    pub name: String,
    pub kind: ProjectKind,
    pub description: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CreateDocument {
    pub project_id: ProjectId,
    pub parent_id: Option<DocumentId>,
    pub kind: DocumentKind,
    pub title: String,
    #[serde(default)]
    pub content_json: serde_json::Value,
    #[serde(default)]
    pub position: i64,
}

#[async_trait]
pub trait Repository: Send + Sync {
    async fn list_projects(&self) -> Result<Vec<Project>>;
    async fn find_project(&self, id: ProjectId) -> Result<Option<Project>>;
    async fn insert_project(&self, project: &Project) -> Result<()>;
    async fn list_documents(&self, project_id: ProjectId) -> Result<Vec<Document>>;
    async fn insert_document(&self, document: &Document) -> Result<()>;
}

#[derive(Clone)]
pub struct Application {
    repository: Arc<dyn Repository>,
}

impl Application {
    #[must_use]
    pub fn new(repository: Arc<dyn Repository>) -> Self {
        Self { repository }
    }

    pub async fn list_projects(&self) -> Result<Vec<Project>> {
        self.repository.list_projects().await
    }

    pub async fn get_project(&self, id: ProjectId) -> Result<Project> {
        self.repository
            .find_project(id)
            .await?
            .ok_or(ApplicationError::NotFound { entity: "project" })
    }

    pub async fn create_project(&self, input: CreateProject) -> Result<Project> {
        let name = input.name.trim();
        if name.is_empty() {
            return Err(ApplicationError::Validation(
                "project name cannot be empty".into(),
            ));
        }
        let now = Utc::now();
        let project = Project {
            id: ProjectId::new(),
            name: name.to_owned(),
            kind: input.kind,
            description: input
                .description
                .map(|value| value.trim().to_owned())
                .filter(|value| !value.is_empty()),
            created_at: now,
            updated_at: now,
        };
        self.repository.insert_project(&project).await?;
        Ok(project)
    }

    pub async fn list_documents(&self, project_id: ProjectId) -> Result<Vec<Document>> {
        self.get_project(project_id).await?;
        self.repository.list_documents(project_id).await
    }

    pub async fn create_document(&self, input: CreateDocument) -> Result<Document> {
        self.get_project(input.project_id).await?;
        let title = input.title.trim();
        if title.is_empty() {
            return Err(ApplicationError::Validation(
                "document title cannot be empty".into(),
            ));
        }
        let now = Utc::now();
        let document = Document {
            id: DocumentId::new(),
            project_id: input.project_id,
            parent_id: input.parent_id,
            kind: input.kind,
            title: title.to_owned(),
            content_json: input.content_json,
            position: input.position,
            revision: 1,
            created_at: now,
            updated_at: now,
        };
        self.repository.insert_document(&document).await?;
        Ok(document)
    }
}
