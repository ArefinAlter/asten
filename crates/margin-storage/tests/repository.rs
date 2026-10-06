use std::sync::Arc;

use margin_application::{
    Application, CreateDocument, CreateFact, CreateProject, CreateWorldEntry, UpdateWorldEntry,
};
use margin_domain::{DocumentKind, ProjectKind, Severity, WorldKind, WorldStatus};
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

    application
        .create_fact(CreateFact {
            project_id: project.id,
            entity_key: "Mira".into(),
            predicate: "hair_color".into(),
            value: "auburn".into(),
            modality: Default::default(),
            status: Default::default(),
            valid_from: None,
            valid_to: None,
            source: None,
        })
        .await
        .unwrap();
    application
        .create_document(CreateDocument {
            project_id: project.id,
            parent_id: None,
            kind: DocumentKind::Scene,
            title: "Harbor".into(),
            content_json: json!({
                "text": "Mira pushed her black hair out of her eyes."
            }),
            position: 1,
        })
        .await
        .unwrap();
    let diagnostics = application.analyze_project(project.id).await.unwrap();
    assert!(
        diagnostics.iter().any(|item| {
            item.code == "CANON-E013" && item.severity == Severity::Warning
        }),
        "expected a lexical hair-color warning, got {diagnostics:?}"
    );

    let place = application
        .create_world_entry(CreateWorldEntry {
            project_id: project.id,
            kind: WorldKind::Place,
            title: "Dusk Harbor".into(),
            summary: "Appears only after the bells.".into(),
            prompt: None,
            status: Some(WorldStatus::Spark),
            board_x: None,
            board_y: None,
            map_x: None,
            map_y: None,
            hue: None,
        })
        .await
        .unwrap();
    application
        .update_world_entry(
            place.id,
            UpdateWorldEntry {
                status: Some(WorldStatus::Settled),
                map_x: Some(Some(42.0)),
                map_y: Some(Some(61.0)),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    let pulse = application.world_pulse(project.id).await.unwrap();
    assert_eq!(pulse.places, 1);
    assert_eq!(pulse.settled, 1);
    assert!(pulse.discoveries.contains(&"ugly_triangles".into()));
    let facts = application.list_facts(project.id).await.unwrap();
    assert!(
        facts
            .iter()
            .any(|fact| fact.entity_key == "dusk_harbor" && fact.predicate == "place"),
        "settling a place should mint a confirmed fact, got {facts:?}"
    );
    let compiled = application
        .compile_project(
            project.id,
            margin_application::CompileRequest {
                format: margin_application::CompileFormat::Markdown,
                document_ids: Vec::new(),
            },
        )
        .await
        .unwrap();
    assert!(compiled.body.contains("Harbor"));
    drop(application);
    let _ = std::fs::remove_file(&path);
    let _ = std::fs::remove_file(path.with_extension("db-shm"));
    let _ = std::fs::remove_file(path.with_extension("db-wal"));
}
