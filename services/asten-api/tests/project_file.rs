//! The project file: a project out as JSON and back in as a new project, with
//! every reference between records rewritten; renaming and deleting a project;
//! and the ways a project file can be refused.

use std::sync::Arc;

use asten_api::{ApiConfig, router};
use asten_application::{Application, AuthMode};
use asten_storage::SqliteRepository;
use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode, header},
};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tower::ServiceExt;

async fn desk() -> Router {
    let repository = SqliteRepository::in_memory().await.unwrap();
    let application = Application::with_auth(Arc::new(repository), AuthMode::Off);
    router(
        application,
        ApiConfig {
            auth_mode: AuthMode::Off,
            ..ApiConfig::default()
        },
    )
}

async fn json(app: &Router, method: &str, path: &str, body: Option<Value>) -> (StatusCode, Value) {
    let mut request = Request::builder()
        .method(method)
        .uri(format!("/api/v1{path}"))
        .header(header::ACCEPT, "application/json");
    if body.is_some() {
        request = request.header(header::CONTENT_TYPE, "application/json");
    }
    let request = request
        .body(match body {
            Some(value) => Body::from(value.to_string()),
            None => Body::empty(),
        })
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}

fn paragraph(text: &str) -> Value {
    json!({"type":"paragraph","attrs":{"id": uuid::Uuid::now_v7().to_string()},"content":[{"type":"text","text":text}]})
}

#[tokio::test]
async fn a_project_leaves_as_one_file_and_comes_back_as_a_new_project() {
    let app = desk().await;
    let (_, project) = json(
        &app,
        "POST",
        "/projects",
        Some(json!({"name":"The Harbor","kind":"novel","description":"A test"})),
    )
    .await;
    let pid = project["id"].as_str().unwrap().to_owned();
    let (_, root) = json(
        &app,
        "POST",
        &format!("/projects/{pid}/documents"),
        Some(json!({"kind":"manuscript","title":"Book"})),
    )
    .await;
    let root_id = root["id"].as_str().unwrap().to_owned();
    let (_, chapter) = json(
        &app,
        "POST",
        &format!("/projects/{pid}/documents"),
        Some(json!({
            "kind":"chapter","title":"One","parent_id":root_id,"position":2,
            "content_json":{"type":"doc","content":[paragraph("Mira pushed her black hair back.")]},
            "scene":{"story_time":"day 1"}
        })),
    )
    .await;
    let chapter_id = chapter["id"].as_str().unwrap().to_owned();
    let (_, entity) = json(
        &app,
        "POST",
        &format!("/projects/{pid}/entities"),
        Some(json!({"kind":"character","name":"Mira","aliases":["the keeper"],"notes":"Keeper."})),
    )
    .await;
    json(&app, "POST", &format!("/projects/{pid}/facts"), Some(json!({"entity_key":"mira","predicate":"hair_color","value":"black","status":"confirmed"}))).await;
    json(
        &app,
        "POST",
        &format!("/projects/{pid}/events"),
        Some(json!({"title":"Arrival","entity_id":entity["id"],"document_id":chapter_id,"story_time":"day 1","reading_order":1})),
    )
    .await;
    json(
        &app,
        "POST",
        &format!("/projects/{pid}/world-entries"),
        Some(json!({"kind":"place","title":"Gull Point","summary":"A headland."})),
    )
    .await;
    json(
        &app,
        "POST",
        &format!("/projects/{pid}/documents/{chapter_id}/comments"),
        Some(json!({"body":"Check the colour","from_offset":2,"to_offset":10,"block_id": chapter["content_json"]["content"][0]["attrs"]["id"]})),
    )
    .await;
    json(
        &app,
        "PUT",
        &format!("/projects/{pid}/settings"),
        Some(json!({"strictness":"thorough"})),
    )
    .await;

    // Out.
    let (status, file) = json(&app, "GET", &format!("/projects/{pid}/export"), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(file["format"], "asten-project");
    assert_eq!(file["version"], 1);
    assert_eq!(file["documents"].as_array().unwrap().len(), 2);

    // In, as a new project.
    let (status, copy) = json(&app, "POST", "/projects/bundle", Some(file.clone())).await;
    assert_eq!(status, StatusCode::CREATED, "{copy}");
    assert_ne!(copy["id"], project["id"]);
    assert_eq!(copy["name"], "The Harbor");
    assert_eq!(copy["description"], "A test");
    let cid = copy["id"].as_str().unwrap().to_owned();

    let (_, docs) = json(&app, "GET", &format!("/projects/{cid}/documents"), None).await;
    let docs = docs.as_array().unwrap();
    assert_eq!(docs.len(), 2);
    let new_root = docs.iter().find(|d| d["title"] == "Book").unwrap();
    let new_chapter = docs.iter().find(|d| d["title"] == "One").unwrap();
    assert_ne!(new_chapter["id"], chapter["id"]);
    assert_eq!(
        new_chapter["parent_id"], new_root["id"],
        "the tree is rebuilt on the new ids"
    );
    assert_eq!(new_chapter["position"], 2);
    assert_eq!(new_chapter["scene"]["story_time"], "day 1");
    assert_eq!(
        new_chapter["content_json"]["content"][0]["content"][0]["text"],
        "Mira pushed her black hair back."
    );

    let (_, entities) = json(&app, "GET", &format!("/projects/{cid}/entities"), None).await;
    assert_eq!(entities[0]["name"], "Mira");
    assert_eq!(entities[0]["aliases"][0], "the keeper");
    let (_, facts) = json(&app, "GET", &format!("/projects/{cid}/facts"), None).await;
    assert_eq!(facts[0]["value"], "black");
    assert_eq!(facts[0]["status"], "confirmed");
    let (_, events) = json(&app, "GET", &format!("/projects/{cid}/events"), None).await;
    assert_eq!(events[0]["title"], "Arrival");
    assert_eq!(
        events[0]["document_id"], new_chapter["id"],
        "events follow their scene to the new id"
    );
    assert_eq!(events[0]["entity_id"], entities[0]["id"]);
    let (_, world) = json(&app, "GET", &format!("/projects/{cid}/world-entries"), None).await;
    assert_eq!(world[0]["title"], "Gull Point");
    let new_chapter_id = new_chapter["id"].as_str().unwrap();
    let (_, comments) = json(
        &app,
        "GET",
        &format!("/projects/{cid}/documents/{new_chapter_id}/comments"),
        None,
    )
    .await;
    assert_eq!(comments[0]["body"], "Check the colour");
    assert_eq!(comments[0]["from_offset"], 2);
    let (_, settings) = json(&app, "GET", &format!("/projects/{cid}/settings"), None).await;
    assert_eq!(settings["strictness"], "thorough");

    // The original is untouched; the copy is its own project.
    let (_, listed) = json(&app, "GET", "/projects", None).await;
    assert_eq!(listed.as_array().unwrap().len(), 2);
    let (_, diag) = json(
        &app,
        "GET",
        &format!("/projects/{cid}/story/diagnostics"),
        None,
    )
    .await;
    assert!(diag["diagnostics"].is_array());
}

#[tokio::test]
async fn a_project_can_be_renamed_and_deleted() {
    let app = desk().await;
    let (_, project) = json(
        &app,
        "POST",
        "/projects",
        Some(json!({"name":"Old","kind":"novel"})),
    )
    .await;
    let pid = project["id"].as_str().unwrap();
    json(
        &app,
        "POST",
        &format!("/projects/{pid}/documents"),
        Some(json!({"kind":"scene","title":"S"})),
    )
    .await;
    let (status, renamed) = json(
        &app,
        "PUT",
        &format!("/projects/{pid}"),
        Some(json!({"name":"  New  "})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(renamed["name"], "New");
    let (status, _) = json(
        &app,
        "PUT",
        &format!("/projects/{pid}"),
        Some(json!({"name":"   "})),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    let (status, _) = json(&app, "DELETE", &format!("/projects/{pid}"), None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (status, _) = json(&app, "GET", &format!("/projects/{pid}"), None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (status, _) = json(&app, "DELETE", &format!("/projects/{pid}"), None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (_, listed) = json(&app, "GET", "/projects", None).await;
    assert!(listed.as_array().unwrap().is_empty());
}

#[tokio::test]
async fn files_that_are_not_projects_are_refused_whole() {
    let app = desk().await;
    for bad in [
        json!({"hello": "world"}),
        json!({"format":"asten-project","version":99,"project":{"name":"X"}}),
        json!({"format":"asten-project","version":1,"project":{"name":""}}),
        json!({"format":"asten-project","version":1,"project":{"name":"X"},"documents":[{"id":"not a document"}]}),
        json!({"format":"asten-project","version":1,"project":{"name":"X"},"facts":"nope"}),
    ] {
        let (status, body) = json(&app, "POST", "/projects/bundle", Some(bad.clone())).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{bad}: {body}");
    }
    let (_, listed) = json(&app, "GET", "/projects", None).await;
    assert!(
        listed.as_array().unwrap().is_empty(),
        "a refused file leaves no half-made project"
    );
}
