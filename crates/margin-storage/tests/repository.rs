use std::sync::Arc;

use margin_application::{Application, CreateDocument, CreateProject};
use margin_domain::{DocumentKind, ProjectKind};
use margin_storage::SqliteRepository;
use serde_json::json;
use uuid::Uuid;

#[tokio::test]
async fn persists_projects_and_documents() {
    let path = std::env::temp_dir().join(format!("margin-test-{}.db", Uuid::now_v7()));
    let repository = SqliteRepository::open(&path).await.unwrap();
    let application = Application::new(Arc::new(repository));

    let project = application
        .create_project(CreateProject {
            name: "A test novel".into(),
            kind: ProjectKind::Novel,
            description: None,
        })
        .await
        .unwrap();
    application
        .create_document(CreateDocument {
            project_id: project.id,
            parent_id: None,
            kind: DocumentKind::Chapter,
            title: "Chapter one".into(),
            content_json: json!({"type": "doc", "content": []}),
            position: 0,
        })
        .await
        .unwrap();

    assert_eq!(application.list_projects().await.unwrap().len(), 1);
    assert_eq!(
        application.list_documents(project.id).await.unwrap().len(),
        1
    );

    drop(application);
    let _ = std::fs::remove_file(&path);
    let _ = std::fs::remove_file(path.with_extension("db-shm"));
    let _ = std::fs::remove_file(path.with_extension("db-wal"));
}
