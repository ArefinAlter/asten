//! The documents routes: summaries without bodies, one body at a time, and
//! how much each costs on the wire.

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

async fn call(
    app: &Router,
    method: &str,
    path: &str,
    body: Option<Value>,
) -> (StatusCode, Vec<u8>) {
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
    (status, bytes.to_vec())
}

async fn json(app: &Router, method: &str, path: &str, body: Option<Value>) -> (StatusCode, Value) {
    let (status, bytes) = call(app, method, path, body).await;
    let value = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, value)
}

fn paragraph(id: &str, text: &str) -> Value {
    json!({"type":"paragraph","attrs":{"id":id},"content":[{"type":"text","text":text}]})
}

async fn project(app: &Router) -> String {
    let (status, project) = json(
        app,
        "POST",
        "/projects",
        Some(json!({"name":"The Harbor","kind":"novel"})),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    project["id"].as_str().unwrap().to_owned()
}

#[tokio::test]
async fn summaries_carry_no_bodies_and_count_words_by_the_shared_rule() {
    let app = desk().await;
    let project = project(&app).await;
    let (status, created) = json(
        &app,
        "POST",
        &format!("/projects/{project}/documents"),
        Some(json!({
            "kind": "scene",
            "title": "Chapter 2",
            "content_json": {"type":"doc","content":[
                paragraph("b1", "Mira tied back her auburn hair and waited for Oren."),
                paragraph("b2", "one\u{a0}two\u{2003}three")
            ]}
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let id = created["id"].as_str().unwrap().to_owned();

    let (status, summaries) = json(
        &app,
        "GET",
        &format!("/projects/{project}/documents?fields=summary"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let summary = &summaries.as_array().unwrap()[0];
    assert_eq!(summary["id"], created["id"]);
    assert!(summary.get("content_json").is_none(), "{summary}");
    // Ten words in the first paragraph; the NBSP and em space split the second into three.
    assert_eq!(summary["words"], 13);
    assert_eq!(summary["head_seq"], created["head_seq"]);
    assert_eq!(summary["chain_hash"], created["chain_hash"]);
    assert_eq!(summary["title"], "Chapter 2");

    // The plain list still carries bodies, for the clients that want them.
    let (status, full) = json(&app, "GET", &format!("/projects/{project}/documents"), None).await;
    assert_eq!(status, StatusCode::OK);
    assert!(full[0]["content_json"]["content"].is_array());

    // One body at a time, addressed through its project.
    let (status, document) = json(
        &app,
        "GET",
        &format!("/projects/{project}/documents/{id}"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(document["content_json"]["content"][1]["attrs"]["id"], "b2");

    let other = uuid::Uuid::now_v7();
    let (status, _) = json(
        &app,
        "GET",
        &format!("/projects/{other}/documents/{id}"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    let (status, _) = json(
        &app,
        "GET",
        &format!("/projects/{project}/documents?fields=everything"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

/// `cargo test -p asten-api --test documents -- --ignored --nocapture` prints
/// the bytes a 250k-word project costs per list, with and without bodies.
#[tokio::test]
#[ignore = "benchmark; run by hand with --nocapture"]
async fn bench_document_payloads() {
    let app = desk().await;
    let project = project(&app).await;
    let sentence = "Mira walked along the harbor wall while Oren counted the boats and the gulls argued over the nets. ";
    for chapter in 0..50 {
        let blocks: Vec<Value> = (0..50)
            .map(|index| paragraph(&format!("c{chapter}-{index}"), &sentence.repeat(5)))
            .collect();
        let (status, _) = json(
            &app,
            "POST",
            &format!("/projects/{project}/documents"),
            Some(json!({
                "kind": "chapter",
                "title": format!("Chapter {chapter}"),
                "position": chapter,
                "content_json": {"type":"doc","content":blocks}
            })),
        )
        .await;
        assert_eq!(status, StatusCode::CREATED);
    }
    let (status, full) = call(&app, "GET", &format!("/projects/{project}/documents"), None).await;
    assert_eq!(status, StatusCode::OK);
    let (status, summary) = call(
        &app,
        "GET",
        &format!("/projects/{project}/documents?fields=summary"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let summaries: Value = serde_json::from_slice(&summary).unwrap();
    let words: u64 = summaries
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["words"].as_u64().unwrap())
        .sum();
    println!(
        "GET /documents: {} bytes; GET /documents?fields=summary: {} bytes ({} documents, {words} words)",
        full.len(),
        summary.len(),
        summaries.as_array().unwrap().len()
    );
}
