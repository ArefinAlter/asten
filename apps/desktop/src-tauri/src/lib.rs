use std::sync::Arc;

use margin_application::{
    Application, CreateDocument, CreateFact, CreateProject, CreateWorldEntry, UpdateDocument,
    UpdateWorldEntry,
};
use margin_domain::{
    Diagnostic, Document, DocumentId, Fact, Project, ProjectId, WorldEntry, WorldEntryId, WorldPulse,
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let data_directory = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_directory)?;
            let database_path = data_directory.join("margin.db");
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
            world_pulse
        ])
        .run(tauri::generate_context!())
        .expect("error while running Margin");
}
