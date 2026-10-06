use std::sync::Arc;

use margin_application::{
    Application, CompileRequest, CreateCanonEntity, CreateComment, CreateDocument, CreateFact,
    CreateProject, CreateSnapshot, CreateSuppression, CreateTimelineEvent, CreateTrackedChange,
    CreateWorldEntry, ImportRequest, ResolveTrackedChange, UpdateCanonEntity, UpdateComment,
    UpdateDocument, UpdateMethodPack, UpdateTimelineEvent, UpdateWorldEntry,
};
use margin_domain::{
    CanonEntity, Comment, CommentId, CompileArtifact, Diagnostic, Document, DocumentId, DocumentOp,
    DocumentSnapshot, EntityId, Fact, MethodPack, MethodPackId, Project, ProjectId, SnapshotId,
    Suppression, TimelineEvent, TimelineEventId, TrackedChange, TrackedChangeId, WorldEntry,
    WorldEntryId, WorldPulse,
};
use tauri::{Manager, State};
use uuid::Uuid;

struct AppState(Application);

#[tauri::command]
async fn list_projects(state: State<'_, AppState>) -> Result<Vec<Project>, String> {
    state
        .0
        .list_projects()
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn create_project(
    state: State<'_, AppState>,
    input: CreateProject,
) -> Result<Project, String> {
    state
        .0
        .create_project(input)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn list_documents(
    state: State<'_, AppState>,
    project_id: Uuid,
) -> Result<Vec<Document>, String> {
    state
        .0
        .list_documents(ProjectId(project_id))
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn create_document(
    state: State<'_, AppState>,
    input: CreateDocument,
) -> Result<Document, String> {
    state
        .0
        .create_document(input)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn update_document(
    state: State<'_, AppState>,
    document_id: Uuid,
    input: UpdateDocument,
) -> Result<Document, String> {
    state
        .0
        .update_document(DocumentId(document_id), input)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn list_facts(state: State<'_, AppState>, project_id: Uuid) -> Result<Vec<Fact>, String> {
    state
        .0
        .list_facts(ProjectId(project_id))
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn create_fact(state: State<'_, AppState>, input: CreateFact) -> Result<Fact, String> {
    state
        .0
        .create_fact(input)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn list_diagnostics(
    state: State<'_, AppState>,
    project_id: Uuid,
) -> Result<Vec<Diagnostic>, String> {
    state
        .0
        .analyze_project(ProjectId(project_id))
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn list_world_entries(
    state: State<'_, AppState>,
    project_id: Uuid,
) -> Result<Vec<WorldEntry>, String> {
    state
        .0
        .list_world_entries(ProjectId(project_id))
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn create_world_entry(
    state: State<'_, AppState>,
    input: CreateWorldEntry,
) -> Result<WorldEntry, String> {
    state
        .0
        .create_world_entry(input)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn update_world_entry(
    state: State<'_, AppState>,
    entry_id: Uuid,
    input: UpdateWorldEntry,
) -> Result<WorldEntry, String> {
    state
        .0
        .update_world_entry(WorldEntryId(entry_id), input)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn world_pulse(
    state: State<'_, AppState>,
    project_id: Uuid,
) -> Result<WorldPulse, String> {
    state
        .0
        .world_pulse(ProjectId(project_id))
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn list_entities(
    state: State<'_, AppState>,
    project_id: Uuid,
) -> Result<Vec<CanonEntity>, String> {
    state
        .0
        .list_entities(ProjectId(project_id))
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn create_entity(
    state: State<'_, AppState>,
    input: CreateCanonEntity,
) -> Result<CanonEntity, String> {
    state
        .0
        .create_entity(input)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn update_entity(
    state: State<'_, AppState>,
    entity_id: Uuid,
    input: UpdateCanonEntity,
) -> Result<CanonEntity, String> {
    state
        .0
        .update_entity(EntityId(entity_id), input)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn list_events(
    state: State<'_, AppState>,
    project_id: Uuid,
) -> Result<Vec<TimelineEvent>, String> {
    state
        .0
        .list_events(ProjectId(project_id))
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn create_event(
    state: State<'_, AppState>,
    input: CreateTimelineEvent,
) -> Result<TimelineEvent, String> {
    state
        .0
        .create_event(input)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn update_event(
    state: State<'_, AppState>,
    event_id: Uuid,
    input: UpdateTimelineEvent,
) -> Result<TimelineEvent, String> {
    state
        .0
        .update_event(TimelineEventId(event_id), input)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn list_comments(
    state: State<'_, AppState>,
    document_id: Uuid,
) -> Result<Vec<Comment>, String> {
    state
        .0
        .list_comments(DocumentId(document_id))
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn create_comment(
    state: State<'_, AppState>,
    input: CreateComment,
) -> Result<Comment, String> {
    state
        .0
        .create_comment(input)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn update_comment(
    state: State<'_, AppState>,
    comment_id: Uuid,
    input: UpdateComment,
) -> Result<Comment, String> {
    state
        .0
        .update_comment(CommentId(comment_id), input)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn list_ops(
    state: State<'_, AppState>,
    document_id: Uuid,
) -> Result<Vec<DocumentOp>, String> {
    state
        .0
        .list_ops(DocumentId(document_id))
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn list_snapshots(
    state: State<'_, AppState>,
    document_id: Uuid,
) -> Result<Vec<DocumentSnapshot>, String> {
    state
        .0
        .list_snapshots(DocumentId(document_id))
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn create_snapshot(
    state: State<'_, AppState>,
    input: CreateSnapshot,
) -> Result<DocumentSnapshot, String> {
    state
        .0
        .create_snapshot(input)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn restore_snapshot(
    state: State<'_, AppState>,
    snapshot_id: Uuid,
) -> Result<Document, String> {
    state
        .0
        .restore_snapshot(SnapshotId(snapshot_id))
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn list_suppressions(
    state: State<'_, AppState>,
    project_id: Uuid,
) -> Result<Vec<Suppression>, String> {
    state
        .0
        .list_suppressions(ProjectId(project_id))
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn create_suppression(
    state: State<'_, AppState>,
    input: CreateSuppression,
) -> Result<Suppression, String> {
    state
        .0
        .create_suppression(input)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn list_packs(state: State<'_, AppState>) -> Result<Vec<MethodPack>, String> {
    state.0.list_packs().await.map_err(|error| error.to_string())
}

#[tauri::command]
async fn update_pack(
    state: State<'_, AppState>,
    pack_id: Uuid,
    input: UpdateMethodPack,
) -> Result<MethodPack, String> {
    state
        .0
        .update_pack(MethodPackId(pack_id), input)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn list_tracked_changes(
    state: State<'_, AppState>,
    document_id: Uuid,
) -> Result<Vec<TrackedChange>, String> {
    state
        .0
        .list_tracked_changes(DocumentId(document_id))
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn create_tracked_change(
    state: State<'_, AppState>,
    input: CreateTrackedChange,
) -> Result<TrackedChange, String> {
    state
        .0
        .create_tracked_change(input)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn resolve_tracked_change(
    state: State<'_, AppState>,
    change_id: Uuid,
    input: ResolveTrackedChange,
) -> Result<TrackedChange, String> {
    state
        .0
        .resolve_tracked_change(TrackedChangeId(change_id), input)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn compile_project(
    state: State<'_, AppState>,
    project_id: Uuid,
    request: CompileRequest,
) -> Result<CompileArtifact, String> {
    state
        .0
        .compile_project(ProjectId(project_id), request)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn import_document(
    state: State<'_, AppState>,
    project_id: Uuid,
    request: ImportRequest,
) -> Result<Document, String> {
    state
        .0
        .import_document(ProjectId(project_id), request)
        .await
        .map_err(|error| error.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let data_directory = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_directory)?;
            let database_path = data_directory.join("asten.db");
            let repository = tauri::async_runtime::block_on(
                margin_storage::SqliteRepository::open(database_path),
            )?;
            app.manage(AppState(Application::new(Arc::new(repository))));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_projects,
            create_project,
            list_documents,
            create_document,
            update_document,
            list_facts,
            create_fact,
            list_diagnostics,
            list_world_entries,
            create_world_entry,
            update_world_entry,
            world_pulse,
            list_entities,
            create_entity,
            update_entity,
            list_events,
            create_event,
            update_event,
            list_comments,
            create_comment,
            update_comment,
            list_ops,
            list_snapshots,
            create_snapshot,
            restore_snapshot,
            list_suppressions,
            create_suppression,
            list_packs,
            update_pack,
            list_tracked_changes,
            create_tracked_change,
            resolve_tracked_change,
            compile_project,
            import_document
        ])
        .run(tauri::generate_context!())
        .expect("error while running Asten");
}
