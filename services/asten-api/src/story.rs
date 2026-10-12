//! The story language server over HTTP, the change stream, settings,
//! invariants, routes and baselines.

use std::convert::Infallible;

use asten_application::{CreateBaseline, CreateInvariant, CreateRoute, UpdateSettings};
use asten_domain::{
    Baseline, BaselineId, DocumentId, EntityId, InvariantId, ProjectId, ProjectSettings, RouteId,
    StoredInvariant, StoredRoute,
};
use asten_ls::{
    ApplyRepairRequest, ApplyRepairResponse, BisectResult, CastSearchHit, DiagnosticsSnapshot,
    FactCard, MentionRef, RenamePreview, RenameRequest, SceneSummary, StreamEvent, WhereSettled,
};
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::sse::{Event, KeepAlive, Sse},
    routing::{get, post},
};
use futures::stream::{self, Stream, StreamExt};
use serde::Deserialize;
use tokio_stream::wrappers::BroadcastStream;
use uuid::Uuid;

use crate::{ApiError, ApiState};

pub fn router() -> Router<ApiState> {
    Router::new()
        .route("/projects/{project_id}/stream", get(stream))
        .route(
            "/projects/{project_id}/settings",
            get(get_settings).put(update_settings),
        )
        .route("/projects/{project_id}/story/diagnostics", get(diagnostics))
        .route("/projects/{project_id}/story/fact-card", get(fact_card))
        .route("/projects/{project_id}/story/mentions", get(mentions))
        .route(
            "/projects/{project_id}/story/where-settled",
            get(where_settled),
        )
        .route(
            "/projects/{project_id}/story/scene-summary/{document_id}",
            get(scene_summary),
        )
        .route("/projects/{project_id}/story/cast-search", get(cast_search))
        .route(
            "/projects/{project_id}/story/repairs/apply",
            post(apply_repair),
        )
        .route(
            "/projects/{project_id}/story/bisect/{diagnostic_id}",
            get(bisect),
        )
        .route("/projects/{project_id}/story/catalog", get(catalog))
        .route(
            "/projects/{project_id}/entities/{entity_id}/rename",
            post(rename),
        )
        .route(
            "/projects/{project_id}/invariants",
            get(list_invariants).post(create_invariant),
        )
        .route(
            "/projects/{project_id}/invariants/{invariant_id}",
            axum::routing::put(update_invariant).delete(delete_invariant),
        )
        .route(
            "/projects/{project_id}/routes",
            get(list_routes).post(create_route),
        )
        .route(
            "/projects/{project_id}/routes/{route_id}",
            axum::routing::delete(delete_route),
        )
        .route(
            "/projects/{project_id}/baselines",
            get(list_baselines).post(create_baseline),
        )
        .route(
            "/projects/{project_id}/baselines/{baseline_id}/activate",
            post(activate_baseline),
        )
        .route(
            "/projects/{project_id}/baselines/{baseline_id}",
            axum::routing::delete(delete_baseline),
        )
        .route(
            "/projects/{project_id}/baselines/clear",
            post(clear_baseline),
        )
}

/// Server-sent events: revisions, diagnostics deltas, document and canon changes.
async fn stream(
    State(state): State<ApiState>,
    Path(project_id): Path<Uuid>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, ApiError> {
    let project_id = ProjectId(project_id);
    state.application.get_project(project_id).await?;
    let receiver = state.application.subscribe(project_id);
    // Say hello at once so the connection is established before the first change.
    let hello = stream::once(async { Ok(Event::default().event("heartbeat").data("{}")) });
    let changes = BroadcastStream::new(receiver).filter_map(|item| async move {
        match item {
            Ok(event) => {
                let name = match &event {
                    StreamEvent::Revision { .. } => "revision",
                    StreamEvent::Diagnostics(_) => "diagnostics",
                    StreamEvent::Document { .. } => "document",
                    StreamEvent::Canon { .. } => "canon",
                    StreamEvent::Heartbeat => "heartbeat",
                };
                let data = serde_json::to_string(&event).unwrap_or_else(|_| "{}".into());
                Some(Ok(Event::default().event(name).data(data)))
            }
            Err(_) => Some(Ok(Event::default().event("lagged").data("{}"))),
        }
    });
    Ok(Sse::new(hello.chain(changes)).keep_alive(
        KeepAlive::new()
            .interval(std::time::Duration::from_secs(20))
            .text("ping"),
    ))
}

async fn get_settings(
    State(state): State<ApiState>,
    Path(project_id): Path<Uuid>,
) -> Result<Json<ProjectSettings>, ApiError> {
    Ok(Json(
        state
            .application
            .get_settings(ProjectId(project_id))
            .await?,
    ))
}

async fn update_settings(
    State(state): State<ApiState>,
    Path(project_id): Path<Uuid>,
    Json(input): Json<UpdateSettings>,
) -> Result<Json<ProjectSettings>, ApiError> {
    Ok(Json(
        state
            .application
            .update_settings(ProjectId(project_id), input)
            .await?,
    ))
}

async fn diagnostics(
    State(state): State<ApiState>,
    Path(project_id): Path<Uuid>,
) -> Result<Json<DiagnosticsSnapshot>, ApiError> {
    Ok(Json(
        state
            .application
            .analyze_project(ProjectId(project_id))
            .await?,
    ))
}

#[derive(Deserialize)]
struct FactCardQuery {
    entity: String,
    #[serde(default)]
    document: Option<Uuid>,
}

async fn fact_card(
    State(state): State<ApiState>,
    Path(project_id): Path<Uuid>,
    Query(query): Query<FactCardQuery>,
) -> Result<Json<FactCard>, ApiError> {
    Ok(Json(
        state
            .application
            .fact_card(
                ProjectId(project_id),
                &query.entity,
                query.document.map(DocumentId),
            )
            .await?,
    ))
}

#[derive(Deserialize)]
struct EntityQuery {
    entity: String,
}

async fn mentions(
    State(state): State<ApiState>,
    Path(project_id): Path<Uuid>,
    Query(query): Query<EntityQuery>,
) -> Result<Json<Vec<MentionRef>>, ApiError> {
    Ok(Json(
        state
            .application
            .mentions(ProjectId(project_id), &query.entity)
            .await?,
    ))
}

#[derive(Deserialize)]
struct FactQuery {
    fact: String,
}

async fn where_settled(
    State(state): State<ApiState>,
    Path(project_id): Path<Uuid>,
    Query(query): Query<FactQuery>,
) -> Result<Json<WhereSettled>, ApiError> {
    Ok(Json(
        state
            .application
            .where_settled(ProjectId(project_id), &query.fact)
            .await?,
    ))
}

async fn scene_summary(
    State(state): State<ApiState>,
    Path((project_id, document_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<SceneSummary>, ApiError> {
    Ok(Json(
        state
            .application
            .scene_summary(ProjectId(project_id), DocumentId(document_id))
            .await?,
    ))
}

#[derive(Deserialize)]
struct SearchQuery {
    q: String,
}

async fn cast_search(
    State(state): State<ApiState>,
    Path(project_id): Path<Uuid>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<Vec<CastSearchHit>>, ApiError> {
    Ok(Json(
        state
            .application
            .cast_search(ProjectId(project_id), &query.q)
            .await?,
    ))
}

async fn apply_repair(
    State(state): State<ApiState>,
    Path(project_id): Path<Uuid>,
    Json(request): Json<ApplyRepairRequest>,
) -> Result<Json<ApplyRepairResponse>, ApiError> {
    Ok(Json(
        state
            .application
            .apply_repair(ProjectId(project_id), request)
            .await?,
    ))
}

async fn bisect(
    State(state): State<ApiState>,
    Path((project_id, diagnostic_id)): Path<(Uuid, String)>,
) -> Result<Json<BisectResult>, ApiError> {
    Ok(Json(
        state
            .application
            .bisect(ProjectId(project_id), &diagnostic_id)
            .await?,
    ))
}

async fn catalog() -> Json<&'static [asten_diagnostics::CodeInfo]> {
    Json(asten_diagnostics::catalog())
}

async fn rename(
    State(state): State<ApiState>,
    Path((project_id, entity_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<RenameRequest>,
) -> Result<Json<RenamePreview>, ApiError> {
    Ok(Json(
        state
            .application
            .rename_entity(ProjectId(project_id), EntityId(entity_id), request)
            .await?,
    ))
}

async fn list_invariants(
    State(state): State<ApiState>,
    Path(project_id): Path<Uuid>,
) -> Result<Json<Vec<StoredInvariant>>, ApiError> {
    Ok(Json(
        state
            .application
            .list_invariants(ProjectId(project_id))
            .await?,
    ))
}

async fn create_invariant(
    State(state): State<ApiState>,
    Path(project_id): Path<Uuid>,
    Json(input): Json<CreateInvariant>,
) -> Result<(StatusCode, Json<StoredInvariant>), ApiError> {
    Ok((
        StatusCode::CREATED,
        Json(
            state
                .application
                .create_invariant(ProjectId(project_id), input)
                .await?,
        ),
    ))
}

async fn update_invariant(
    State(state): State<ApiState>,
    Path((project_id, invariant_id)): Path<(Uuid, Uuid)>,
    Json(invariant): Json<asten_ir::Invariant>,
) -> Result<Json<StoredInvariant>, ApiError> {
    Ok(Json(
        state
            .application
            .update_invariant(ProjectId(project_id), InvariantId(invariant_id), invariant)
            .await?,
    ))
}

async fn delete_invariant(
    State(state): State<ApiState>,
    Path((project_id, invariant_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    state
        .application
        .delete_invariant(ProjectId(project_id), InvariantId(invariant_id))
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn list_routes(
    State(state): State<ApiState>,
    Path(project_id): Path<Uuid>,
) -> Result<Json<Vec<StoredRoute>>, ApiError> {
    Ok(Json(
        state.application.list_routes(ProjectId(project_id)).await?,
    ))
}

async fn create_route(
    State(state): State<ApiState>,
    Path(project_id): Path<Uuid>,
    Json(input): Json<CreateRoute>,
) -> Result<(StatusCode, Json<StoredRoute>), ApiError> {
    Ok((
        StatusCode::CREATED,
        Json(
            state
                .application
                .create_route(ProjectId(project_id), input)
                .await?,
        ),
    ))
}

async fn delete_route(
    State(state): State<ApiState>,
    Path((project_id, route_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    state
        .application
        .delete_route(ProjectId(project_id), RouteId(route_id))
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn list_baselines(
    State(state): State<ApiState>,
    Path(project_id): Path<Uuid>,
) -> Result<Json<Vec<Baseline>>, ApiError> {
    Ok(Json(
        state
            .application
            .list_baselines(ProjectId(project_id))
            .await?,
    ))
}

async fn create_baseline(
    State(state): State<ApiState>,
    Path(project_id): Path<Uuid>,
    Json(input): Json<CreateBaseline>,
) -> Result<(StatusCode, Json<Baseline>), ApiError> {
    Ok((
        StatusCode::CREATED,
        Json(
            state
                .application
                .create_baseline(ProjectId(project_id), input)
                .await?,
        ),
    ))
}

async fn activate_baseline(
    State(state): State<ApiState>,
    Path((project_id, baseline_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<Vec<Baseline>>, ApiError> {
    Ok(Json(
        state
            .application
            .activate_baseline(ProjectId(project_id), Some(BaselineId(baseline_id)))
            .await?,
    ))
}

async fn clear_baseline(
    State(state): State<ApiState>,
    Path(project_id): Path<Uuid>,
) -> Result<Json<Vec<Baseline>>, ApiError> {
    Ok(Json(
        state
            .application
            .activate_baseline(ProjectId(project_id), None)
            .await?,
    ))
}

async fn delete_baseline(
    State(state): State<ApiState>,
    Path((project_id, baseline_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    state
        .application
        .delete_baseline(ProjectId(project_id), BaselineId(baseline_id))
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
