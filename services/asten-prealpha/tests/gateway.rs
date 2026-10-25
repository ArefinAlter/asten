//! The hosted gateway against real sandboxes in a temporary directory.

use std::{net::SocketAddr, path::PathBuf, sync::Arc};

use asten_prealpha::{Config, Gateway, VISITOR_COOKIE};
use axum::{
    Router,
    body::Body,
    extract::ConnectInfo,
    http::{Request, StatusCode, header},
    response::Response,
};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tower::ServiceExt;

struct Harness {
    app: Router,
    gateway: Arc<Gateway>,
    dir: PathBuf,
}

impl Drop for Harness {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

async fn harness(tune: impl FnOnce(&mut Config)) -> Harness {
    let dir = std::env::temp_dir().join(format!(
        "asten-prealpha-test-{}-{}",
        std::process::id(),
        uuid_like()
    ));
    let mut config = Config::with_data_dir(&dir);
    tune(&mut config);
    let gateway = Gateway::open(config).await.expect("gateway opens");
    Harness {
        app: asten_prealpha::router(gateway.clone()),
        gateway,
        dir,
    }
}

fn uuid_like() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or_default();
    format!("{nanos:x}")
}

fn request(method: &str, path: &str, cookie: Option<&str>, body: Option<Value>) -> Request<Body> {
    request_from(method, path, cookie, body, "203.0.113.7")
}

fn request_from(
    method: &str,
    path: &str,
    cookie: Option<&str>,
    body: Option<Value>,
    ip: &str,
) -> Request<Body> {
    let mut builder = Request::builder()
        .method(method)
        .uri(path)
        .header(header::HOST, "asten.example.org");
    if let Some(cookie) = cookie {
        builder = builder.header(header::COOKIE, format!("{VISITOR_COOKIE}={cookie}"));
    }
    let mut request = match body {
        Some(body) => builder
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(body.to_string()))
            .expect("request"),
        None => builder.body(Body::empty()).expect("request"),
    };
    let peer: SocketAddr = format!("{ip}:40000").parse().expect("address");
    request.extensions_mut().insert(ConnectInfo(peer));
    request
}

async fn send(app: &Router, request: Request<Body>) -> Response {
    app.clone().oneshot(request).await.expect("infallible")
}

async fn json_of(response: Response) -> Value {
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("body")
        .to_bytes();
    serde_json::from_slice(&bytes).unwrap_or(Value::Null)
}

fn cookie_of(response: &Response) -> String {
    let set = response
        .headers()
        .get(header::SET_COOKIE)
        .and_then(|value| value.to_str().ok())
        .expect("a new visitor gets a cookie");
    assert!(set.contains("HttpOnly"));
    assert!(set.contains("SameSite=Strict"));
    set.split(';')
        .next()
        .and_then(|pair| pair.split_once('='))
        .map(|(_, value)| value.to_owned())
        .expect("cookie value")
}

#[tokio::test]
async fn a_new_visitor_is_a_signed_in_desk_of_their_own() {
    let h = harness(|_| {}).await;
    let first = send(&h.app, request("GET", "/api/v1/auth/me", None, None)).await;
    assert_eq!(first.status(), StatusCode::OK);
    let cookie = cookie_of(&first);
    let me = json_of(first).await;
    assert_eq!(me["kind"], "desk");
    // The same cookie stays the same visitor, and no new cookie is set.
    let again = send(
        &h.app,
        request("GET", "/api/v1/projects", Some(&cookie), None),
    )
    .await;
    assert_eq!(again.status(), StatusCode::OK);
    assert!(again.headers().get(header::SET_COOKIE).is_none());
}

#[tokio::test]
async fn visitors_do_not_see_each_other() {
    let h = harness(|_| {}).await;
    let a = send(&h.app, request("GET", "/api/v1/projects", None, None)).await;
    let a = cookie_of(&a);
    let b = send(
        &h.app,
        request_from("GET", "/api/v1/projects", None, None, "198.51.100.4"),
    )
    .await;
    let b = cookie_of(&b);
    let created = send(
        &h.app,
        request(
            "POST",
            "/api/v1/projects",
            Some(&a),
            Some(json!({ "name": "Visitor A", "kind": "novel", "description": "" })),
        ),
    )
    .await;
    assert!(created.status().is_success(), "{}", created.status());
    let mine =
        json_of(send(&h.app, request("GET", "/api/v1/projects", Some(&a), None)).await).await;
    let theirs =
        json_of(send(&h.app, request("GET", "/api/v1/projects", Some(&b), None)).await).await;
    assert_eq!(mine.as_array().map(Vec::len), Some(1));
    assert_eq!(theirs.as_array().map(Vec::len), Some(0));
}

#[tokio::test]
async fn cross_origin_and_cross_site_requests_are_refused() {
    let h = harness(|_| {}).await;
    let mut foreign = request(
        "POST",
        "/api/v1/projects",
        None,
        Some(json!({ "name": "x" })),
    );
    foreign.headers_mut().insert(
        header::ORIGIN,
        "https://zeus.example.org".parse().expect("value"),
    );
    assert_eq!(send(&h.app, foreign).await.status(), StatusCode::FORBIDDEN);
    let mut sibling = request("POST", "/svc/v1/reset", None, None);
    sibling
        .headers_mut()
        .insert("sec-fetch-site", "same-site".parse().expect("value"));
    assert_eq!(send(&h.app, sibling).await.status(), StatusCode::FORBIDDEN);
    let mut own = request("GET", "/api/v1/projects", None, None);
    own.headers_mut().insert(
        header::ORIGIN,
        "https://asten.example.org".parse().expect("value"),
    );
    own.headers_mut()
        .insert("sec-fetch-site", "same-origin".parse().expect("value"));
    assert_eq!(send(&h.app, own).await.status(), StatusCode::OK);
}

#[tokio::test]
async fn new_visitors_are_capped_overall_and_per_address() {
    let h = harness(|c| {
        c.max_visitors = 2;
        c.new_per_client_per_hour = 1;
    })
    .await;
    let ok = send(&h.app, request("GET", "/api/v1/projects", None, None)).await;
    assert_eq!(ok.status(), StatusCode::OK);
    let same_address = send(&h.app, request("GET", "/api/v1/projects", None, None)).await;
    assert_eq!(same_address.status(), StatusCode::TOO_MANY_REQUESTS);
    let other = send(
        &h.app,
        request_from("GET", "/api/v1/projects", None, None, "198.51.100.9"),
    )
    .await;
    assert_eq!(other.status(), StatusCode::OK);
    let full = send(
        &h.app,
        request_from("GET", "/api/v1/projects", None, None, "192.0.2.1"),
    )
    .await;
    assert_eq!(full.status(), StatusCode::SERVICE_UNAVAILABLE);
}

#[tokio::test]
async fn state_round_trips_within_its_limits() {
    let h = harness(|c| {
        c.max_state_value_bytes = 64;
        c.max_state_keys = 2;
    })
    .await;
    let first = send(&h.app, request("GET", "/svc/v1/state", None, None)).await;
    let cookie = cookie_of(&first);
    assert_eq!(json_of(first).await, json!({ "items": {} }));
    let put = send(
        &h.app,
        request(
            "PUT",
            "/svc/v1/state/asten-workbench:prefs:v1",
            Some(&cookie),
            Some(json!({ "theme": "dark" })),
        ),
    )
    .await;
    assert_eq!(put.status(), StatusCode::NO_CONTENT);
    let items =
        json_of(send(&h.app, request("GET", "/svc/v1/state", Some(&cookie), None)).await).await;
    assert_eq!(items["items"]["asten-workbench:prefs:v1"]["theme"], "dark");
    let big = send(
        &h.app,
        request(
            "PUT",
            "/svc/v1/state/big",
            Some(&cookie),
            Some(json!({ "text": "x".repeat(100) })),
        ),
    )
    .await;
    assert_eq!(big.status(), StatusCode::PAYLOAD_TOO_LARGE);
    let bad = send(
        &h.app,
        request(
            "PUT",
            "/svc/v1/state/claim:seed",
            Some(&cookie),
            Some(json!(1)),
        ),
    )
    .await;
    assert_eq!(bad.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        send(
            &h.app,
            request("PUT", "/svc/v1/state/two", Some(&cookie), Some(json!(2)))
        )
        .await
        .status(),
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        send(
            &h.app,
            request("PUT", "/svc/v1/state/three", Some(&cookie), Some(json!(3)))
        )
        .await
        .status(),
        StatusCode::INSUFFICIENT_STORAGE
    );
}

#[tokio::test]
async fn a_claim_is_first_only_once() {
    let h = harness(|_| {}).await;
    let first = send(
        &h.app,
        request("POST", "/svc/v1/claim/sample-novel", None, None),
    )
    .await;
    let cookie = cookie_of(&first);
    assert_eq!(json_of(first).await["first"], true);
    let second = send(
        &h.app,
        request("POST", "/svc/v1/claim/sample-novel", Some(&cookie), None),
    )
    .await;
    assert_eq!(json_of(second).await["first"], false);
}

#[tokio::test]
async fn signing_out_does_not_strand_the_visitor() {
    let h = harness(|_| {}).await;
    let first = send(&h.app, request("GET", "/api/v1/auth/me", None, None)).await;
    let cookie = cookie_of(&first);
    let out = send(
        &h.app,
        request("POST", "/api/v1/auth/logout", Some(&cookie), None),
    )
    .await;
    assert!(out.status().is_success(), "{}", out.status());
    let me = send(
        &h.app,
        request("GET", "/api/v1/auth/me", Some(&cookie), None),
    )
    .await;
    assert_eq!(me.status(), StatusCode::OK);
}

#[tokio::test]
async fn expired_visitors_are_deleted_with_their_sandbox() {
    let h = harness(|_| {}).await;
    let first = send(&h.app, request("GET", "/api/v1/projects", None, None)).await;
    let cookie = cookie_of(&first);
    let sandboxes = h.dir.join("sandboxes");
    let count = || {
        std::fs::read_dir(&sandboxes)
            .map(|d| d.count())
            .unwrap_or(0)
    };
    assert!(count() >= 1, "the sandbox file exists");
    let later = chrono::Utc::now().timestamp() + 25 * 3600;
    assert_eq!(h.gateway.sweep_at(later).await.expect("sweep"), 1);
    let db_files = std::fs::read_dir(&sandboxes)
        .map(|d| {
            d.filter_map(Result::ok)
                .filter(|e| e.file_name().to_string_lossy().ends_with(".db"))
                .count()
        })
        .unwrap_or(0);
    assert_eq!(db_files, 0, "the sandbox is gone");
    // The old cookie now names nobody: the next request starts a new visitor.
    let next = send(
        &h.app,
        request_from("GET", "/api/v1/projects", Some(&cookie), None, "192.0.2.50"),
    )
    .await;
    assert_eq!(next.status(), StatusCode::OK);
    assert!(next.headers().get(header::SET_COOKIE).is_some());
}

#[tokio::test]
async fn the_visitor_cookie_never_reaches_the_api_and_svc_is_not_cached() {
    let h = harness(|_| {}).await;
    let session = send(&h.app, request("GET", "/svc/v1/session", None, None)).await;
    assert_eq!(
        session
            .headers()
            .get(header::CACHE_CONTROL)
            .and_then(|v| v.to_str().ok()),
        Some("no-store")
    );
    let body = json_of(session).await;
    assert_eq!(body["limits"]["ttl_hours"], 24);
    assert!(
        body["served_by"]["browser_memory"][0]
            .as_str()
            .is_some_and(|s| s.contains("keys"))
    );
}

#[tokio::test]
async fn client_addresses_are_forgotten_after_an_hour() {
    let h = harness(|c| c.new_per_client_per_hour = 1).await;
    let first = send(&h.app, request("GET", "/api/v1/projects", None, None)).await;
    assert_eq!(first.status(), StatusCode::OK);
    let refused = send(&h.app, request("GET", "/api/v1/projects", None, None)).await;
    assert_eq!(refused.status(), StatusCode::TOO_MANY_REQUESTS);
    let later = chrono::Utc::now().timestamp() + 2 * 3600;
    h.gateway.sweep_at(later).await.expect("sweep");
    let store = h.dir.join("visitors.db");
    let kept = sqlx::SqlitePool::connect(&format!("sqlite://{}", store.display()))
        .await
        .expect("store");
    let addresses: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM visitors WHERE client <> ''")
        .fetch_one(&kept)
        .await
        .expect("count");
    assert_eq!(addresses, 0, "no client address outlives the hour");
}
