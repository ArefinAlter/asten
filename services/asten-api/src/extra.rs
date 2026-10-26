use asten_application::{
    CompileRequest, CreateCanonEntity, CreateComment, CreateSnapshot, CreateSuppression,
    CreateTimelineEvent, CreateTrackedChange, ImportRequest, ResolveTrackedChange,
    UpdateCanonEntity, UpdateComment, UpdateMethodPack, UpdateProject, UpdateTimelineEvent,
};
use asten_domain::{
    CanonEntity, Comment, CommentId, CompileArtifact, Document, DocumentId, DocumentOp,
    DocumentSnapshot, EntityId, MethodPack, MethodPackId, Project, ProjectId, SnapshotId,
    Suppression, TimelineEvent, TimelineEventId, TrackedChange, TrackedChangeId,
};
use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::get,
};
use uuid::Uuid;

use crate::{ApiError, ApiState};

pub fn router() -> Router<ApiState> {
    Router::new()
        .route(
            "/projects/{project_id}/entities",
            get(list_entities).post(create_entity),
        )
        .route(
            "/projects/{project_id}/entities/{entity_id}",
            axum::routing::put(update_entity).delete(delete_entity),
        )
        .route(
            "/projects/{project_id}/entities/{entity_id}/merge-into/{target_id}",
            axum::routing::post(merge_entity),
        )
        .route("/projects/{project_id}/proposals", get(list_proposals))
        .route(
            "/projects/{project_id}/events",
            get(list_events).post(create_event),
        )
        .route(
            "/projects/{project_id}/events/{event_id}",
            axum::routing::put(update_event).delete(delete_event),
        )
        .route(
            "/projects/{project_id}/documents/{document_id}/comments",
            get(list_comments).post(create_comment),
        )
        .route(
            "/projects/{project_id}/comments/{comment_id}",
            axum::routing::put(update_comment),
        )
        .route(
            "/projects/{project_id}/documents/{document_id}/ops",
            get(list_ops),
        )
        .route(
            "/projects/{project_id}/documents/{document_id}/snapshots",
            get(list_snapshots).post(create_snapshot),
        )
        .route(
            "/projects/{project_id}/snapshots/{snapshot_id}/restore",
            axum::routing::post(restore_snapshot),
        )
        .route(
            "/projects/{project_id}/suppressions",
            get(list_suppressions).post(create_suppression),
        )
        .route(
            "/projects/{project_id}/suppressions/{suppression_id}",
            axum::routing::delete(delete_suppression),
        )
        .route("/packs", get(list_packs))
        .route("/packs/{pack_id}", axum::routing::put(update_pack))
        .route(
            "/projects/{project_id}/documents/{document_id}/changes",
            get(list_changes).post(create_change),
        )
        .route(
            "/projects/{project_id}/changes/{change_id}/resolve",
            axum::routing::post(resolve_change),
        )
        .route(
            "/projects/{project_id}/compile",
            axum::routing::post(compile_project),
        )
        .route(
            "/projects/{project_id}/import",
            axum::routing::post(import_document),
        )
        .route("/projects/{project_id}/export", get(export_project))
        .route("/projects/bundle", axum::routing::post(import_project_file))
}

#[utoipa::path(
    put,
    path = "/projects/{project_id}",
    tag = "projects",
    summary = "Rename a project",
    params(("project_id" = Uuid, Path, description = "The project")),
    request_body = UpdateProject,
    responses(
        (status = 200, description = "The project", body = Project),
        (status = 401, description = "No valid token or session cookie", body = crate::ErrorResponse),
        (status = 404, description = "No project with that id", body = crate::ErrorResponse),
        (status = 422, description = "The name is empty", body = crate::ErrorResponse)
    ),
    security(("asten_token" = []))
)]
pub(crate) async fn update_project(
    State(state): State<ApiState>,
    Path(project_id): Path<Uuid>,
    Json(input): Json<UpdateProject>,
) -> Result<Json<Project>, ApiError> {
    Ok(Json(
        state
            .application
            .update_project(ProjectId(project_id), input)
            .await?,
    ))
}

#[utoipa::path(
    delete,
    path = "/projects/{project_id}",
    tag = "projects",
    summary = "Delete a project",
    description = "Removes the project and everything in it. This cannot be undone; the library's backups still hold the last copy.",
    params(("project_id" = Uuid, Path, description = "The project")),
    responses(
        (status = 204, description = "Deleted"),
        (status = 401, description = "No valid token or session cookie", body = crate::ErrorResponse),
        (status = 404, description = "No project with that id", body = crate::ErrorResponse)
    ),
    security(("asten_token" = []))
)]
pub(crate) async fn delete_project(
    State(state): State<ApiState>,
    Path(project_id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    state
        .application
        .delete_project(ProjectId(project_id))
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/projects/{project_id}/export",
    tag = "projects",
    summary = "Export a project file",
    description = "The project as one JSON document (format asten-project): documents with their content, cast, facts, timeline, world board, routes, rules, settings and comments. Version history is not included; a backup of the library keeps it.",
    params(("project_id" = Uuid, Path, description = "The project")),
    responses(
        (status = 200, description = "The project file", body = serde_json::Value),
        (status = 401, description = "No valid token or session cookie", body = crate::ErrorResponse),
        (status = 404, description = "No project with that id", body = crate::ErrorResponse)
    ),
    security(("asten_token" = []))
)]
pub(crate) async fn export_project(
    State(state): State<ApiState>,
    Path(project_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    Ok(Json(
        state
            .application
            .export_project(ProjectId(project_id))
            .await?,
    ))
}

#[utoipa::path(
    post,
    path = "/projects/bundle",
    tag = "projects",
    summary = "Open a project file",
    description = "Creates a new project from a project file written by the export route. Ids are minted again and every reference rewritten; a file with a record this version cannot read is refused whole.",
    request_body = serde_json::Value,
    responses(
        (status = 201, description = "The new project", body = Project),
        (status = 401, description = "No valid token or session cookie", body = crate::ErrorResponse),
        (status = 422, description = "This is not a project file, or one that this version cannot read", body = crate::ErrorResponse)
    ),
    security(("asten_token" = []))
)]
pub(crate) async fn import_project_file(
    State(state): State<ApiState>,
    Json(bundle): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<Project>), ApiError> {
    let project = state.application.import_project_bundle(bundle).await?;
    Ok((StatusCode::CREATED, Json(project)))
}

#[utoipa::path(
    get,
    path = "/projects/{project_id}/entities",
    tag = "canon",
    summary = "List canon entities",
    params(("project_id" = Uuid, Path, description = "The project")),
    responses(
        (status = 200, description = "The entities", body = Vec<CanonEntity>),
        (status = 401, description = "No valid token or session cookie", body = crate::ErrorResponse),
        (status = 404, description = "No project with that id", body = crate::ErrorResponse)
    ),
    security(("asten_token" = []))
)]
async fn list_entities(
    State(state): State<ApiState>,
    Path(project_id): Path<Uuid>,
) -> Result<Json<Vec<CanonEntity>>, ApiError> {
    Ok(Json(
        state
            .application
            .list_entities(ProjectId(project_id))
            .await?,
    ))
}

#[utoipa::path(
    post,
    path = "/projects/{project_id}/entities",
    tag = "canon",
    summary = "Create a canon entity",
    description = "The `project_id` in the body is replaced by the one in the path.",
    params(("project_id" = Uuid, Path, description = "The project")),
    request_body = CreateCanonEntity,
    responses(
        (status = 201, description = "The new entity", body = CanonEntity),
        (status = 401, description = "No valid token or session cookie", body = crate::ErrorResponse),
        (status = 404, description = "No project with that id", body = crate::ErrorResponse),
        (status = 422, description = "The name is empty", body = crate::ErrorResponse)
    ),
    security(("asten_token" = []))
)]
async fn create_entity(
    State(state): State<ApiState>,
    Path(project_id): Path<Uuid>,
    Json(mut input): Json<CreateCanonEntity>,
) -> Result<(StatusCode, Json<CanonEntity>), ApiError> {
    input.project_id = ProjectId(project_id);
    let entity = state.application.create_entity(input).await?;
    Ok((StatusCode::CREATED, Json(entity)))
}

#[utoipa::path(
    put,
    path = "/projects/{project_id}/entities/{entity_id}",
    tag = "canon",
    summary = "Update a canon entity",
    params(("project_id" = Uuid, Path, description = "The project"), ("entity_id" = Uuid, Path, description = "The canon entity")),
    request_body = UpdateCanonEntity,
    responses(
        (status = 200, description = "The entity", body = CanonEntity),
        (status = 401, description = "No valid token or session cookie", body = crate::ErrorResponse),
        (status = 404, description = "No entity with that id", body = crate::ErrorResponse),
        (status = 422, description = "The name is empty", body = crate::ErrorResponse)
    ),
    security(("asten_token" = []))
)]
async fn update_entity(
    State(state): State<ApiState>,
    Path((_project_id, entity_id)): Path<(Uuid, Uuid)>,
    Json(input): Json<UpdateCanonEntity>,
) -> Result<Json<CanonEntity>, ApiError> {
    Ok(Json(
        state
            .application
            .update_entity(EntityId(entity_id), input)
            .await?,
    ))
}

#[utoipa::path(
    delete,
    path = "/projects/{project_id}/entities/{entity_id}",
    tag = "canon",
    summary = "Delete a canon entity",
    params(("project_id" = Uuid, Path, description = "The project"), ("entity_id" = Uuid, Path, description = "The canon entity")),
    responses(
        (status = 204, description = "Deleted"),
        (status = 401, description = "No valid token or session cookie", body = crate::ErrorResponse),
        (status = 404, description = "No entity with that id", body = crate::ErrorResponse)
    ),
    security(("asten_token" = []))
)]
async fn delete_entity(
    State(state): State<ApiState>,
    Path((_project_id, entity_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    state.application.delete_entity(EntityId(entity_id)).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/projects/{project_id}/entities/{entity_id}/merge-into/{target_id}",
    tag = "canon",
    summary = "Merge one entity into another",
    description = "Moves the facts, events and aliases of `entity_id` onto `target_id` and deletes the source.",
    params(("project_id" = Uuid, Path, description = "The project"), ("entity_id" = Uuid, Path, description = "The canon entity"), ("target_id" = Uuid, Path, description = "The entity that survives the merge")),
    responses(
        (status = 200, description = "The surviving entity", body = CanonEntity),
        (status = 401, description = "No valid token or session cookie", body = crate::ErrorResponse),
        (status = 404, description = "No entity with one of those ids", body = crate::ErrorResponse),
        (status = 422, description = "An entity cannot be merged into itself", body = crate::ErrorResponse)
    ),
    security(("asten_token" = []))
)]
async fn merge_entity(
    State(state): State<ApiState>,
    Path((_project_id, entity_id, target_id)): Path<(Uuid, Uuid, Uuid)>,
) -> Result<Json<CanonEntity>, ApiError> {
    Ok(Json(
        state
            .application
            .merge_entities(EntityId(target_id), EntityId(entity_id))
            .await?,
    ))
}

#[utoipa::path(
    get,
    path = "/projects/{project_id}/proposals",
    tag = "canon",
    summary = "List proposed facts",
    description = "Facts the engine inferred from the text that the author has not confirmed or rejected.",
    params(("project_id" = Uuid, Path, description = "The project")),
    responses(
        (status = 200, description = "The proposals", body = Vec<asten_domain::Fact>),
        (status = 401, description = "No valid token or session cookie", body = crate::ErrorResponse),
        (status = 404, description = "No project with that id", body = crate::ErrorResponse)
    ),
    security(("asten_token" = []))
)]
async fn list_proposals(
    State(state): State<ApiState>,
    Path(project_id): Path<Uuid>,
) -> Result<Json<Vec<asten_domain::Fact>>, ApiError> {
    Ok(Json(
        state.application.proposals(ProjectId(project_id)).await?,
    ))
}

#[utoipa::path(
    delete,
    path = "/projects/{project_id}/events/{event_id}",
    tag = "canon",
    summary = "Delete a timeline event",
    params(("project_id" = Uuid, Path, description = "The project"), ("event_id" = Uuid, Path, description = "The timeline event")),
    responses(
        (status = 204, description = "Deleted"),
        (status = 401, description = "No valid token or session cookie", body = crate::ErrorResponse),
        (status = 404, description = "No event with that id", body = crate::ErrorResponse)
    ),
    security(("asten_token" = []))
)]
async fn delete_event(
    State(state): State<ApiState>,
    Path((_project_id, event_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    state
        .application
        .delete_event(TimelineEventId(event_id))
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    delete,
    path = "/projects/{project_id}/suppressions/{suppression_id}",
    tag = "story",
    summary = "Lift a suppression",
    params(("project_id" = Uuid, Path, description = "The project"), ("suppression_id" = Uuid, Path, description = "The suppression")),
    responses(
        (status = 204, description = "Deleted"),
        (status = 401, description = "No valid token or session cookie", body = crate::ErrorResponse),
        (status = 404, description = "No suppression with that id", body = crate::ErrorResponse)
    ),
    security(("asten_token" = []))
)]
async fn delete_suppression(
    State(state): State<ApiState>,
    Path((_project_id, suppression_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    state
        .application
        .delete_suppression(asten_domain::SuppressionId(suppression_id))
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/projects/{project_id}/events",
    tag = "canon",
    summary = "List timeline events",
    params(("project_id" = Uuid, Path, description = "The project")),
    responses(
        (status = 200, description = "The events", body = Vec<TimelineEvent>),
        (status = 401, description = "No valid token or session cookie", body = crate::ErrorResponse),
        (status = 404, description = "No project with that id", body = crate::ErrorResponse)
    ),
    security(("asten_token" = []))
)]
async fn list_events(
    State(state): State<ApiState>,
    Path(project_id): Path<Uuid>,
) -> Result<Json<Vec<TimelineEvent>>, ApiError> {
    Ok(Json(
        state.application.list_events(ProjectId(project_id)).await?,
    ))
}

#[utoipa::path(
    post,
    path = "/projects/{project_id}/events",
    tag = "canon",
    summary = "Create a timeline event",
    description = "The `project_id` in the body is replaced by the one in the path.",
    params(("project_id" = Uuid, Path, description = "The project")),
    request_body = CreateTimelineEvent,
    responses(
        (status = 201, description = "The new event", body = TimelineEvent),
        (status = 401, description = "No valid token or session cookie", body = crate::ErrorResponse),
        (status = 404, description = "No project with that id", body = crate::ErrorResponse),
        (status = 422, description = "The title is empty", body = crate::ErrorResponse)
    ),
    security(("asten_token" = []))
)]
async fn create_event(
    State(state): State<ApiState>,
    Path(project_id): Path<Uuid>,
    Json(mut input): Json<CreateTimelineEvent>,
) -> Result<(StatusCode, Json<TimelineEvent>), ApiError> {
    input.project_id = ProjectId(project_id);
    let event = state.application.create_event(input).await?;
    Ok((StatusCode::CREATED, Json(event)))
}

#[utoipa::path(
    put,
    path = "/projects/{project_id}/events/{event_id}",
    tag = "canon",
    summary = "Update a timeline event",
    params(("project_id" = Uuid, Path, description = "The project"), ("event_id" = Uuid, Path, description = "The timeline event")),
    request_body = UpdateTimelineEvent,
    responses(
        (status = 200, description = "The event", body = TimelineEvent),
        (status = 401, description = "No valid token or session cookie", body = crate::ErrorResponse),
        (status = 404, description = "No event with that id", body = crate::ErrorResponse),
        (status = 422, description = "The title is empty", body = crate::ErrorResponse)
    ),
    security(("asten_token" = []))
)]
async fn update_event(
    State(state): State<ApiState>,
    Path((_project_id, event_id)): Path<(Uuid, Uuid)>,
    Json(input): Json<UpdateTimelineEvent>,
) -> Result<Json<TimelineEvent>, ApiError> {
    Ok(Json(
        state
            .application
            .update_event(TimelineEventId(event_id), input)
            .await?,
    ))
}

#[utoipa::path(
    get,
    path = "/projects/{project_id}/documents/{document_id}/comments",
    tag = "documents",
    summary = "List comments",
    params(("project_id" = Uuid, Path, description = "The project"), ("document_id" = Uuid, Path, description = "The document")),
    responses(
        (status = 200, description = "The comments", body = Vec<Comment>),
        (status = 401, description = "No valid token or session cookie", body = crate::ErrorResponse),
        (status = 404, description = "No document with that id", body = crate::ErrorResponse)
    ),
    security(("asten_token" = []))
)]
async fn list_comments(
    State(state): State<ApiState>,
    Path((_project_id, document_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<Vec<Comment>>, ApiError> {
    Ok(Json(
        state
            .application
            .list_comments(DocumentId(document_id))
            .await?,
    ))
}

#[utoipa::path(
    post,
    path = "/projects/{project_id}/documents/{document_id}/comments",
    tag = "documents",
    summary = "Add a comment",
    description = "The `document_id` in the body is replaced by the one in the path.",
    params(("project_id" = Uuid, Path, description = "The project"), ("document_id" = Uuid, Path, description = "The document")),
    request_body = CreateComment,
    responses(
        (status = 201, description = "The new comment", body = Comment),
        (status = 401, description = "No valid token or session cookie", body = crate::ErrorResponse),
        (status = 404, description = "No document with that id", body = crate::ErrorResponse),
        (status = 422, description = "The body is empty", body = crate::ErrorResponse)
    ),
    security(("asten_token" = []))
)]
async fn create_comment(
    State(state): State<ApiState>,
    Path((_project_id, document_id)): Path<(Uuid, Uuid)>,
    Json(mut input): Json<CreateComment>,
) -> Result<(StatusCode, Json<Comment>), ApiError> {
    input.document_id = DocumentId(document_id);
    let comment = state.application.create_comment(input).await?;
    Ok((StatusCode::CREATED, Json(comment)))
}

#[utoipa::path(
    put,
    path = "/projects/{project_id}/comments/{comment_id}",
    tag = "documents",
    summary = "Edit or resolve a comment",
    params(("project_id" = Uuid, Path, description = "The project"), ("comment_id" = Uuid, Path, description = "The comment")),
    request_body = UpdateComment,
    responses(
        (status = 200, description = "The comment", body = Comment),
        (status = 401, description = "No valid token or session cookie", body = crate::ErrorResponse),
        (status = 404, description = "No comment with that id", body = crate::ErrorResponse),
        (status = 422, description = "The body is empty", body = crate::ErrorResponse)
    ),
    security(("asten_token" = []))
)]
async fn update_comment(
    State(state): State<ApiState>,
    Path((_project_id, comment_id)): Path<(Uuid, Uuid)>,
    Json(input): Json<UpdateComment>,
) -> Result<Json<Comment>, ApiError> {
    Ok(Json(
        state
            .application
            .update_comment(CommentId(comment_id), input)
            .await?,
    ))
}

#[utoipa::path(
    get,
    path = "/projects/{project_id}/documents/{document_id}/ops",
    tag = "history",
    summary = "Legacy document operations",
    description = "Coarse operations kept for older history views. New history is the block event log at `…/history`.",
    params(("project_id" = Uuid, Path, description = "The project"), ("document_id" = Uuid, Path, description = "The document")),
    responses(
        (status = 200, description = "The operations", body = Vec<DocumentOp>),
        (status = 401, description = "No valid token or session cookie", body = crate::ErrorResponse),
        (status = 404, description = "No document with that id", body = crate::ErrorResponse)
    ),
    security(("asten_token" = []))
)]
async fn list_ops(
    State(state): State<ApiState>,
    Path((_project_id, document_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<Vec<DocumentOp>>, ApiError> {
    Ok(Json(
        state.application.list_ops(DocumentId(document_id)).await?,
    ))
}

#[utoipa::path(
    get,
    path = "/projects/{project_id}/documents/{document_id}/snapshots",
    tag = "history",
    summary = "List snapshots",
    params(("project_id" = Uuid, Path, description = "The project"), ("document_id" = Uuid, Path, description = "The document")),
    responses(
        (status = 200, description = "The snapshots, newest first", body = Vec<DocumentSnapshot>),
        (status = 401, description = "No valid token or session cookie", body = crate::ErrorResponse),
        (status = 404, description = "No document with that id", body = crate::ErrorResponse)
    ),
    security(("asten_token" = []))
)]
async fn list_snapshots(
    State(state): State<ApiState>,
    Path((_project_id, document_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<Vec<DocumentSnapshot>>, ApiError> {
    Ok(Json(
        state
            .application
            .list_snapshots(DocumentId(document_id))
            .await?,
    ))
}

#[utoipa::path(
    post,
    path = "/projects/{project_id}/documents/{document_id}/snapshots",
    tag = "history",
    summary = "Take a snapshot",
    description = "Stores the current content with the event sequence and chain hash it commits to. The `document_id` in the body is replaced by the one in the path.",
    params(("project_id" = Uuid, Path, description = "The project"), ("document_id" = Uuid, Path, description = "The document")),
    request_body = CreateSnapshot,
    responses(
        (status = 201, description = "The snapshot", body = DocumentSnapshot),
        (status = 401, description = "No valid token or session cookie", body = crate::ErrorResponse),
        (status = 404, description = "No document with that id", body = crate::ErrorResponse)
    ),
    security(("asten_token" = []))
)]
async fn create_snapshot(
    State(state): State<ApiState>,
    Path((_project_id, document_id)): Path<(Uuid, Uuid)>,
    Json(mut input): Json<CreateSnapshot>,
) -> Result<(StatusCode, Json<DocumentSnapshot>), ApiError> {
    input.document_id = DocumentId(document_id);
    let snapshot = state.application.create_snapshot(input).await?;
    Ok((StatusCode::CREATED, Json(snapshot)))
}

#[utoipa::path(
    post,
    path = "/projects/{project_id}/snapshots/{snapshot_id}/restore",
    tag = "history",
    summary = "Restore a snapshot",
    description = "Commits the snapshot's content as a new revision; nothing is rewritten.",
    params(("project_id" = Uuid, Path, description = "The project"), ("snapshot_id" = Uuid, Path, description = "The snapshot")),
    responses(
        (status = 200, description = "The document after the restore", body = Document),
        (status = 401, description = "No valid token or session cookie", body = crate::ErrorResponse),
        (status = 404, description = "No snapshot with that id", body = crate::ErrorResponse)
    ),
    security(("asten_token" = []))
)]
async fn restore_snapshot(
    State(state): State<ApiState>,
    Path((_project_id, snapshot_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<Document>, ApiError> {
    Ok(Json(
        state
            .application
            .restore_snapshot(SnapshotId(snapshot_id))
            .await?,
    ))
}

#[utoipa::path(
    get,
    path = "/projects/{project_id}/suppressions",
    tag = "story",
    summary = "List suppressions",
    params(("project_id" = Uuid, Path, description = "The project")),
    responses(
        (status = 200, description = "The suppressions", body = Vec<Suppression>),
        (status = 401, description = "No valid token or session cookie", body = crate::ErrorResponse),
        (status = 404, description = "No project with that id", body = crate::ErrorResponse)
    ),
    security(("asten_token" = []))
)]
async fn list_suppressions(
    State(state): State<ApiState>,
    Path(project_id): Path<Uuid>,
) -> Result<Json<Vec<Suppression>>, ApiError> {
    Ok(Json(
        state
            .application
            .list_suppressions(ProjectId(project_id))
            .await?,
    ))
}

#[utoipa::path(
    post,
    path = "/projects/{project_id}/suppressions",
    tag = "story",
    summary = "Silence a rule",
    description = "Scoped to one occurrence, an entity, a document or the project. An occurrence suppression is bound to its evidence fingerprint and goes stale when the text moves on. The `project_id` in the body is replaced by the one in the path.",
    params(("project_id" = Uuid, Path, description = "The project")),
    request_body = CreateSuppression,
    responses(
        (status = 201, description = "The suppression", body = Suppression),
        (status = 401, description = "No valid token or session cookie", body = crate::ErrorResponse),
        (status = 404, description = "No project with that id", body = crate::ErrorResponse),
        (status = 422, description = "The rule code is empty", body = crate::ErrorResponse)
    ),
    security(("asten_token" = []))
)]
async fn create_suppression(
    State(state): State<ApiState>,
    Path(project_id): Path<Uuid>,
    Json(mut input): Json<CreateSuppression>,
) -> Result<(StatusCode, Json<Suppression>), ApiError> {
    input.project_id = ProjectId(project_id);
    let suppression = state.application.create_suppression(input).await?;
    Ok((StatusCode::CREATED, Json(suppression)))
}

#[utoipa::path(
    get,
    path = "/packs",
    tag = "packs",
    summary = "List method packs",
    description = "The installed packs of predicates, cues and rules, with their manifests.",
    responses(
        (status = 200, description = "The packs", body = Vec<MethodPack>),
        (status = 401, description = "No valid token or session cookie", body = crate::ErrorResponse)
    ),
    security(("asten_token" = []))
)]
async fn list_packs(State(state): State<ApiState>) -> Result<Json<Vec<MethodPack>>, ApiError> {
    Ok(Json(state.application.list_packs().await?))
}

#[utoipa::path(
    put,
    path = "/packs/{pack_id}",
    tag = "packs",
    summary = "Enable or disable a method pack",
    params(("pack_id" = Uuid, Path, description = "The method pack")),
    request_body = UpdateMethodPack,
    responses(
        (status = 200, description = "The pack", body = MethodPack),
        (status = 401, description = "No valid token or session cookie", body = crate::ErrorResponse),
        (status = 404, description = "No pack with that id", body = crate::ErrorResponse),
        (status = 422, description = "The core pack cannot be disabled", body = crate::ErrorResponse)
    ),
    security(("asten_token" = []))
)]
async fn update_pack(
    State(state): State<ApiState>,
    Path(pack_id): Path<Uuid>,
    Json(input): Json<UpdateMethodPack>,
) -> Result<Json<MethodPack>, ApiError> {
    Ok(Json(
        state
            .application
            .update_pack(MethodPackId(pack_id), input)
            .await?,
    ))
}

#[utoipa::path(
    get,
    path = "/projects/{project_id}/documents/{document_id}/changes",
    tag = "documents",
    summary = "List tracked changes",
    params(("project_id" = Uuid, Path, description = "The project"), ("document_id" = Uuid, Path, description = "The document")),
    responses(
        (status = 200, description = "The changes", body = Vec<TrackedChange>),
        (status = 401, description = "No valid token or session cookie", body = crate::ErrorResponse),
        (status = 404, description = "No document with that id", body = crate::ErrorResponse)
    ),
    security(("asten_token" = []))
)]
async fn list_changes(
    State(state): State<ApiState>,
    Path((_project_id, document_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<Vec<TrackedChange>>, ApiError> {
    Ok(Json(
        state
            .application
            .list_tracked_changes(DocumentId(document_id))
            .await?,
    ))
}

#[utoipa::path(
    post,
    path = "/projects/{project_id}/documents/{document_id}/changes",
    tag = "documents",
    summary = "Record a tracked change",
    description = "The `document_id` in the body is replaced by the one in the path.",
    params(("project_id" = Uuid, Path, description = "The project"), ("document_id" = Uuid, Path, description = "The document")),
    request_body = CreateTrackedChange,
    responses(
        (status = 201, description = "The change", body = TrackedChange),
        (status = 401, description = "No valid token or session cookie", body = crate::ErrorResponse),
        (status = 404, description = "No document with that id", body = crate::ErrorResponse),
        (status = 422, description = "The text is empty", body = crate::ErrorResponse)
    ),
    security(("asten_token" = []))
)]
async fn create_change(
    State(state): State<ApiState>,
    Path((_project_id, document_id)): Path<(Uuid, Uuid)>,
    Json(mut input): Json<CreateTrackedChange>,
) -> Result<(StatusCode, Json<TrackedChange>), ApiError> {
    input.document_id = DocumentId(document_id);
    let change = state.application.create_tracked_change(input).await?;
    Ok((StatusCode::CREATED, Json(change)))
}

#[utoipa::path(
    post,
    path = "/projects/{project_id}/changes/{change_id}/resolve",
    tag = "documents",
    summary = "Accept or reject a tracked change",
    params(("project_id" = Uuid, Path, description = "The project"), ("change_id" = Uuid, Path, description = "The tracked change")),
    request_body = ResolveTrackedChange,
    responses(
        (status = 200, description = "The change", body = TrackedChange),
        (status = 401, description = "No valid token or session cookie", body = crate::ErrorResponse),
        (status = 404, description = "No change with that id", body = crate::ErrorResponse)
    ),
    security(("asten_token" = []))
)]
async fn resolve_change(
    State(state): State<ApiState>,
    Path((_project_id, change_id)): Path<(Uuid, Uuid)>,
    Json(input): Json<ResolveTrackedChange>,
) -> Result<Json<TrackedChange>, ApiError> {
    Ok(Json(
        state
            .application
            .resolve_tracked_change(TrackedChangeId(change_id), input)
            .await?,
    ))
}

#[utoipa::path(
    post,
    path = "/projects/{project_id}/compile",
    tag = "compile",
    summary = "Compile the manuscript",
    description = "Renders the prose documents, or the ones named in `document_ids`, to Markdown, plain text, Fountain, HTML or DOCX. Documents marked `exclude_from_compile` are skipped. The artifact body is base64 when `encoding` says so.",
    params(("project_id" = Uuid, Path, description = "The project")),
    request_body = CompileRequest,
    responses(
        (status = 200, description = "The compiled artifact", body = CompileArtifact),
        (status = 401, description = "No valid token or session cookie", body = crate::ErrorResponse),
        (status = 404, description = "No project with that id", body = crate::ErrorResponse),
        (status = 422, description = "Nothing to compile", body = crate::ErrorResponse)
    ),
    security(("asten_token" = []))
)]
async fn compile_project(
    State(state): State<ApiState>,
    Path(project_id): Path<Uuid>,
    Json(request): Json<CompileRequest>,
) -> Result<Json<CompileArtifact>, ApiError> {
    Ok(Json(
        state
            .application
            .compile_project(ProjectId(project_id), request)
            .await?,
    ))
}

#[utoipa::path(
    post,
    path = "/projects/{project_id}/import",
    tag = "compile",
    summary = "Import a document",
    description = "Creates a document from Markdown, plain text, Fountain, HTML or DOCX. Binary bodies are base64 with `encoding` set to `base64`.",
    params(("project_id" = Uuid, Path, description = "The project")),
    request_body = ImportRequest,
    responses(
        (status = 201, description = "The imported document", body = Document),
        (status = 401, description = "No valid token or session cookie", body = crate::ErrorResponse),
        (status = 404, description = "No project with that id", body = crate::ErrorResponse),
        (status = 422, description = "The body cannot be decoded or the format is wrong", body = crate::ErrorResponse)
    ),
    security(("asten_token" = []))
)]
async fn import_document(
    State(state): State<ApiState>,
    Path(project_id): Path<Uuid>,
    Json(request): Json<ImportRequest>,
) -> Result<(StatusCode, Json<Document>), ApiError> {
    let document = state
        .application
        .import_document(ProjectId(project_id), request)
        .await?;
    Ok((StatusCode::CREATED, Json(document)))
}
