use std::sync::Arc;

use margin_application::{Application, CreateDocument, CreateProject};
use margin_domain::{Document, Project, ProjectId};
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
            create_document
        ])
        .run(tauri::generate_context!())
        .expect("error while running Margin");
}
