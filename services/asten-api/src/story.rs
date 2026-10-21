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
#[utoipa::path(
    get,
    path = "/projects/{project_id}/stream",
    tag = "story",
    summary = "Subscribe to a project's change stream",
    description = "Server-sent events. The event names are `revision`, `diagnostics`, `document`, `canon` and `heartbeat`, each with a `StreamEvent` as JSON data; `lagged` (data `{}`) says the client fell behind and should refetch; a `ping` comment arrives every twenty seconds. The first event is a `heartbeat`. Authenticate as usual with the bearer token or the `asten_session` cookie, or, from an EventSource that cannot set headers, with a ticket from `POST /projects/{project_id}/stream-ticket` passed as `?ticket=`: it lives one minute and is spent on first use, so nothing worth stealing reaches a proxy log.",
    params(("project_id" = Uuid, Path, description = "The project"), ("ticket" = Option<String>, Query, description = "A stream ticket, instead of the Authorization header or the cookie")),
    responses(
        (status = 200, description = "An event stream that stays open", body = StreamEvent, content_type = "text/event-stream"),
        (status = 401, description = "No valid token, cookie or ticket; a ticket that is spent or expired", body = crate::ErrorResponse),
        (status = 404, description = "No project with that id", body = crate::ErrorResponse)
    ),
    security(("asten_token" = []))
)]
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

#[utoipa::path(
    get,
    path = "/projects/{project_id}/settings",
    tag = "projects",
    summary = "Read project settings",
    description = "Strictness, narrator reliability, the story clock and the lifecycle rules. Defaults are returned for a project that never saved any.",
    params(("project_id" = Uuid, Path, description = "The project")),
    responses(
        (status = 200, description = "The settings", body = ProjectSettings),
        (status = 401, description = "No valid token or session cookie", body = crate::ErrorResponse),
        (status = 404, description = "No project with that id", body = crate::ErrorResponse)
    ),
    security(("asten_token" = []))
)]
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

#[utoipa::path(
    put,
    path = "/projects/{project_id}/settings",
    tag = "projects",
    summary = "Update project settings",
    description = "Only the fields sent change. A fictional calendar is validated.",
    params(("project_id" = Uuid, Path, description = "The project")),
    request_body = UpdateSettings,
    responses(
        (status = 200, description = "The settings", body = ProjectSettings),
        (status = 401, description = "No valid token or session cookie", body = crate::ErrorResponse),
        (status = 404, description = "No project with that id", body = crate::ErrorResponse),
        (status = 422, description = "The calendar is malformed", body = crate::ErrorResponse)
    ),
    security(("asten_token" = []))
)]
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

#[utoipa::path(
    get,
    path = "/projects/{project_id}/story/diagnostics",
    tag = "story",
    summary = "The full diagnostics snapshot",
    description = "`story/diagnostics`: every mark at the current revision, with the obligation ledger, language tiers, stale suppressions and timings.",
    params(("project_id" = Uuid, Path, description = "The project")),
    responses(
        (status = 200, description = "The snapshot", body = DiagnosticsSnapshot),
        (status = 401, description = "No valid token or session cookie", body = crate::ErrorResponse),
        (status = 404, description = "No project with that id", body = crate::ErrorResponse)
    ),
    security(("asten_token" = []))
)]
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

#[derive(Deserialize, utoipa::IntoParams)]
#[into_params(parameter_in = Query)]
struct FactCardQuery {
    /// The entity key, as `CanonEntity.key()` derives it from the name.
    entity: String,
    /// Read the card as of this document's place in the story.
    #[serde(default)]
    document: Option<Uuid>,
}

#[utoipa::path(
    get,
    path = "/projects/{project_id}/story/fact-card",
    tag = "story",
    summary = "An entity's fact card",
    description = "`story/factCard`: what is settled about an entity, optionally as of a document's place in the story.",
    params(("project_id" = Uuid, Path, description = "The project"), FactCardQuery),
    responses(
        (status = 200, description = "The card", body = FactCard),
        (status = 401, description = "No valid token or session cookie", body = crate::ErrorResponse),
        (status = 404, description = "No project, or no entity with that key", body = crate::ErrorResponse)
    ),
    security(("asten_token" = []))
)]
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

#[derive(Deserialize, utoipa::IntoParams)]
#[into_params(parameter_in = Query)]
struct EntityQuery {
    /// The entity key.
    entity: String,
}

#[utoipa::path(
    get,
    path = "/projects/{project_id}/story/mentions",
    tag = "story",
    summary = "Where an entity is mentioned",
    description = "`story/mentions`: every span that names the entity, in reading order.",
    params(("project_id" = Uuid, Path, description = "The project"), EntityQuery),
    responses(
        (status = 200, description = "The mentions", body = Vec<MentionRef>),
        (status = 401, description = "No valid token or session cookie", body = crate::ErrorResponse),
        (status = 404, description = "No project with that id", body = crate::ErrorResponse)
    ),
    security(("asten_token" = []))
)]
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

#[derive(Deserialize, utoipa::IntoParams)]
#[into_params(parameter_in = Query)]
struct FactQuery {
    /// The fact id, as it appears in a fact card or a diagnostic.
    fact: String,
}

#[utoipa::path(
    get,
    path = "/projects/{project_id}/story/where-settled",
    tag = "story",
    summary = "Where a fact was settled",
    description = "`story/whereSettled`: the span, document, actor and time that settled a fact.",
    params(("project_id" = Uuid, Path, description = "The project"), FactQuery),
    responses(
        (status = 200, description = "The settling evidence", body = WhereSettled),
        (status = 401, description = "No valid token or session cookie", body = crate::ErrorResponse),
        (status = 404, description = "No project, or no fact with that id", body = crate::ErrorResponse)
    ),
    security(("asten_token" = []))
)]
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

#[utoipa::path(
    get,
    path = "/projects/{project_id}/story/scene-summary/{document_id}",
    tag = "story",
    summary = "Summarise a scene",
    description = "`story/sceneSummary`: the scene's place in story time, its cast, counts of events, observations and marks, and its script tier.",
    params(("project_id" = Uuid, Path, description = "The project"), ("document_id" = Uuid, Path, description = "The document")),
    responses(
        (status = 200, description = "The summary", body = SceneSummary),
        (status = 401, description = "No valid token or session cookie", body = crate::ErrorResponse),
        (status = 404, description = "No project, or no prose document with that id", body = crate::ErrorResponse)
    ),
    security(("asten_token" = []))
)]
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

#[derive(Deserialize, utoipa::IntoParams)]
#[into_params(parameter_in = Query)]
struct SearchQuery {
    /// Part of a name or alias.
    q: String,
}

#[utoipa::path(
    get,
    path = "/projects/{project_id}/story/cast-search",
    tag = "story",
    summary = "Search the cast",
    description = "`story/castSearch`: entities whose name or alias matches, scored.",
    params(("project_id" = Uuid, Path, description = "The project"), SearchQuery),
    responses(
        (status = 200, description = "The hits, best first", body = Vec<CastSearchHit>),
        (status = 401, description = "No valid token or session cookie", body = crate::ErrorResponse),
        (status = 404, description = "No project with that id", body = crate::ErrorResponse)
    ),
    security(("asten_token" = []))
)]
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

#[utoipa::path(
    post,
    path = "/projects/{project_id}/story/repairs/apply",
    tag = "story",
    summary = "Apply a repair",
    description = "Applies one of the repairs a diagnostic offers, by index: a change event, a context mark, a validity window, a scene placement, a suppression, and so on. The canon changes; the next snapshot shows the result.",
    params(("project_id" = Uuid, Path, description = "The project")),
    request_body = ApplyRepairRequest,
    responses(
        (status = 200, description = "What was applied", body = ApplyRepairResponse),
        (status = 401, description = "No valid token or session cookie", body = crate::ErrorResponse),
        (status = 404, description = "No project, or the diagnostic is gone from the current snapshot", body = crate::ErrorResponse),
        (status = 422, description = "No repair at that index, or it cannot be applied", body = crate::ErrorResponse)
    ),
    security(("asten_token" = []))
)]
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

#[utoipa::path(
    get,
    path = "/projects/{project_id}/story/bisect/{diagnostic_id}",
    tag = "story",
    summary = "Find the edit that introduced a mark",
    description = "Walks the document history to the first event after which the diagnostic appears.",
    params(("project_id" = Uuid, Path, description = "The project"), ("diagnostic_id" = String, Path, description = "The diagnostic id, as the snapshot lists it")),
    responses(
        (status = 200, description = "The culprit event, if found", body = BisectResult),
        (status = 401, description = "No valid token or session cookie", body = crate::ErrorResponse),
        (status = 404, description = "No project, or no diagnostic with that id", body = crate::ErrorResponse)
    ),
    security(("asten_token" = []))
)]
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

#[utoipa::path(
    get,
    path = "/projects/{project_id}/story/catalog",
    tag = "story",
    summary = "The diagnostic code catalogue",
    description = "Static: the code, area, title, summary and pack of every rule the engine can raise. The same for every project.",
    params(("project_id" = Uuid, Path, description = "The project")),
    responses(
        (status = 200, description = "The catalogue", body = Vec<asten_diagnostics::CodeInfo>),
        (status = 401, description = "No valid token or session cookie", body = crate::ErrorResponse)
    ),
    security(("asten_token" = []))
)]
async fn catalog() -> Json<&'static [asten_diagnostics::CodeInfo]> {
    Json(asten_diagnostics::catalog())
}

#[utoipa::path(
    post,
    path = "/projects/{project_id}/entities/{entity_id}/rename",
    tag = "story",
    summary = "Rename an entity",
    description = "Previews every mention that would change, as before/after spans; set `apply` to write them and the new name.",
    params(("project_id" = Uuid, Path, description = "The project"), ("entity_id" = Uuid, Path, description = "The canon entity")),
    request_body = RenameRequest,
    responses(
        (status = 200, description = "The preview, or what was changed", body = RenamePreview),
        (status = 401, description = "No valid token or session cookie", body = crate::ErrorResponse),
        (status = 404, description = "No project, or no entity with that id", body = crate::ErrorResponse),
        (status = 422, description = "The new name is empty", body = crate::ErrorResponse)
    ),
    security(("asten_token" = []))
)]
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

#[utoipa::path(
    get,
    path = "/projects/{project_id}/invariants",
    tag = "canon",
    summary = "List invariants",
    description = "The author's rules over the story, as LTLf patterns.",
    params(("project_id" = Uuid, Path, description = "The project")),
    responses(
        (status = 200, description = "The invariants", body = Vec<StoredInvariant>),
        (status = 401, description = "No valid token or session cookie", body = crate::ErrorResponse),
        (status = 404, description = "No project with that id", body = crate::ErrorResponse)
    ),
    security(("asten_token" = []))
)]
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

#[utoipa::path(
    post,
    path = "/projects/{project_id}/invariants",
    tag = "canon",
    summary = "Create an invariant",
    params(("project_id" = Uuid, Path, description = "The project")),
    request_body = CreateInvariant,
    responses(
        (status = 201, description = "The stored invariant", body = StoredInvariant),
        (status = 401, description = "No valid token or session cookie", body = crate::ErrorResponse),
        (status = 404, description = "No project with that id", body = crate::ErrorResponse),
        (status = 422, description = "The label is empty or the pattern is malformed", body = crate::ErrorResponse)
    ),
    security(("asten_token" = []))
)]
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

#[utoipa::path(
    put,
    path = "/projects/{project_id}/invariants/{invariant_id}",
    tag = "canon",
    summary = "Replace an invariant",
    params(("project_id" = Uuid, Path, description = "The project"), ("invariant_id" = Uuid, Path, description = "The invariant")),
    request_body = asten_ir::Invariant,
    responses(
        (status = 200, description = "The stored invariant", body = StoredInvariant),
        (status = 401, description = "No valid token or session cookie", body = crate::ErrorResponse),
        (status = 404, description = "No invariant with that id in this project", body = crate::ErrorResponse),
        (status = 422, description = "The pattern is malformed", body = crate::ErrorResponse)
    ),
    security(("asten_token" = []))
)]
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

#[utoipa::path(
    delete,
    path = "/projects/{project_id}/invariants/{invariant_id}",
    tag = "canon",
    summary = "Delete an invariant",
    params(("project_id" = Uuid, Path, description = "The project"), ("invariant_id" = Uuid, Path, description = "The invariant")),
    responses(
        (status = 204, description = "Deleted"),
        (status = 401, description = "No valid token or session cookie", body = crate::ErrorResponse),
        (status = 404, description = "No project with that id", body = crate::ErrorResponse)
    ),
    security(("asten_token" = []))
)]
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

#[utoipa::path(
    get,
    path = "/projects/{project_id}/routes",
    tag = "canon",
    summary = "List travel routes",
    description = "How long it takes to get from one place to another, for the journey checks.",
    params(("project_id" = Uuid, Path, description = "The project")),
    responses(
        (status = 200, description = "The routes", body = Vec<StoredRoute>),
        (status = 401, description = "No valid token or session cookie", body = crate::ErrorResponse),
        (status = 404, description = "No project with that id", body = crate::ErrorResponse)
    ),
    security(("asten_token" = []))
)]
async fn list_routes(
    State(state): State<ApiState>,
    Path(project_id): Path<Uuid>,
) -> Result<Json<Vec<StoredRoute>>, ApiError> {
    Ok(Json(
        state.application.list_routes(ProjectId(project_id)).await?,
    ))
}

#[utoipa::path(
    post,
    path = "/projects/{project_id}/routes",
    tag = "canon",
    summary = "Create a travel route",
    params(("project_id" = Uuid, Path, description = "The project")),
    request_body = CreateRoute,
    responses(
        (status = 201, description = "The stored route", body = StoredRoute),
        (status = 401, description = "No valid token or session cookie", body = crate::ErrorResponse),
        (status = 404, description = "No project with that id", body = crate::ErrorResponse),
        (status = 422, description = "The route joins a place to itself or its bounds are inverted", body = crate::ErrorResponse)
    ),
    security(("asten_token" = []))
)]
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

#[utoipa::path(
    delete,
    path = "/projects/{project_id}/routes/{route_id}",
    tag = "canon",
    summary = "Delete a travel route",
    params(("project_id" = Uuid, Path, description = "The project"), ("route_id" = Uuid, Path, description = "The travel route")),
    responses(
        (status = 204, description = "Deleted"),
        (status = 401, description = "No valid token or session cookie", body = crate::ErrorResponse),
        (status = 404, description = "No project with that id", body = crate::ErrorResponse)
    ),
    security(("asten_token" = []))
)]
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

#[utoipa::path(
    get,
    path = "/projects/{project_id}/baselines",
    tag = "story",
    summary = "List baselines",
    description = "Frozen sets of accepted marks. While one is active only marks new since it are highlighted.",
    params(("project_id" = Uuid, Path, description = "The project")),
    responses(
        (status = 200, description = "The baselines", body = Vec<Baseline>),
        (status = 401, description = "No valid token or session cookie", body = crate::ErrorResponse),
        (status = 404, description = "No project with that id", body = crate::ErrorResponse)
    ),
    security(("asten_token" = []))
)]
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

#[utoipa::path(
    post,
    path = "/projects/{project_id}/baselines",
    tag = "story",
    summary = "Freeze the current marks as a baseline",
    description = "Records the current revision and diagnostic ids under a name and makes the baseline active.",
    params(("project_id" = Uuid, Path, description = "The project")),
    request_body = CreateBaseline,
    responses(
        (status = 201, description = "The baseline", body = Baseline),
        (status = 401, description = "No valid token or session cookie", body = crate::ErrorResponse),
        (status = 404, description = "No project with that id", body = crate::ErrorResponse),
        (status = 422, description = "The name is empty", body = crate::ErrorResponse)
    ),
    security(("asten_token" = []))
)]
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

#[utoipa::path(
    post,
    path = "/projects/{project_id}/baselines/{baseline_id}/activate",
    tag = "story",
    summary = "Activate a baseline",
    params(("project_id" = Uuid, Path, description = "The project"), ("baseline_id" = Uuid, Path, description = "The baseline")),
    responses(
        (status = 200, description = "Every baseline, with the active flag updated", body = Vec<Baseline>),
        (status = 401, description = "No valid token or session cookie", body = crate::ErrorResponse),
        (status = 404, description = "No baseline with that id in this project", body = crate::ErrorResponse)
    ),
    security(("asten_token" = []))
)]
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

#[utoipa::path(
    post,
    path = "/projects/{project_id}/baselines/clear",
    tag = "story",
    summary = "Deactivate the baseline",
    description = "Every mark is highlighted again.",
    params(("project_id" = Uuid, Path, description = "The project")),
    responses(
        (status = 200, description = "Every baseline, none active", body = Vec<Baseline>),
        (status = 401, description = "No valid token or session cookie", body = crate::ErrorResponse),
        (status = 404, description = "No project with that id", body = crate::ErrorResponse)
    ),
    security(("asten_token" = []))
)]
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

#[utoipa::path(
    delete,
    path = "/projects/{project_id}/baselines/{baseline_id}",
    tag = "story",
    summary = "Delete a baseline",
    params(("project_id" = Uuid, Path, description = "The project"), ("baseline_id" = Uuid, Path, description = "The baseline")),
    responses(
        (status = 204, description = "Deleted"),
        (status = 401, description = "No valid token or session cookie", body = crate::ErrorResponse),
        (status = 404, description = "No project with that id", body = crate::ErrorResponse)
    ),
    security(("asten_token" = []))
)]
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
