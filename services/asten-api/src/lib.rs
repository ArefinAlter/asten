mod extra;

use std::{sync::Arc, time::Duration};

use asten_application::{
    Application, ApplicationError, CreateDocument, CreateFact, CreateProject, CreateWorldEntry,
    UpdateDocument, UpdateWorldEntry,
};
use asten_domain::{
    Diagnostic, Document, DocumentId, Fact, Project, ProjectId, WorldEntry, WorldEntryId,
    WorldPulse,
};
use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, Path, State},
    http::{HeaderName, Method, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
};
use serde::Serialize;
use tower_http::{
    catch_panic::CatchPanicLayer,
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    timeout::TimeoutLayer,
    trace::TraceLayer,
};
use uuid::Uuid;

#[derive(Clone)]
pub struct ApiState {
    pub application: Application,
}

pub fn router(application: Application) -> Router {
    let request_id = HeaderName::from_static("x-request-id");
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_headers(Any)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE]);

    Router::new()
        .route("/health", get(health))
        .nest(
            "/api/v1",
            Router::new()
                .route("/projects", get(list_projects).post(create_project))
                .route("/projects/{project_id}", get(get_project))
                .route(
                    "/projects/{project_id}/documents",
                    get(list_documents).post(create_document),
                )
                .route(
                    "/projects/{project_id}/documents/{document_id}",
                    axum::routing::put(update_document),
                )
                .route(
                    "/projects/{project_id}/facts",
                    get(list_facts).post(create_fact),
                )
                .route("/projects/{project_id}/diagnostics", get(list_diagnostics))
                .route(
                    "/projects/{project_id}/world-entries",
                    get(list_world_entries).post(create_world_entry),
                )
                .route(
                    "/projects/{project_id}/world-entries/{entry_id}",
                    axum::routing::put(update_world_entry),
                )
                .route("/projects/{project_id}/world-pulse", get(world_pulse))
                .merge(extra::router()),
        )
        .with_state(ApiState { application })
        .layer(DefaultBodyLimit::max(10 * 1024 * 1024))
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            Duration::from_secs(30),
        ))
        .layer(CompressionLayer::new())
        .layer(cors)
        .layer(PropagateRequestIdLayer::new(request_id.clone()))
        .layer(SetRequestIdLayer::new(request_id, MakeRequestUuid))
        .layer(TraceLayer::new_for_http())
        .layer(CatchPanicLayer::new())
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: "asten-api",
    })
}

async fn list_projects(State(state): State<ApiState>) -> Result<Json<Vec<Project>>, ApiError> {
    Ok(Json(state.application.list_projects().await?))
}

async fn get_project(
    State(state): State<ApiState>,
    Path(project_id): Path<Uuid>,
) -> Result<Json<Project>, ApiError> {
    Ok(Json(
        state.application.get_project(ProjectId(project_id)).await?,
    ))
}

async fn create_project(
    State(state): State<ApiState>,
    Json(input): Json<CreateProject>,
) -> Result<(StatusCode, Json<Project>), ApiError> {
    let project = state.application.create_project(input).await?;
    Ok((StatusCode::CREATED, Json(project)))
}

async fn list_documents(
    State(state): State<ApiState>,
    Path(project_id): Path<Uuid>,
) -> Result<Json<Vec<Document>>, ApiError> {
    Ok(Json(
        state
            .application
            .list_documents(ProjectId(project_id))
            .await?,
    ))
}

async fn create_document(
    State(state): State<ApiState>,
    Path(project_id): Path<Uuid>,
    Json(mut input): Json<CreateDocument>,
) -> Result<(StatusCode, Json<Document>), ApiError> {
    input.project_id = ProjectId(project_id);
    let document = state.application.create_document(input).await?;
    Ok((StatusCode::CREATED, Json(document)))
}

async fn update_document(
    State(state): State<ApiState>,
    Path((_, document_id)): Path<(Uuid, Uuid)>,
    Json(input): Json<UpdateDocument>,
) -> Result<Json<Document>, ApiError> {
    Ok(Json(
        state
            .application
            .update_document(DocumentId(document_id), input)
            .await?,
    ))
}

async fn list_facts(
    State(state): State<ApiState>,
    Path(project_id): Path<Uuid>,
) -> Result<Json<Vec<Fact>>, ApiError> {
    Ok(Json(
        state.application.list_facts(ProjectId(project_id)).await?,
    ))
}

async fn create_fact(
    State(state): State<ApiState>,
    Path(project_id): Path<Uuid>,
    Json(mut input): Json<CreateFact>,
) -> Result<(StatusCode, Json<Fact>), ApiError> {
    input.project_id = ProjectId(project_id);
    let fact = state.application.create_fact(input).await?;
    Ok((StatusCode::CREATED, Json(fact)))
}

async fn list_diagnostics(
    State(state): State<ApiState>,
    Path(project_id): Path<Uuid>,
) -> Result<Json<Vec<Diagnostic>>, ApiError> {
    Ok(Json(
        state
            .application
            .analyze_project(ProjectId(project_id))
            .await?,
    ))
}

async fn list_world_entries(
    State(state): State<ApiState>,
    Path(project_id): Path<Uuid>,
) -> Result<Json<Vec<WorldEntry>>, ApiError> {
    Ok(Json(
        state
            .application
            .list_world_entries(ProjectId(project_id))
            .await?,
    ))
}

async fn create_world_entry(
    State(state): State<ApiState>,
    Path(project_id): Path<Uuid>,
    Json(mut input): Json<CreateWorldEntry>,
) -> Result<(StatusCode, Json<WorldEntry>), ApiError> {
    input.project_id = ProjectId(project_id);
    let entry = state.application.create_world_entry(input).await?;
    Ok((StatusCode::CREATED, Json(entry)))
}

async fn update_world_entry(
    State(state): State<ApiState>,
    Path((_, entry_id)): Path<(Uuid, Uuid)>,
    Json(input): Json<UpdateWorldEntry>,
) -> Result<Json<WorldEntry>, ApiError> {
    Ok(Json(
        state
            .application
            .update_world_entry(WorldEntryId(entry_id), input)
            .await?,
    ))
}

async fn world_pulse(
    State(state): State<ApiState>,
    Path(project_id): Path<Uuid>,
) -> Result<Json<WorldPulse>, ApiError> {
    Ok(Json(
        state.application.world_pulse(ProjectId(project_id)).await?,
    ))
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    service: &'static str,
}

pub struct ApiError(ApplicationError);

impl From<ApplicationError> for ApiError {
    fn from(value: ApplicationError) -> Self {
        Self(value)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = match self.0 {
            ApplicationError::NotFound { .. } => StatusCode::NOT_FOUND,
            ApplicationError::Validation(_) => StatusCode::UNPROCESSABLE_ENTITY,
            ApplicationError::Storage(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        let body = Json(ErrorResponse {
            error: ErrorBody {
                code: status.as_str().to_owned(),
                message: self.0.to_string(),
            },
        });
        (status, body).into_response()
    }
}

#[derive(Serialize)]
struct ErrorResponse {
    error: ErrorBody,
}

#[derive(Serialize)]
struct ErrorBody {
    code: String,
    message: String,
}

pub async fn application_for_database(path: &str) -> anyhow::Result<Application> {
    let repository = asten_storage::SqliteRepository::open(path).await?;
    Ok(Application::new(Arc::new(repository)))
}
