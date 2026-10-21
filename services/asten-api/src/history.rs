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

#[utoipa::path(
    get,
    path = "/projects/{project_id}/documents/{document_id}/history",
    tag = "history",
    summary = "The document's event log",
    description = "Every block event in sequence order, with its actor, time, content hash and chain hashes.",
    params(("project_id" = Uuid, Path, description = "The project"), ("document_id" = Uuid, Path, description = "The document")),
    responses(
        (status = 200, description = "The events", body = Vec<EventRecord>),
        (status = 401, description = "No valid token or session cookie", body = crate::ErrorResponse),
        (status = 404, description = "No document with that id", body = crate::ErrorResponse)
    ),
    security(("asten_token" = []))
)]
async fn history(
    State(state): State<ApiState>,
    Path((_project_id, document_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<Vec<EventRecord>>, ApiError> {
    Ok(Json(
        state
            .application
            .document_history(DocumentId(document_id))
            .await?,
    ))
}

#[utoipa::path(
    get,
    path = "/projects/{project_id}/documents/{document_id}/verify",
    tag = "history",
    summary = "Verify the event log",
    description = "Recomputes the hash chain from the first event and the Merkle root of the current blocks, and says whether they match what is stored.",
    params(("project_id" = Uuid, Path, description = "The project"), ("document_id" = Uuid, Path, description = "The document")),
    responses(
        (status = 200, description = "The report", body = VerifyReport),
        (status = 401, description = "No valid token or session cookie", body = crate::ErrorResponse),
        (status = 404, description = "No document with that id", body = crate::ErrorResponse)
    ),
    security(("asten_token" = []))
)]
async fn verify(
    State(state): State<ApiState>,
    Path((_project_id, document_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<VerifyReport>, ApiError> {
    Ok(Json(
        state
            .application
            .verify_document(DocumentId(document_id))
            .await?,
    ))
}

#[utoipa::path(
    get,
    path = "/projects/{project_id}/documents/{document_id}/at/{seq}",
    tag = "history",
    summary = "The document at a sequence number",
    description = "Materialises the ProseMirror JSON as it stood after event `seq`, from the nearest snapshot forward.",
    params(("project_id" = Uuid, Path, description = "The project"), ("document_id" = Uuid, Path, description = "The document"), ("seq" = i64, Path, description = "An event sequence number; 0 is the empty document")),
    responses(
        (status = 200, description = "The ProseMirror document", body = serde_json::Value),
        (status = 401, description = "No valid token or session cookie", body = crate::ErrorResponse),
        (status = 404, description = "No document with that id", body = crate::ErrorResponse)
    ),
    security(("asten_token" = []))
)]
async fn at_seq(
    State(state): State<ApiState>,
    Path((_project_id, document_id, seq)): Path<(Uuid, Uuid, i64)>,
) -> Result<Json<serde_json::Value>, ApiError> {
    Ok(Json(
        state
            .application
            .document_at(DocumentId(document_id), seq)
            .await?,
    ))
}

#[utoipa::path(
    get,
    path = "/projects/{project_id}/documents/{document_id}/blame",
    tag = "history",
    summary = "Blame",
    description = "Per block id, which event last wrote each span of its text.",
    params(("project_id" = Uuid, Path, description = "The project"), ("document_id" = Uuid, Path, description = "The document")),
    responses(
        (status = 200, description = "Spans keyed by block id", body = BTreeMap<String, Vec<BlameSpan>>),
        (status = 401, description = "No valid token or session cookie", body = crate::ErrorResponse),
        (status = 404, description = "No document with that id", body = crate::ErrorResponse)
    ),
    security(("asten_token" = []))
)]
async fn blame(
    State(state): State<ApiState>,
    Path((_project_id, document_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<BTreeMap<String, Vec<BlameSpan>>>, ApiError> {
    Ok(Json(
        state
            .application
            .document_blame(DocumentId(document_id))
            .await?,
    ))
}

#[utoipa::path(
    get,
    path = "/projects/{project_id}/documents/{document_id}/drafts",
    tag = "history",
    summary = "List drafts",
    description = "Personal branches of the document, merged back three-way.",
    params(("project_id" = Uuid, Path, description = "The project"), ("document_id" = Uuid, Path, description = "The document")),
    responses(
        (status = 200, description = "The drafts", body = Vec<Draft>),
        (status = 401, description = "No valid token or session cookie", body = crate::ErrorResponse),
        (status = 404, description = "No document with that id", body = crate::ErrorResponse)
    ),
    security(("asten_token" = []))
)]
async fn list_drafts(
    State(state): State<ApiState>,
    Path((_project_id, document_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<Vec<Draft>>, ApiError> {
    Ok(Json(
        state
            .application
            .list_drafts(DocumentId(document_id))
            .await?,
    ))
}

#[utoipa::path(
    post,
    path = "/projects/{project_id}/documents/{document_id}/drafts",
    tag = "history",
    summary = "Branch a draft",
    description = "Copies the document's current content and remembers the sequence it branched from.",
    params(("project_id" = Uuid, Path, description = "The project"), ("document_id" = Uuid, Path, description = "The document")),
    request_body = CreateDraft,
    responses(
        (status = 201, description = "The draft", body = Draft),
        (status = 401, description = "No valid token or session cookie", body = crate::ErrorResponse),
        (status = 404, description = "No document with that id", body = crate::ErrorResponse),
        (status = 422, description = "The name is empty", body = crate::ErrorResponse)
    ),
    security(("asten_token" = []))
)]
async fn create_draft(
    State(state): State<ApiState>,
    Path((_project_id, document_id)): Path<(Uuid, Uuid)>,
    Json(input): Json<CreateDraft>,
) -> Result<(StatusCode, Json<Draft>), ApiError> {
    let draft = state
        .application
        .create_draft(DocumentId(document_id), input)
        .await?;
    Ok((StatusCode::CREATED, Json(draft)))
}

#[utoipa::path(
    put,
    path = "/projects/{project_id}/drafts/{draft_id}",
    tag = "history",
    summary = "Update a draft",
    params(("project_id" = Uuid, Path, description = "The project"), ("draft_id" = Uuid, Path, description = "The draft")),
    request_body = UpdateDraft,
    responses(
        (status = 200, description = "The draft", body = Draft),
        (status = 401, description = "No valid token or session cookie", body = crate::ErrorResponse),
        (status = 404, description = "No draft with that id", body = crate::ErrorResponse),
        (status = 409, description = "The draft was already merged", body = crate::ErrorResponse)
    ),
    security(("asten_token" = []))
)]
async fn update_draft(
    State(state): State<ApiState>,
    Path((_project_id, draft_id)): Path<(Uuid, Uuid)>,
    Json(input): Json<UpdateDraft>,
) -> Result<Json<Draft>, ApiError> {
    Ok(Json(
        state
            .application
            .update_draft(DraftId(draft_id), input)
            .await?,
    ))
}

#[utoipa::path(
    delete,
    path = "/projects/{project_id}/drafts/{draft_id}",
    tag = "history",
    summary = "Delete a draft",
    params(("project_id" = Uuid, Path, description = "The project"), ("draft_id" = Uuid, Path, description = "The draft")),
    responses(
        (status = 204, description = "Deleted"),
        (status = 401, description = "No valid token or session cookie", body = crate::ErrorResponse),
        (status = 404, description = "No draft with that id", body = crate::ErrorResponse)
    ),
    security(("asten_token" = []))
)]
async fn delete_draft(
    State(state): State<ApiState>,
    Path((_project_id, draft_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    state.application.delete_draft(DraftId(draft_id)).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/projects/{project_id}/drafts/{draft_id}/merge-preview",
    tag = "history",
    summary = "Preview a three-way merge",
    description = "Merges the draft against the main document from their common base without writing anything, and lists the conflicts left for `resolutions` to settle.",
    params(("project_id" = Uuid, Path, description = "The project"), ("draft_id" = Uuid, Path, description = "The draft")),
    request_body = MergeRequest,
    responses(
        (status = 200, description = "The merged content and its conflicts", body = MergePreview),
        (status = 401, description = "No valid token or session cookie", body = crate::ErrorResponse),
        (status = 404, description = "No draft with that id", body = crate::ErrorResponse)
    ),
    security(("asten_token" = []))
)]
async fn merge_preview(
    State(state): State<ApiState>,
    Path((_project_id, draft_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<MergeRequest>,
) -> Result<Json<MergePreview>, ApiError> {
    Ok(Json(
        state
            .application
            .merge_preview(DraftId(draft_id), &request)
            .await?,
    ))
}

#[utoipa::path(
    post,
    path = "/projects/{project_id}/drafts/{draft_id}/merge",
    tag = "history",
    summary = "Merge a draft",
    description = "Commits the three-way merge to the main document. Every remaining conflict needs a side named in `resolutions`.",
    params(("project_id" = Uuid, Path, description = "The project"), ("draft_id" = Uuid, Path, description = "The draft")),
    request_body = MergeRequest,
    responses(
        (status = 200, description = "The main document after the merge", body = Document),
        (status = 401, description = "No valid token or session cookie", body = crate::ErrorResponse),
        (status = 404, description = "No draft with that id", body = crate::ErrorResponse),
        (status = 409, description = "Conflicts remain unresolved, or the draft was already merged", body = crate::ErrorResponse)
    ),
    security(("asten_token" = []))
)]
async fn merge_draft(
    State(state): State<ApiState>,
    Path((_project_id, draft_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<MergeRequest>,
) -> Result<Json<Document>, ApiError> {
    Ok(Json(
        state
            .application
            .merge_draft(DraftId(draft_id), &request)
            .await?,
    ))
}
