//! Use cases and the repository boundary. Transport layers (the HTTP API and
//! the Tauri shell) call these methods and never touch storage directly.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use asten_blocks::{DocState, EventRecord};
use asten_domain::{
    AuthToken, AuthTokenId, Baseline, BaselineId, BlockSnapshot, CanonEntity, Comment, CommentId,
    CompileArtifact, Document, DocumentId, DocumentKind, DocumentOp, DocumentSnapshot,
    DocumentSummary, Draft, DraftId, EntityId, Fact, FactId, FactModality, FactStatus, IdeaPrompt,
    InvariantId, MethodPack, MethodPackId, OwnerCredential, PairingCode, PairingCodeId, Project,
    ProjectId, ProjectKind, ProjectSettings, RouteId, ScenePlacement, SnapshotId, SourceAnchor,
    StoredInvariant, StoredRoute, Suppression, SuppressionId, TimelineEvent, TimelineEventId,
    TrackedChange, TrackedChangeId, WorldEntry, WorldEntryId, WorldKind, WorldPulse, WorldStatus,
    compute_world_pulse, default_world_hue, empty_prosemirror_doc, ensure_block_ids,
    entity_key_from_title, normalize_key,
};
use asten_engine::Engine;
use asten_ir::{Anchor, Context, Provenance};
use asten_ls::StreamEvent;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::broadcast;

pub mod auth;
mod compile;
mod engine_app;
pub mod history;
mod import;
pub mod story;

pub use asten_auth::AuthMode;
pub use compile::{CompileFormat, CompileRequest, compile_documents};
pub use engine_app::{
    CreateCanonEntity, CreateComment, CreateSnapshot, CreateSuppression, CreateTimelineEvent,
    CreateTrackedChange, ResolveTrackedChange, UpdateCanonEntity, UpdateComment, UpdateMethodPack,
    UpdateTimelineEvent,
};
pub use history::{CreateDraft, MergePreview, MergeRequest, UpdateDraft, VerifyReport};
pub use import::{ImportFormat, ImportRequest, imported_document_json, imported_title};
pub use story::{CreateBaseline, CreateInvariant, CreateRoute, UpdateSettings};

#[derive(Debug, Error)]
pub enum ApplicationError {
    #[error("{entity} was not found")]
    NotFound { entity: &'static str },
    #[error("invalid input: {0}")]
    Validation(String),
    #[error("storage operation failed: {0}")]
    Storage(String),
    #[error("not allowed: {0}")]
    Unauthorized(String),
    #[error("conflict: {0}")]
    Conflict(String),
}

pub type Result<T> = std::result::Result<T, ApplicationError>;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(
    feature = "ts",
    derive(ts_rs::TS),
    ts(export, export_to = "application/")
)]
pub struct DeskInfo {
    pub version: String,
    /// Operating system and processor of the machine running the API.
    pub host: String,
    /// True when the API is embedded in the desktop or mobile shell.
    pub embedded: bool,
    pub auth_mode: AuthMode,
    /// Where the SQLite file lives. Only desk and owner sessions see it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub database_path: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(
    feature = "ts",
    derive(ts_rs::TS),
    ts(export, export_to = "application/")
)]
#[cfg_attr(feature = "ts", ts(optional_fields = nullable))]
pub struct CreateProject {
    pub name: String,
    pub kind: ProjectKind,
    pub description: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(
    feature = "ts",
    derive(ts_rs::TS),
    ts(export, export_to = "application/")
)]
#[cfg_attr(feature = "ts", ts(optional_fields = nullable))]
pub struct CreateDocument {
    #[serde(default)]
    #[cfg_attr(feature = "ts", ts(as = "Option<_>", optional))]
    pub project_id: ProjectId,
    pub parent_id: Option<DocumentId>,
    pub kind: DocumentKind,
    pub title: String,
    #[serde(default)]
    #[cfg_attr(feature = "ts", ts(as = "Option<_>", optional))]
    pub content_json: serde_json::Value,
    #[serde(default)]
    #[cfg_attr(feature = "ts", ts(as = "Option<_>", optional))]
    pub position: i64,
    #[serde(default)]
    pub scene: Option<ScenePlacement>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(
    feature = "ts",
    derive(ts_rs::TS),
    ts(export, export_to = "application/")
)]
#[cfg_attr(feature = "ts", ts(optional_fields = nullable))]
pub struct UpdateDocument {
    pub title: Option<String>,
    pub content_json: Option<serde_json::Value>,
    pub parent_id: Option<Option<DocumentId>>,
    pub position: Option<i64>,
    pub exclude_from_compile: Option<bool>,
    #[serde(default)]
    pub scene: Option<ScenePlacement>,
    #[serde(default)]
    pub actor: Option<String>,
    /// Optimistic concurrency: refuse when the document moved past this head.
    #[serde(default)]
    pub expected_head_seq: Option<i64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(
    feature = "ts",
    derive(ts_rs::TS),
    ts(export, export_to = "application/")
)]
#[cfg_attr(feature = "ts", ts(optional_fields = nullable))]
pub struct CreateFact {
    #[serde(default)]
    #[cfg_attr(feature = "ts", ts(as = "Option<_>", optional))]
    pub project_id: ProjectId,
    pub entity_key: String,
    pub predicate: String,
    pub value: String,
    #[serde(default)]
    #[cfg_attr(feature = "ts", ts(as = "Option<_>", optional))]
    pub modality: FactModality,
    #[serde(default)]
    #[cfg_attr(feature = "ts", ts(as = "Option<_>", optional))]
    pub status: FactStatus,
    pub valid_from: Option<String>,
    pub valid_to: Option<String>,
    #[serde(default)]
    pub source: Option<SourceAnchor>,
    #[serde(default)]
    pub context: Option<Context>,
    #[serde(default)]
    pub anchor: Option<Anchor>,
    #[serde(default)]
    pub confidence: Option<f32>,
    #[serde(default)]
    pub provenance: Option<Provenance>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(
    feature = "ts",
    derive(ts_rs::TS),
    ts(export, export_to = "application/")
)]
#[cfg_attr(feature = "ts", ts(optional_fields = nullable))]
pub struct UpdateFact {
    pub value: Option<String>,
    pub predicate: Option<String>,
    pub status: Option<FactStatus>,
    pub modality: Option<FactModality>,
    pub valid_from: Option<Option<String>>,
    pub valid_to: Option<Option<String>>,
    pub context: Option<Context>,
    pub anchor: Option<Option<Anchor>>,
    #[serde(default)]
    pub actor: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(
    feature = "ts",
    derive(ts_rs::TS),
    ts(export, export_to = "application/")
)]
#[cfg_attr(feature = "ts", ts(optional_fields = nullable))]
pub struct CreateWorldEntry {
    #[serde(default)]
    #[cfg_attr(feature = "ts", ts(as = "Option<_>", optional))]
    pub project_id: ProjectId,
    pub kind: WorldKind,
    pub title: String,
    #[serde(default)]
    #[cfg_attr(feature = "ts", ts(as = "Option<_>", optional))]
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
#[cfg_attr(
    feature = "ts",
    derive(ts_rs::TS),
    ts(export, export_to = "application/")
)]
#[cfg_attr(feature = "ts", ts(optional_fields = nullable))]
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

/// The storage boundary. One implementation today: SQLite.
#[async_trait]
pub trait Repository: Send + Sync {
    async fn list_projects(&self) -> Result<Vec<Project>>;
    async fn find_project(&self, id: ProjectId) -> Result<Option<Project>>;
    async fn insert_project(&self, project: &Project) -> Result<()>;
    async fn update_project(&self, project: &Project) -> Result<()>;

    async fn list_documents(&self, project_id: ProjectId) -> Result<Vec<Document>>;
    async fn find_document(&self, id: DocumentId) -> Result<Option<Document>>;
    async fn insert_document(&self, document: &Document) -> Result<()>;
    async fn update_document(&self, document: &Document) -> Result<()>;
    async fn delete_document(&self, id: DocumentId) -> Result<()>;
    /// Atomically store a document change: the row, its materialised state and
    /// the new hash-chained events.
    async fn commit_document(
        &self,
        document: &Document,
        state: &DocState,
        events: &[EventRecord],
    ) -> Result<()>;
    async fn document_state(&self, id: DocumentId) -> Result<Option<DocState>>;
    async fn list_block_events(
        &self,
        document_id: DocumentId,
        from_seq: i64,
    ) -> Result<Vec<EventRecord>>;
    async fn insert_block_snapshot(&self, snapshot: &BlockSnapshot) -> Result<()>;
    async fn latest_block_snapshot(
        &self,
        document_id: DocumentId,
        through_seq: Option<i64>,
    ) -> Result<Option<BlockSnapshot>>;

    async fn list_facts(&self, project_id: ProjectId) -> Result<Vec<Fact>>;
    async fn find_fact(&self, id: FactId) -> Result<Option<Fact>>;
    async fn insert_fact(&self, fact: &Fact) -> Result<()>;
    async fn update_fact(&self, fact: &Fact) -> Result<()>;
    async fn delete_fact(&self, id: FactId) -> Result<()>;

    async fn list_world_entries(&self, project_id: ProjectId) -> Result<Vec<WorldEntry>>;
    async fn find_world_entry(&self, id: WorldEntryId) -> Result<Option<WorldEntry>>;
    async fn insert_world_entry(&self, entry: &WorldEntry) -> Result<()>;
    async fn update_world_entry(&self, entry: &WorldEntry) -> Result<()>;

    async fn list_entities(&self, project_id: ProjectId) -> Result<Vec<CanonEntity>>;
    async fn find_entity(&self, id: EntityId) -> Result<Option<CanonEntity>>;
    async fn insert_entity(&self, entity: &CanonEntity) -> Result<()>;
    async fn update_entity(&self, entity: &CanonEntity) -> Result<()>;
    async fn delete_entity(&self, id: EntityId) -> Result<()>;

    async fn list_events(&self, project_id: ProjectId) -> Result<Vec<TimelineEvent>>;
    async fn find_event(&self, id: TimelineEventId) -> Result<Option<TimelineEvent>>;
    async fn insert_event(&self, event: &TimelineEvent) -> Result<()>;
    async fn update_event(&self, event: &TimelineEvent) -> Result<()>;
    async fn delete_event(&self, id: TimelineEventId) -> Result<()>;

    async fn list_comments(&self, document_id: DocumentId) -> Result<Vec<Comment>>;
    async fn find_comment(&self, id: CommentId) -> Result<Option<Comment>>;
    async fn insert_comment(&self, comment: &Comment) -> Result<()>;
    async fn update_comment(&self, comment: &Comment) -> Result<()>;

    async fn next_op_seq(&self, document_id: DocumentId) -> Result<i64>;
    async fn insert_op(&self, op: &DocumentOp) -> Result<()>;
    async fn list_ops(&self, document_id: DocumentId) -> Result<Vec<DocumentOp>>;

    async fn insert_snapshot(&self, snapshot: &DocumentSnapshot) -> Result<()>;
    async fn list_snapshots(&self, document_id: DocumentId) -> Result<Vec<DocumentSnapshot>>;
    async fn find_snapshot(&self, id: SnapshotId) -> Result<Option<DocumentSnapshot>>;

    async fn list_suppressions(&self, project_id: ProjectId) -> Result<Vec<Suppression>>;
    async fn find_suppression(&self, id: SuppressionId) -> Result<Option<Suppression>>;
    async fn insert_suppression(&self, suppression: &Suppression) -> Result<()>;
    async fn update_suppression(&self, suppression: &Suppression) -> Result<()>;
    async fn delete_suppression(&self, id: SuppressionId) -> Result<()>;

    async fn list_packs(&self) -> Result<Vec<MethodPack>>;
    async fn update_pack(&self, pack: &MethodPack) -> Result<()>;
    async fn find_pack(&self, id: MethodPackId) -> Result<Option<MethodPack>>;

    async fn list_tracked_changes(&self, document_id: DocumentId) -> Result<Vec<TrackedChange>>;
    async fn insert_tracked_change(&self, change: &TrackedChange) -> Result<()>;
    async fn update_tracked_change(&self, change: &TrackedChange) -> Result<()>;
    async fn find_tracked_change(&self, id: TrackedChangeId) -> Result<Option<TrackedChange>>;

    async fn list_drafts(&self, document_id: DocumentId) -> Result<Vec<Draft>>;
    async fn find_draft(&self, id: DraftId) -> Result<Option<Draft>>;
    async fn insert_draft(&self, draft: &Draft) -> Result<()>;
    async fn update_draft(&self, draft: &Draft) -> Result<()>;
    async fn delete_draft(&self, id: DraftId) -> Result<()>;

    async fn get_settings(&self, project_id: ProjectId) -> Result<Option<ProjectSettings>>;
    async fn put_settings(&self, settings: &ProjectSettings) -> Result<()>;

    async fn list_invariants(&self, project_id: ProjectId) -> Result<Vec<StoredInvariant>>;
    async fn insert_invariant(&self, invariant: &StoredInvariant) -> Result<()>;
    async fn update_invariant(&self, invariant: &StoredInvariant) -> Result<()>;
    async fn delete_invariant(&self, id: InvariantId) -> Result<()>;

    async fn list_routes(&self, project_id: ProjectId) -> Result<Vec<StoredRoute>>;
    async fn insert_route(&self, route: &StoredRoute) -> Result<()>;
    async fn delete_route(&self, id: RouteId) -> Result<()>;

    async fn list_baselines(&self, project_id: ProjectId) -> Result<Vec<Baseline>>;
    async fn insert_baseline(&self, baseline: &Baseline) -> Result<()>;
    async fn update_baseline(&self, baseline: &Baseline) -> Result<()>;
    async fn delete_baseline(&self, id: BaselineId) -> Result<()>;

    async fn list_tokens(&self) -> Result<Vec<AuthToken>>;
    async fn find_token_by_hash(&self, hash: &str) -> Result<Option<AuthToken>>;
    async fn insert_token(&self, token: &AuthToken) -> Result<()>;
    async fn update_token(&self, token: &AuthToken) -> Result<()>;
    async fn delete_token(&self, id: AuthTokenId) -> Result<()>;

    async fn list_pairing_codes(&self) -> Result<Vec<PairingCode>>;
    async fn insert_pairing_code(&self, code: &PairingCode) -> Result<()>;
    async fn update_pairing_code(&self, code: &PairingCode) -> Result<()>;
    /// Count one wrong guess against a code, atomically.
    async fn record_pairing_failure(&self, id: PairingCodeId) -> Result<()>;
    /// Mark a code consumed if, and only if, it is still open. `false` when
    /// another request got there first or the code is locked.
    async fn consume_pairing_code(&self, id: PairingCodeId, at: DateTime<Utc>) -> Result<bool>;
    async fn delete_pairing_code(&self, id: PairingCodeId) -> Result<()>;

    async fn owner_credential(&self) -> Result<Option<OwnerCredential>>;
    /// Store the first owner credential. `false` when one already exists.
    async fn create_owner_credential(&self, credential: &OwnerCredential) -> Result<bool>;
    /// Replace the owner credential.
    async fn set_owner_credential(&self, credential: &OwnerCredential) -> Result<()>;
}

/// The application: use cases over a repository, one canon engine per open
/// project, and a change stream per project.
#[derive(Clone)]
pub struct Application {
    repository: Arc<dyn Repository>,
    engines: Arc<Mutex<HashMap<ProjectId, Engine>>>,
    streams: Arc<Mutex<HashMap<ProjectId, broadcast::Sender<StreamEvent>>>>,
    auth_mode: AuthMode,
    /// Per-client buckets for the authentication routes.
    rate_limiter: Arc<asten_auth::RateLimiter>,
    /// One bucket per route, shared by every client, so a crowd of fresh
    /// addresses cannot multiply the per-client allowance.
    global_limiter: Arc<asten_auth::RateLimiter>,
    stream_tickets: Arc<Mutex<HashMap<String, auth::StreamTicket>>>,
}

impl Application {
    #[must_use]
    pub fn new(repository: Arc<dyn Repository>) -> Self {
        Self::with_auth(repository, AuthMode::Pairing)
    }

    #[must_use]
    pub fn with_auth(repository: Arc<dyn Repository>, auth_mode: AuthMode) -> Self {
        Self {
            repository,
            engines: Arc::new(Mutex::new(HashMap::new())),
            streams: Arc::new(Mutex::new(HashMap::new())),
            auth_mode,
            rate_limiter: Arc::new(asten_auth::RateLimiter::new(10, 10)),
            global_limiter: Arc::new(asten_auth::RateLimiter::new(60, 60)),
            stream_tickets: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    #[must_use]
    pub fn auth_mode(&self) -> AuthMode {
        self.auth_mode
    }

    #[must_use]
    pub fn repository(&self) -> &Arc<dyn Repository> {
        &self.repository
    }

    /// Subscribe to a project's change stream.
    pub fn subscribe(&self, project_id: ProjectId) -> broadcast::Receiver<StreamEvent> {
        let mut streams = self
            .streams
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        streams
            .entry(project_id)
            .or_insert_with(|| broadcast::channel(256).0)
            .subscribe()
    }

    pub(crate) fn notify(&self, project_id: ProjectId, event: StreamEvent) {
        let streams = self
            .streams
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(sender) = streams.get(&project_id) {
            let _ = sender.send(event);
        }
    }

    /// Re-analyse in the background and push the diagnostics delta to subscribers.
    pub(crate) fn schedule_refresh(&self, project_id: ProjectId) {
        self.notify(
            project_id,
            StreamEvent::Canon {
                what: "changed".into(),
            },
        );
        if tokio::runtime::Handle::try_current().is_err() {
            return;
        }
        let app = self.clone();
        tokio::spawn(async move {
            if let Err(error) = app.analyze_project(project_id).await {
                tracing::warn!(%project_id, %error, "background analysis failed");
            }
        });
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
        self.repository
            .put_settings(&ProjectSettings::for_project(project.id))
            .await?;
        Ok(project)
    }

    pub async fn list_documents(&self, project_id: ProjectId) -> Result<Vec<Document>> {
        self.get_project(project_id).await?;
        self.repository.list_documents(project_id).await
    }

    /// Every document of the project without its body. The desk lists these
    /// and fetches one body at a time with `get_project_document`, so a
    /// keystroke-commit on a long manuscript never moves the whole book.
    pub async fn list_document_summaries(
        &self,
        project_id: ProjectId,
    ) -> Result<Vec<DocumentSummary>> {
        Ok(self
            .list_documents(project_id)
            .await?
            .iter()
            .map(document_summary)
            .collect())
    }

    pub async fn get_document(&self, id: DocumentId) -> Result<Document> {
        self.repository
            .find_document(id)
            .await?
            .ok_or(ApplicationError::NotFound { entity: "document" })
    }

    /// A document addressed through its project: not found when it belongs
    /// to another one.
    pub async fn get_project_document(
        &self,
        project_id: ProjectId,
        id: DocumentId,
    ) -> Result<Document> {
        let document = self.get_document(id).await?;
        if document.project_id != project_id {
            return Err(ApplicationError::NotFound { entity: "document" });
        }
        Ok(document)
    }

    pub async fn create_document(&self, input: CreateDocument) -> Result<Document> {
        self.get_project(input.project_id).await?;
        let title = input.title.trim();
        if title.is_empty() {
            return Err(ApplicationError::Validation(
                "document title cannot be empty".into(),
            ));
        }
        let content_json =
            if input.content_json.is_null() || input.content_json == serde_json::json!({}) {
                empty_prosemirror_doc()
            } else {
                ensure_block_ids(input.content_json)
            };
        let now = Utc::now();
        let mut document = Document {
            id: DocumentId::new(),
            project_id: input.project_id,
            parent_id: input.parent_id,
            kind: input.kind,
            title: title.to_owned(),
            content_json: empty_prosemirror_doc(),
            position: input.position,
            revision: 0,
            exclude_from_compile: false,
            scene: input.scene.unwrap_or_default(),
            head_seq: 0,
            chain_hash: asten_blocks::ZERO_HASH_HEX.to_owned(),
            created_at: now,
            updated_at: now,
        };
        self.repository.insert_document(&document).await?;
        // The first commit writes the genesis events for the initial content.
        document = self
            .commit_content(document, content_json, "author")
            .await?;
        self.schedule_refresh(document.project_id);
        Ok(document)
    }

    pub async fn update_document(&self, id: DocumentId, input: UpdateDocument) -> Result<Document> {
        let mut document = self.get_document(id).await?;
        if let Some(expected) = input.expected_head_seq {
            if expected != document.head_seq {
                return Err(ApplicationError::Conflict(format!(
                    "the document moved on (head {} vs expected {expected}); reload before saving",
                    document.head_seq
                )));
            }
        }
        let mut meta_changed = false;
        if let Some(title) = input.title {
            let title = title.trim();
            if title.is_empty() {
                return Err(ApplicationError::Validation(
                    "document title cannot be empty".into(),
                ));
            }
            if document.title != title {
                document.title = title.to_owned();
                meta_changed = true;
            }
        }
        if let Some(parent_id) = input.parent_id {
            if parent_id == Some(document.id) {
                return Err(ApplicationError::Validation(
                    "a document cannot be its own parent".into(),
                ));
            }
            if document.parent_id != parent_id {
                document.parent_id = parent_id;
                meta_changed = true;
            }
        }
        if let Some(position) = input.position {
            if document.position != position {
                document.position = position;
                meta_changed = true;
            }
        }
        if let Some(exclude_from_compile) = input.exclude_from_compile {
            if document.exclude_from_compile != exclude_from_compile {
                document.exclude_from_compile = exclude_from_compile;
                meta_changed = true;
            }
        }
        if let Some(scene) = input.scene {
            if document.scene != scene {
                document.scene = scene;
                meta_changed = true;
            }
        }
        let actor = input.actor.unwrap_or_else(|| "author".into());
        if let Some(content_json) = input.content_json {
            document = self
                .commit_content(document, ensure_block_ids(content_json), &actor)
                .await?;
            if meta_changed {
                document.updated_at = Utc::now();
                self.repository.update_document(&document).await?;
            }
        } else if meta_changed {
            document.revision += 1;
            document.updated_at = Utc::now();
            self.repository.update_document(&document).await?;
        }
        self.notify(
            document.project_id,
            StreamEvent::Document {
                document: asten_ir::DocId::new(document.id.to_string()),
                seq: document.head_seq,
            },
        );
        self.schedule_refresh(document.project_id);
        Ok(document)
    }

    pub async fn delete_document(&self, id: DocumentId) -> Result<()> {
        let document = self.get_document(id).await?;
        self.repository.delete_document(id).await?;
        self.schedule_refresh(document.project_id);
        Ok(())
    }

    pub async fn list_facts(&self, project_id: ProjectId) -> Result<Vec<Fact>> {
        self.get_project(project_id).await?;
        self.repository.list_facts(project_id).await
    }

    pub async fn get_fact(&self, id: FactId) -> Result<Fact> {
        self.repository
            .find_fact(id)
            .await?
            .ok_or(ApplicationError::NotFound { entity: "fact" })
    }

    pub async fn create_fact(&self, input: CreateFact) -> Result<Fact> {
        self.get_project(input.project_id).await?;
        let entity_key = entity_key_from_title(&input.entity_key);
        let predicate = entity_key_from_title(&input.predicate);
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
        let status = input.status;
        let fact = Fact {
            id: FactId::new(),
            project_id: input.project_id,
            entity_key,
            predicate,
            value,
            modality: input.modality,
            status,
            valid_from: empty_to_none(input.valid_from),
            valid_to: empty_to_none(input.valid_to),
            source: input.source,
            context: input.context.unwrap_or(Context::World),
            confidence: input.confidence.unwrap_or(1.0).clamp(0.0, 1.0),
            anchor: input.anchor,
            provenance: input.provenance.unwrap_or(Provenance::Author {
                action: "create".into(),
            }),
            settled_by: (status == FactStatus::Confirmed).then(|| "author".to_owned()),
            settled_at: (status == FactStatus::Confirmed).then_some(now),
            created_at: now,
            updated_at: now,
        };
        self.repository.insert_fact(&fact).await?;
        self.schedule_refresh(fact.project_id);
        Ok(fact)
    }

    pub async fn update_fact(&self, id: FactId, input: UpdateFact) -> Result<Fact> {
        let mut fact = self.get_fact(id).await?;
        if let Some(value) = input.value {
            let value = value.trim();
            if value.is_empty() {
                return Err(ApplicationError::Validation("a value is required".into()));
            }
            fact.value = value.to_owned();
        }
        if let Some(predicate) = input.predicate {
            fact.predicate = entity_key_from_title(&predicate);
        }
        if let Some(status) = input.status {
            if status == FactStatus::Confirmed && fact.status != FactStatus::Confirmed {
                fact.settled_by = Some(input.actor.clone().unwrap_or_else(|| "author".into()));
                fact.settled_at = Some(Utc::now());
                fact.confidence = 1.0;
            }
            fact.status = status;
        }
        if let Some(modality) = input.modality {
            fact.modality = modality;
        }
        if let Some(valid_from) = input.valid_from {
            fact.valid_from = empty_to_none(valid_from);
        }
        if let Some(valid_to) = input.valid_to {
            fact.valid_to = empty_to_none(valid_to);
        }
        if let Some(context) = input.context {
            fact.context = context;
        }
        if let Some(anchor) = input.anchor {
            fact.anchor = anchor;
        }
        fact.updated_at = Utc::now();
        self.repository.update_fact(&fact).await?;
        self.schedule_refresh(fact.project_id);
        Ok(fact)
    }

    pub async fn delete_fact(&self, id: FactId) -> Result<()> {
        let fact = self.get_fact(id).await?;
        self.repository.delete_fact(id).await?;
        self.schedule_refresh(fact.project_id);
        Ok(())
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
            board_x: input.board_x.unwrap_or(8.0 + ((existing * 13) % 68) as f64),
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
        self.schedule_refresh(entry.project_id);
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
        self.schedule_refresh(entry.project_id);
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
                context: Context::World,
                confidence: 1.0,
                anchor: None,
                provenance: Provenance::Wiki {
                    document: asten_ir::DocId::new(entry.id.to_string()),
                },
                settled_by: Some("author".into()),
                settled_at: Some(now),
                created_at: now,
                updated_at: now,
            })
            .await
    }

    pub async fn compile_project(
        &self,
        project_id: ProjectId,
        request: CompileRequest,
    ) -> Result<CompileArtifact> {
        let project = self.get_project(project_id).await?;
        let mut documents = self.repository.list_documents(project_id).await?;
        if !request.document_ids.is_empty() {
            documents.retain(|document| request.document_ids.contains(&document.id));
        }
        compile_documents(&project.name, &documents, &request.format)
    }

    pub async fn import_document(
        &self,
        project_id: ProjectId,
        request: ImportRequest,
    ) -> Result<Document> {
        let content_json = imported_document_json(&request)?;
        self.create_document(CreateDocument {
            project_id,
            parent_id: request.parent_id,
            kind: request.kind.unwrap_or(DocumentKind::Manuscript),
            title: imported_title(&request.filename),
            content_json,
            position: 0,
            scene: None,
        })
        .await
    }

    /// Entities by key, for mapping engine ids back to rows.
    pub(crate) async fn entity_by_key(
        &self,
        project_id: ProjectId,
        key: &str,
    ) -> Result<Option<CanonEntity>> {
        let entities = self.repository.list_entities(project_id).await?;
        Ok(entities
            .into_iter()
            .find(|entity| entity.key() == key || normalize_key(&entity.name) == key))
    }
}

/// The document without its body, with `words` counted by the one rule the
/// desk and the API share (`asten_blocks::word_count`).
#[must_use]
pub fn document_summary(document: &Document) -> DocumentSummary {
    document.summary(asten_blocks::word_count(&document.content_json) as u64)
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

pub(crate) fn empty_to_none(value: Option<String>) -> Option<String> {
    value.and_then(|item| {
        let trimmed = item.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_owned())
        }
    })
}
