use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use margin_diagnostics::{AnalysisContext, DiagnosticEngine};
use margin_domain::{
    Diagnostic, Document, DocumentId, DocumentKind, Fact, FactId, FactModality, FactStatus,
    IdeaPrompt, Project, ProjectId, ProjectKind, SourceAnchor, WorldEntry, WorldEntryId, WorldKind,
    WorldPulse, WorldStatus, compute_world_pulse, default_world_hue, entity_key_from_title,
    normalize_key,
};
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
    #[serde(default)]
    pub project_id: ProjectId,
    pub parent_id: Option<DocumentId>,
    pub kind: DocumentKind,
    pub title: String,
    #[serde(default)]
    pub content_json: serde_json::Value,
    #[serde(default)]
    pub position: i64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UpdateDocument {
    pub title: Option<String>,
    pub content_json: Option<serde_json::Value>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CreateFact {
    #[serde(default)]
    pub project_id: ProjectId,
    pub entity_key: String,
    pub predicate: String,
    pub value: String,
    #[serde(default)]
    pub modality: FactModality,
    #[serde(default)]
    pub status: FactStatus,
    pub valid_from: Option<String>,
    pub valid_to: Option<String>,
    pub source: Option<SourceAnchor>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CreateWorldEntry {
    #[serde(default)]
    pub project_id: ProjectId,
    pub kind: WorldKind,
    pub title: String,
    #[serde(default)]
    pub summary: String,
    pub prompt: Option<IdeaPrompt>,
    pub status: Option<WorldStatus>,
    pub board_x: Option<f64>,
    pub board_y: Option<f64>,
    pub map_x: Option<f64>,
    pub map_y: Option<f64>,
    pub hue: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct UpdateWorldEntry {
    pub kind: Option<WorldKind>,
    pub title: Option<String>,
    pub summary: Option<String>,
    pub prompt: Option<Option<IdeaPrompt>>,
    pub status: Option<WorldStatus>,
    pub board_x: Option<f64>,
    pub board_y: Option<f64>,
    pub map_x: Option<Option<f64>>,
    pub map_y: Option<Option<f64>>,
    pub hue: Option<String>,
}

#[async_trait]
pub trait Repository: Send + Sync {
    async fn list_projects(&self) -> Result<Vec<Project>>;
    async fn find_project(&self, id: ProjectId) -> Result<Option<Project>>;
    async fn insert_project(&self, project: &Project) -> Result<()>;
    async fn list_documents(&self, project_id: ProjectId) -> Result<Vec<Document>>;
    async fn find_document(&self, id: DocumentId) -> Result<Option<Document>>;
    async fn insert_document(&self, document: &Document) -> Result<()>;
    async fn update_document(&self, document: &Document) -> Result<()>;
    async fn list_facts(&self, project_id: ProjectId) -> Result<Vec<Fact>>;
    async fn insert_fact(&self, fact: &Fact) -> Result<()>;
    async fn list_world_entries(&self, project_id: ProjectId) -> Result<Vec<WorldEntry>>;
    async fn find_world_entry(&self, id: WorldEntryId) -> Result<Option<WorldEntry>>;
    async fn insert_world_entry(&self, entry: &WorldEntry) -> Result<()>;
    async fn update_world_entry(&self, entry: &WorldEntry) -> Result<()>;
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

    pub async fn get_document(&self, id: DocumentId) -> Result<Document> {
        self.repository
            .find_document(id)
            .await?
            .ok_or(ApplicationError::NotFound {
                entity: "document",
            })
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

    pub async fn update_document(
        &self,
        id: DocumentId,
        input: UpdateDocument,
    ) -> Result<Document> {
        let mut document = self.get_document(id).await?;
        if let Some(title) = input.title {
            let title = title.trim();
            if title.is_empty() {
                return Err(ApplicationError::Validation(
                    "document title cannot be empty".into(),
                ));
            }
            document.title = title.to_owned();
        }
        if let Some(content_json) = input.content_json {
            document.content_json = content_json;
        }
        document.revision += 1;
        document.updated_at = Utc::now();
        self.repository.update_document(&document).await?;
        Ok(document)
    }

    pub async fn list_facts(&self, project_id: ProjectId) -> Result<Vec<Fact>> {
        self.get_project(project_id).await?;
        self.repository.list_facts(project_id).await
    }

    pub async fn create_fact(&self, input: CreateFact) -> Result<Fact> {
        self.get_project(input.project_id).await?;
        let entity_key = normalize_key(&input.entity_key);
        let predicate = normalize_key(&input.predicate);
        let value = input.value.trim().to_owned();
        if entity_key.is_empty() || predicate.is_empty() || value.is_empty() {
            return Err(ApplicationError::Validation(
                "entity, predicate, and value are required".into(),
            ));
        }
        if let Some(source) = &input.source {
            self.get_document(source.document_id).await?;
        }
        let now = Utc::now();
        let fact = Fact {
            id: margin_domain::FactId::new(),
            project_id: input.project_id,
            entity_key,
            predicate,
            value,
            modality: input.modality,
            status: input.status,
            valid_from: empty_to_none(input.valid_from),
            valid_to: empty_to_none(input.valid_to),
            source: input.source,
            created_at: now,
            updated_at: now,
        };
        self.repository.insert_fact(&fact).await?;
        Ok(fact)
    }

    pub async fn analyze_project(&self, project_id: ProjectId) -> Result<Vec<Diagnostic>> {
        let project = self.get_project(project_id).await?;
        let documents = self.repository.list_documents(project_id).await?;
        let facts = self.repository.list_facts(project_id).await?;
        Ok(DiagnosticEngine::with_builtin_rules().analyze(&AnalysisContext {
            project: &project,
            documents: &documents,
            facts: &facts,
        }))
    }

    pub async fn list_world_entries(&self, project_id: ProjectId) -> Result<Vec<WorldEntry>> {
        self.get_project(project_id).await?;
        self.repository.list_world_entries(project_id).await
    }

    pub async fn get_world_entry(&self, id: WorldEntryId) -> Result<WorldEntry> {
        self.repository
            .find_world_entry(id)
            .await?
            .ok_or(ApplicationError::NotFound {
                entity: "world entry",
            })
    }

    pub async fn create_world_entry(&self, input: CreateWorldEntry) -> Result<WorldEntry> {
        self.get_project(input.project_id).await?;
        let title = input.title.trim();
        let summary = input.summary.trim();
        if title.is_empty() && summary.is_empty() {
            return Err(ApplicationError::Validation(
                "write a scrap, a place, or a question first".into(),
            ));
        }
        let title = if title.is_empty() {
            truncate_title(summary)
        } else {
            title.to_owned()
        };
        let now = Utc::now();
        let existing = self
            .repository
            .list_world_entries(input.project_id)
            .await?
            .len();
        let entry = WorldEntry {
            id: WorldEntryId::new(),
            project_id: input.project_id,
            kind: input.kind.clone(),
            title,
            summary: summary.to_owned(),
            prompt: input.prompt,
            status: input.status.unwrap_or_default(),
            board_x: input
                .board_x
                .unwrap_or(8.0 + ((existing * 13) % 68) as f64),
            board_y: input
                .board_y
                .unwrap_or(10.0 + ((existing * 17) % 56) as f64),
            map_x: input.map_x,
            map_y: input.map_y,
            hue: input
                .hue
                .map(|value| value.trim().to_owned())
                .filter(|value| !value.is_empty())
                .unwrap_or_else(|| default_world_hue(&input.kind).to_owned()),
            created_at: now,
            updated_at: now,
        };
        self.repository.insert_world_entry(&entry).await?;
        if entry.status == WorldStatus::Settled {
            self.mint_world_fact(&entry).await?;
        }
        Ok(entry)
    }

    pub async fn update_world_entry(
        &self,
        id: WorldEntryId,
        input: UpdateWorldEntry,
    ) -> Result<WorldEntry> {
        let mut entry = self.get_world_entry(id).await?;
        let was_settled = entry.status == WorldStatus::Settled;
        if let Some(kind) = input.kind {
            entry.kind = kind;
        }
        if let Some(title) = input.title {
            let title = title.trim();
            if title.is_empty() {
                return Err(ApplicationError::Validation(
                    "a card still needs a name".into(),
                ));
            }
            entry.title = title.to_owned();
        }
        if let Some(summary) = input.summary {
            entry.summary = summary.trim().to_owned();
        }
        if let Some(prompt) = input.prompt {
            entry.prompt = prompt;
        }
        if let Some(status) = input.status {
            entry.status = status;
        }
        if let Some(board_x) = input.board_x {
            entry.board_x = board_x.clamp(0.0, 92.0);
        }
        if let Some(board_y) = input.board_y {
            entry.board_y = board_y.clamp(0.0, 88.0);
        }
        if let Some(map_x) = input.map_x {
            entry.map_x = map_x.map(|value| value.clamp(2.0, 96.0));
        }
        if let Some(map_y) = input.map_y {
            entry.map_y = map_y.map(|value| value.clamp(2.0, 96.0));
        }
        if let Some(hue) = input.hue {
            let hue = hue.trim();
            if !hue.is_empty() {
                entry.hue = hue.to_owned();
            }
        }
        entry.updated_at = Utc::now();
        self.repository.update_world_entry(&entry).await?;
        if entry.status == WorldStatus::Settled && !was_settled {
            self.mint_world_fact(&entry).await?;
        }
        Ok(entry)
    }

    pub async fn world_pulse(&self, project_id: ProjectId) -> Result<WorldPulse> {
        self.get_project(project_id).await?;
        let entries = self.repository.list_world_entries(project_id).await?;
        let documents = self.repository.list_documents(project_id).await?;
        Ok(compute_world_pulse(&entries, &documents))
    }

    async fn mint_world_fact(&self, entry: &WorldEntry) -> Result<()> {
        let Some((predicate, value)) = fact_from_world(entry) else {
            return Ok(());
        };
        let entity_key = entity_key_from_title(&entry.title);
        if entity_key.is_empty() {
            return Ok(());
        }
        let existing = self.repository.list_facts(entry.project_id).await?;
        if existing.iter().any(|fact| {
            fact.entity_key == entity_key && fact.predicate == predicate && fact.value == value
        }) {
            return Ok(());
        }
        let now = Utc::now();
        self.repository
            .insert_fact(&Fact {
                id: FactId::new(),
                project_id: entry.project_id,
                entity_key,
                predicate,
                value,
                modality: FactModality::Established,
                status: FactStatus::Confirmed,
                valid_from: None,
                valid_to: None,
                source: None,
                created_at: now,
                updated_at: now,
            })
            .await
    }
}

fn fact_from_world(entry: &WorldEntry) -> Option<(String, String)> {
    let value = if entry.summary.is_empty() {
        entry.title.clone()
    } else {
        entry.summary.clone()
    };
    let predicate = match entry.kind {
        WorldKind::Place => "place",
        WorldKind::Culture => "culture",
        WorldKind::System => "rule",
        WorldKind::Creature => "creature",
        WorldKind::Item => "item",
        WorldKind::Question | WorldKind::Fragment => return None,
    };
    Some((predicate.to_owned(), value))
}

fn truncate_title(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.chars().count() <= 48 {
        return trimmed.to_owned();
    }
    let mut title: String = trimmed.chars().take(45).collect();
    title.push('…');
    title
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
