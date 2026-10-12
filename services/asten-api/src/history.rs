//! Document history, verification, blame and drafts.

use std::collections::BTreeMap;

use asten_application::{CreateDraft, MergePreview, MergeRequest, UpdateDraft, VerifyReport};
use asten_blocks_shim::{BlameSpan, EventRecord};
use asten_domain::{Document, DocumentId, Draft, DraftId};
use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
};
use uuid::Uuid;

use crate::{ApiError, ApiState};

mod asten_blocks_shim {
    // The API exposes the block model's public records through the application crate.
    pub use asten_application::history::{BlameSpan, EventRecord};
}

pub fn router() -> Router<ApiState> {
    Router::new()
        .route(
            "/projects/{project_id}/documents/{document_id}/history",
            get(history),
        )
        .route(
            "/projects/{project_id}/documents/{document_id}/verify",
            get(verify),
        )
        .route(
            "/projects/{project_id}/documents/{document_id}/at/{seq}",
            get(at_seq),
        )
        .route(
            "/projects/{project_id}/documents/{document_id}/blame",
            get(blame),
        )
        .route(
            "/projects/{project_id}/documents/{document_id}/drafts",
            get(list_drafts).post(create_draft),
        )
        .route(
            "/projects/{project_id}/drafts/{draft_id}",
            axum::routing::put(update_draft).delete(delete_draft),
        )
        .route(
            "/projects/{project_id}/drafts/{draft_id}/merge-preview",
            post(merge_preview),
        )
        .route(
            "/projects/{project_id}/drafts/{draft_id}/merge",
            post(merge_draft),
        )
}

async fn history(
    State(state): State<ApiState>,
    Path((_, document_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<Vec<EventRecord>>, ApiError> {
    Ok(Json(
        state
            .application
            .document_history(DocumentId(document_id))
            .await?,
    ))
}

async fn verify(
    State(state): State<ApiState>,
    Path((_, document_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<VerifyReport>, ApiError> {
    Ok(Json(
        state
            .application
            .verify_document(DocumentId(document_id))
            .await?,
    ))
}

async fn at_seq(
    State(state): State<ApiState>,
    Path((_, document_id, seq)): Path<(Uuid, Uuid, i64)>,
) -> Result<Json<serde_json::Value>, ApiError> {
    Ok(Json(
        state
            .application
            .document_at(DocumentId(document_id), seq)
            .await?,
    ))
}

async fn blame(
    State(state): State<ApiState>,
    Path((_, document_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<BTreeMap<String, Vec<BlameSpan>>>, ApiError> {
    Ok(Json(
        state
            .application
            .document_blame(DocumentId(document_id))
            .await?,
    ))
}

async fn list_drafts(
    State(state): State<ApiState>,
    Path((_, document_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<Vec<Draft>>, ApiError> {
    Ok(Json(
        state
            .application
            .list_drafts(DocumentId(document_id))
            .await?,
    ))
}

async fn create_draft(
    State(state): State<ApiState>,
    Path((_, document_id)): Path<(Uuid, Uuid)>,
    Json(input): Json<CreateDraft>,
) -> Result<(StatusCode, Json<Draft>), ApiError> {
    let draft = state
        .application
        .create_draft(DocumentId(document_id), input)
        .await?;
    Ok((StatusCode::CREATED, Json(draft)))
}

async fn update_draft(
    State(state): State<ApiState>,
    Path((_, draft_id)): Path<(Uuid, Uuid)>,
    Json(input): Json<UpdateDraft>,
) -> Result<Json<Draft>, ApiError> {
    Ok(Json(
        state
            .application
            .update_draft(DraftId(draft_id), input)
            .await?,
    ))
}

async fn delete_draft(
    State(state): State<ApiState>,
    Path((_, draft_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    state.application.delete_draft(DraftId(draft_id)).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn merge_preview(
    State(state): State<ApiState>,
    Path((_, draft_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<MergeRequest>,
) -> Result<Json<MergePreview>, ApiError> {
    Ok(Json(
        state
            .application
            .merge_preview(DraftId(draft_id), &request)
            .await?,
    ))
}

async fn merge_draft(
    State(state): State<ApiState>,
    Path((_, draft_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<MergeRequest>,
) -> Result<Json<Document>, ApiError> {
    Ok(Json(
        state
            .application
            .merge_draft(DraftId(draft_id), &request)
            .await?,
    ))
}
