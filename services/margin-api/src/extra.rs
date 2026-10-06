use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::get,
};
use margin_application::{
    CompileRequest, CreateCanonEntity, CreateComment, CreateSnapshot, CreateSuppression,
    CreateTimelineEvent, CreateTrackedChange, ImportRequest, ResolveTrackedChange,
    UpdateCanonEntity, UpdateComment, UpdateMethodPack, UpdateTimelineEvent,
};
use margin_domain::{
    CanonEntity, Comment, CommentId, CompileArtifact, Document, DocumentId, DocumentOp,
    DocumentSnapshot, EntityId, MethodPack, MethodPackId, ProjectId, SnapshotId, Suppression,
    TimelineEvent, TimelineEventId, TrackedChange, TrackedChangeId,
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
            axum::routing::put(update_entity),
        )
        .route(
            "/projects/{project_id}/events",
            get(list_events).post(create_event),
        )
        .route(
            "/projects/{project_id}/events/{event_id}",
            axum::routing::put(update_event),
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
}

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

async fn create_entity(
    State(state): State<ApiState>,
    Path(project_id): Path<Uuid>,
    Json(mut input): Json<CreateCanonEntity>,
) -> Result<(StatusCode, Json<CanonEntity>), ApiError> {
    input.project_id = ProjectId(project_id);
    let entity = state.application.create_entity(input).await?;
    Ok((StatusCode::CREATED, Json(entity)))
}

async fn update_entity(
    State(state): State<ApiState>,
    Path((_, entity_id)): Path<(Uuid, Uuid)>,
    Json(input): Json<UpdateCanonEntity>,
) -> Result<Json<CanonEntity>, ApiError> {
    Ok(Json(
        state
            .application
            .update_entity(EntityId(entity_id), input)
            .await?,
    ))
}

async fn list_events(
    State(state): State<ApiState>,
    Path(project_id): Path<Uuid>,
) -> Result<Json<Vec<TimelineEvent>>, ApiError> {
    Ok(Json(
        state.application.list_events(ProjectId(project_id)).await?,
    ))
}

async fn create_event(
    State(state): State<ApiState>,
    Path(project_id): Path<Uuid>,
    Json(mut input): Json<CreateTimelineEvent>,
) -> Result<(StatusCode, Json<TimelineEvent>), ApiError> {
    input.project_id = ProjectId(project_id);
    let event = state.application.create_event(input).await?;
    Ok((StatusCode::CREATED, Json(event)))
}

async fn update_event(
    State(state): State<ApiState>,
    Path((_, event_id)): Path<(Uuid, Uuid)>,
    Json(input): Json<UpdateTimelineEvent>,
) -> Result<Json<TimelineEvent>, ApiError> {
    Ok(Json(
        state
            .application
            .update_event(TimelineEventId(event_id), input)
            .await?,
    ))
}

async fn list_comments(
    State(state): State<ApiState>,
    Path((_, document_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<Vec<Comment>>, ApiError> {
    Ok(Json(
        state
            .application
            .list_comments(DocumentId(document_id))
            .await?,
    ))
}

async fn create_comment(
    State(state): State<ApiState>,
    Path((_, document_id)): Path<(Uuid, Uuid)>,
    Json(mut input): Json<CreateComment>,
) -> Result<(StatusCode, Json<Comment>), ApiError> {
    input.document_id = DocumentId(document_id);
    let comment = state.application.create_comment(input).await?;
    Ok((StatusCode::CREATED, Json(comment)))
}

async fn update_comment(
    State(state): State<ApiState>,
    Path((_, comment_id)): Path<(Uuid, Uuid)>,
    Json(input): Json<UpdateComment>,
) -> Result<Json<Comment>, ApiError> {
    Ok(Json(
        state
            .application
            .update_comment(CommentId(comment_id), input)
            .await?,
    ))
}

async fn list_ops(
    State(state): State<ApiState>,
    Path((_, document_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<Vec<DocumentOp>>, ApiError> {
    Ok(Json(
        state.application.list_ops(DocumentId(document_id)).await?,
    ))
}

async fn list_snapshots(
    State(state): State<ApiState>,
    Path((_, document_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<Vec<DocumentSnapshot>>, ApiError> {
    Ok(Json(
        state
            .application
            .list_snapshots(DocumentId(document_id))
            .await?,
    ))
}

async fn create_snapshot(
    State(state): State<ApiState>,
    Path((_, document_id)): Path<(Uuid, Uuid)>,
    Json(mut input): Json<CreateSnapshot>,
) -> Result<(StatusCode, Json<DocumentSnapshot>), ApiError> {
    input.document_id = DocumentId(document_id);
    let snapshot = state.application.create_snapshot(input).await?;
    Ok((StatusCode::CREATED, Json(snapshot)))
}

async fn restore_snapshot(
    State(state): State<ApiState>,
    Path((_, snapshot_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<Document>, ApiError> {
    Ok(Json(
        state
            .application
            .restore_snapshot(SnapshotId(snapshot_id))
            .await?,
    ))
}

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

async fn create_suppression(
    State(state): State<ApiState>,
    Path(project_id): Path<Uuid>,
    Json(mut input): Json<CreateSuppression>,
) -> Result<(StatusCode, Json<Suppression>), ApiError> {
    input.project_id = ProjectId(project_id);
    let suppression = state.application.create_suppression(input).await?;
    Ok((StatusCode::CREATED, Json(suppression)))
}

async fn list_packs(State(state): State<ApiState>) -> Result<Json<Vec<MethodPack>>, ApiError> {
    Ok(Json(state.application.list_packs().await?))
}

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

async fn list_changes(
    State(state): State<ApiState>,
    Path((_, document_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<Vec<TrackedChange>>, ApiError> {
    Ok(Json(
        state
            .application
            .list_tracked_changes(DocumentId(document_id))
            .await?,
    ))
}

async fn create_change(
    State(state): State<ApiState>,
    Path((_, document_id)): Path<(Uuid, Uuid)>,
    Json(mut input): Json<CreateTrackedChange>,
) -> Result<(StatusCode, Json<TrackedChange>), ApiError> {
    input.document_id = DocumentId(document_id);
    let change = state.application.create_tracked_change(input).await?;
    Ok((StatusCode::CREATED, Json(change)))
}

async fn resolve_change(
    State(state): State<ApiState>,
    Path((_, change_id)): Path<(Uuid, Uuid)>,
    Json(input): Json<ResolveTrackedChange>,
) -> Result<Json<TrackedChange>, ApiError> {
    Ok(Json(
        state
            .application
            .resolve_tracked_change(TrackedChangeId(change_id), input)
            .await?,
    ))
}

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
