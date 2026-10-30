//! The browser boundary, end to end: the headers the served desk gets, the
//! cache rules, and the ways a cookie, a ticket or a forwarded address could
//! be bent. Each test opens its own SQLite file and its own desk directory.

use std::{net::SocketAddr, path::PathBuf, sync::Arc};

use asten_api::{ApiConfig, IpNet, router};
use asten_application::{Application, AuthMode, CreateProject};
use asten_domain::{ProjectKind, TokenKind};
use asten_storage::SqliteRepository;
use axum::{
    Router,
    body::Body,
    extract::ConnectInfo,
    http::{Request, Response, StatusCode, header},
};
use base64::Engine as _;
use http_body_util::BodyExt;
use sha2::{Digest, Sha256};
use tower::ServiceExt;
use uuid::Uuid;

const HOST: &str = "localhost:4398";
const INLINE_SCRIPT: &str = "try{document.documentElement.dataset.theme='day'}catch(e){}";
const IMMUTABLE: &str = "public, max-age=31536000, immutable";

struct Desk {
    application: Application,
    router: Router,
    root: PathBuf,
    database: PathBuf,
}

impl Drop for Desk {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
        for suffix in ["", "-shm", "-wal"] {
            let mut path = self.database.clone().into_os_string();
            path.push(suffix);
            let _ = std::fs::remove_file(path);
        }
    }
}

async fn open_desk(adjust: impl FnOnce(&mut ApiConfig)) -> Desk {
    let id = Uuid::now_v7();
    let root = std::env::temp_dir().join(format!("asten-security-{id}"));
    std::fs::create_dir_all(root.join("assets")).unwrap();
    std::fs::write(
        root.join("index.html"),
        format!(
            "<!doctype html><html><head><script>{INLINE_SCRIPT}</script>\
             <script type=\"module\" crossorigin src=\"/assets/index-abc123.js\"></script>\
             </head><body><div id=\"root\"></div></body></html>"
        ),
    )
    .unwrap();
    std::fs::write(root.join("assets/index-abc123.js"), "console.log('desk')").unwrap();
    std::fs::write(
        root.join("sw.js"),
        "self.addEventListener('fetch', () => {})",
    )
    .unwrap();
    // The database lives outside the served directory, as it must.
    let database = std::env::temp_dir().join(format!("asten-security-{id}.db"));
    let repository = SqliteRepository::open(&database).await.unwrap();
    let application = Application::with_auth(Arc::new(repository), AuthMode::Pairing);
    let mut config = ApiConfig {
        serve_desk: Some(root.clone()),
        database_path: Some(database.display().to_string()),
        ..ApiConfig::default()
    };
    adjust(&mut config);
    let router = router(application.clone(), config);
    Desk {
        application,
        router,
        root,
        database,
    }
}

async fn send(router: &Router, request: Request<Body>) -> Response<Body> {
    router.clone().oneshot(request).await.unwrap()
}

async fn text(response: Response<Body>) -> String {
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    String::from_utf8_lossy(&bytes).into_owned()
}

fn get(path: &str) -> Request<Body> {
    Request::builder().uri(path).body(Body::empty()).unwrap()
}

fn value<'a>(response: &'a Response<Body>, name: &str) -> &'a str {
    response
        .headers()
        .get(name)
        .map(|value| value.to_str().unwrap())
        .unwrap_or("")
}

fn json_post(path: &str, body: serde_json::Value) -> axum::http::request::Builder {
    let _ = body;
    Request::builder()
        .method("POST")
        .uri(path)
        .header(header::CONTENT_TYPE, "application/json")
}

/// Pair a guest from the page the API serves. Returns the session token the
/// cookie carries; the test never prints it.
async fn pair_from_the_desk(desk: &Desk) -> String {
    let start = desk
        .application
        .start_pairing("phone", TokenKind::Guest)
        .await
        .unwrap();
    let body = serde_json::json!({ "code": start.code, "label": "phone" });
    let request = json_post("/api/v1/auth/pair", body.clone())
        .header(header::HOST, HOST)
        .header(header::ORIGIN, format!("http://{HOST}"))
        .body(Body::from(body.to_string()))
        .unwrap();
    let response = send(&desk.router, request).await;
    assert_eq!(response.status(), StatusCode::CREATED);
    let set_cookie = value(&response, "set-cookie").to_owned();
    assert!(set_cookie.contains("HttpOnly"), "{set_cookie}");
    assert!(set_cookie.contains("SameSite=Strict"), "{set_cookie}");
    assert!(set_cookie.contains("Path=/"), "{set_cookie}");
    set_cookie
        .split(';')
        .next()
        .and_then(|pair| pair.strip_prefix("asten_session="))
        .unwrap()
        .to_owned()
}

async fn new_project(desk: &Desk) -> Uuid {
    desk.application
        .create_project(CreateProject {
            name: "A novel".into(),
            kind: ProjectKind::Novel,
            description: None,
        })
        .await
        .unwrap()
        .id
        .0
}

async fn stream_ticket(desk: &Desk, token: &str, project: Uuid) -> String {
    let request = Request::builder()
        .method("POST")
        .uri(format!("/api/v1/projects/{project}/stream-ticket"))
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();
    let response = send(&desk.router, request).await;
    assert_eq!(response.status(), StatusCode::CREATED);
    let body: serde_json::Value = serde_json::from_str(&text(response).await).unwrap();
    body["ticket"].as_str().unwrap().to_owned()
}

// ───────────────────────── headers and caching ─────────────────────────

#[tokio::test]
async fn the_served_desk_gets_a_policy_that_names_its_inline_script() {
    let desk = open_desk(|_| {}).await;
    let response = send(&desk.router, get("/")).await;
    assert_eq!(response.status(), StatusCode::OK);
    let csp = value(&response, "content-security-policy").to_owned();
    let hash =
        base64::engine::general_purpose::STANDARD.encode(Sha256::digest(INLINE_SCRIPT.as_bytes()));
    assert!(
        csp.contains(&format!("script-src 'self' 'sha256-{hash}'")),
        "{csp}"
    );
    assert!(csp.starts_with("default-src 'self'; "), "{csp}");
    assert!(csp.contains("connect-src 'self'"), "{csp}");
    assert!(csp.contains("img-src 'self' data: blob:"), "{csp}");
    assert!(csp.contains("font-src 'self' data:"), "{csp}");
    assert!(csp.contains("frame-ancestors 'none'"), "{csp}");
    assert!(csp.contains("object-src 'none'"), "{csp}");
    assert!(!csp.contains("script-src 'self' 'unsafe-inline'"), "{csp}");
    assert_eq!(value(&response, "cache-control"), "no-cache");
    assert_eq!(value(&response, "x-frame-options"), "DENY");
    assert_eq!(value(&response, "x-content-type-options"), "nosniff");
    assert_eq!(value(&response, "referrer-policy"), "no-referrer");
    assert!(
        response
            .headers()
            .get("strict-transport-security")
            .is_none()
    );
    // The desk routes by hash, so a path it never wrote is not found. The
    // answer is still the shell, under the same policy, and is not cached.
    let unknown = send(&desk.router, get("/projects/anything")).await;
    assert_eq!(unknown.status(), StatusCode::NOT_FOUND);
    assert_eq!(value(&unknown, "content-security-policy"), csp);
    assert_eq!(value(&unknown, "cache-control"), "no-cache");
    assert!(value(&unknown, "content-type").starts_with("text/html"));
}

#[tokio::test]
async fn hashed_assets_live_for_a_year_and_missing_ones_are_404() {
    let desk = open_desk(|_| {}).await;
    let asset = send(&desk.router, get("/assets/index-abc123.js")).await;
    assert_eq!(asset.status(), StatusCode::OK);
    assert_eq!(value(&asset, "cache-control"), IMMUTABLE);
    assert_eq!(value(&asset, "x-content-type-options"), "nosniff");
    // Not index.html with a 200, which the service worker would keep forever.
    let missing = send(&desk.router, get("/assets/index-gone.js")).await;
    assert_eq!(missing.status(), StatusCode::NOT_FOUND);
    assert_ne!(value(&missing, "cache-control"), IMMUTABLE);
    // The worker and the manifest are revalidated on every load.
    let worker = send(&desk.router, get("/sw.js")).await;
    assert_eq!(worker.status(), StatusCode::OK);
    assert_eq!(value(&worker, "cache-control"), "no-cache");
}

#[tokio::test]
async fn api_responses_stay_out_of_caches_and_out_of_frames() {
    let desk = open_desk(|_| {}).await;
    for path in ["/health", "/api/v1/auth/status"] {
        let response = send(&desk.router, get(path)).await;
        assert_eq!(response.status(), StatusCode::OK, "{path}");
        assert_eq!(value(&response, "cache-control"), "no-store", "{path}");
        assert_eq!(
            value(&response, "content-security-policy"),
            "default-src 'none'; frame-ancestors 'none'",
            "{path}"
        );
        assert_eq!(value(&response, "x-frame-options"), "DENY", "{path}");
    }
    // Without a desk to serve, nothing is cacheable at all.
    let bare = open_desk(|config| config.serve_desk = None).await;
    let response = send(&bare.router, get("/anything")).await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    assert_eq!(value(&response, "cache-control"), "no-store");
}

#[tokio::test]
async fn hsts_is_sent_only_behind_tls() {
    let plain = open_desk(|_| {}).await;
    let response = send(&plain.router, get("/health")).await;
    assert!(
        response
            .headers()
            .get("strict-transport-security")
            .is_none()
    );
    let secure = open_desk(|config| config.cookie_secure = true).await;
    for path in ["/health", "/", "/assets/index-abc123.js"] {
        let response = send(&secure.router, get(path)).await;
        assert_eq!(
            value(&response, "strict-transport-security"),
            "max-age=31536000",
            "{path}"
        );
    }
}

// ───────────────────────── cookies and origins ─────────────────────────

#[tokio::test]
async fn a_cookie_is_honoured_only_from_the_desks_own_origin() {
    let desk = open_desk(|_| {}).await;
    let token = pair_from_the_desk(&desk).await;
    let logout = |origin: Option<&str>| {
        let mut builder = Request::builder()
            .method("POST")
            .uri("/api/v1/auth/logout")
            .header(header::HOST, HOST)
            .header(header::COOKIE, format!("asten_session={token}"));
        if let Some(origin) = origin {
            builder = builder.header(header::ORIGIN, origin);
        }
        builder.body(Body::empty()).unwrap()
    };
    // A page on another port of the same site: the browser would attach the
    // SameSite=Strict cookie to its form post, and the API must not act on it.
    let forged = send(&desk.router, logout(Some("http://localhost:8080"))).await;
    assert_eq!(forged.status(), StatusCode::UNAUTHORIZED);
    assert!(text(forged).await.contains("own origin"));
    // The token is untouched: the desk itself still gets in with the cookie.
    let me = Request::builder()
        .uri("/api/v1/auth/me")
        .header(header::HOST, HOST)
        .header(header::COOKIE, format!("asten_session={token}"))
        .body(Body::empty())
        .unwrap();
    assert_eq!(send(&desk.router, me).await.status(), StatusCode::OK);
    // A bearer header is nothing a browser adds on its own, so it works from
    // wherever CORS lets the response through.
    let bearer = Request::builder()
        .uri("/api/v1/auth/me")
        .header(header::HOST, HOST)
        .header(header::ORIGIN, "http://localhost:8080")
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();
    assert_eq!(send(&desk.router, bearer).await.status(), StatusCode::OK);
    // From the desk's own origin the cookie is a credential.
    let own = send(&desk.router, logout(Some(&format!("http://{HOST}")))).await;
    assert_eq!(own.status(), StatusCode::NO_CONTENT);
    assert!(value(&own, "set-cookie").contains("Max-Age=0"));
}

#[tokio::test]
async fn pairing_from_another_origin_gets_a_token_but_no_cookie() {
    let desk = open_desk(|_| {}).await;
    let start = desk
        .application
        .start_pairing("pane", TokenKind::Guest)
        .await
        .unwrap();
    let body = serde_json::json!({ "code": start.code });
    let request = json_post("/api/v1/auth/pair", body.clone())
        .header(header::HOST, HOST)
        .header(header::ORIGIN, "https://localhost:3000")
        .body(Body::from(body.to_string()))
        .unwrap();
    let response = send(&desk.router, request).await;
    assert_eq!(response.status(), StatusCode::CREATED);
    assert!(response.headers().get("set-cookie").is_none());
    let paired: serde_json::Value = serde_json::from_str(&text(response).await).unwrap();
    assert_eq!(paired["cookie"], false);
    assert_eq!(paired["kind"], "guest");
    assert!(paired["token"].as_str().unwrap().starts_with("ast_"));
}

#[tokio::test]
async fn a_cookie_jar_split_over_two_headers_is_still_read() {
    let desk = open_desk(|_| {}).await;
    let token = pair_from_the_desk(&desk).await;
    let request = Request::builder()
        .uri("/api/v1/auth/me")
        .header(header::HOST, HOST)
        .header(header::COOKIE, "theme=night")
        .header(header::COOKIE, format!("asten_session={token}"))
        .body(Body::empty())
        .unwrap();
    assert_eq!(send(&desk.router, request).await.status(), StatusCode::OK);
}

// ───────────────────────── tickets and tokens ─────────────────────────

#[tokio::test]
async fn a_stream_ticket_is_bound_to_its_project_and_dies_with_its_token() {
    let desk = open_desk(|_| {}).await;
    let issued = desk
        .application
        .issue_desk_token("test desk")
        .await
        .unwrap();
    let project = new_project(&desk).await;
    let other = new_project(&desk).await;
    // Spent on the wrong project: refused, and gone.
    let first = stream_ticket(&desk, &issued.token, project).await;
    let wrong = get(&format!("/api/v1/projects/{other}/stream?ticket={first}"));
    assert_eq!(
        send(&desk.router, wrong).await.status(),
        StatusCode::UNAUTHORIZED
    );
    let replay = get(&format!("/api/v1/projects/{project}/stream?ticket={first}"));
    assert_eq!(
        send(&desk.router, replay).await.status(),
        StatusCode::UNAUTHORIZED
    );
    // The right project: the stream opens, and the ticket is spent.
    let second = stream_ticket(&desk, &issued.token, project).await;
    let open = send(
        &desk.router,
        get(&format!(
            "/api/v1/projects/{project}/stream?ticket={second}"
        )),
    )
    .await;
    assert_eq!(open.status(), StatusCode::OK);
    assert!(value(&open, "content-type").starts_with("text/event-stream"));
    assert_eq!(value(&open, "cache-control"), "no-store");
    drop(open);
    let again = get(&format!(
        "/api/v1/projects/{project}/stream?ticket={second}"
    ));
    assert_eq!(
        send(&desk.router, again).await.status(),
        StatusCode::UNAUTHORIZED
    );
    // Revoked between asking and connecting: nothing.
    let third = stream_ticket(&desk, &issued.token, project).await;
    desk.application
        .revoke_token(issued.token_id)
        .await
        .unwrap();
    let dead = get(&format!("/api/v1/projects/{project}/stream?ticket={third}"));
    assert_eq!(
        send(&desk.router, dead).await.status(),
        StatusCode::UNAUTHORIZED
    );
    // A ticket is not a bearer token, and a bearer token is not a ticket.
    let as_bearer = Request::builder()
        .uri("/api/v1/auth/me")
        .header(header::AUTHORIZATION, format!("Bearer {third}"))
        .body(Body::empty())
        .unwrap();
    assert_eq!(
        send(&desk.router, as_bearer).await.status(),
        StatusCode::UNAUTHORIZED
    );
}

#[tokio::test]
async fn a_guest_is_not_told_where_the_database_lives() {
    let desk = open_desk(|_| {}).await;
    let guest = pair_from_the_desk(&desk).await;
    let request = Request::builder()
        .uri("/api/v1/desk")
        .header(header::AUTHORIZATION, format!("Bearer {guest}"))
        .body(Body::empty())
        .unwrap();
    let response = send(&desk.router, request).await;
    assert_eq!(response.status(), StatusCode::OK);
    let info: serde_json::Value = serde_json::from_str(&text(response).await).unwrap();
    assert!(info["database_path"].is_null(), "{info}");
    let admin = desk.application.issue_desk_token("desk").await.unwrap();
    let request = Request::builder()
        .uri("/api/v1/desk")
        .header(header::AUTHORIZATION, format!("Bearer {}", admin.token))
        .body(Body::empty())
        .unwrap();
    let info: serde_json::Value =
        serde_json::from_str(&text(send(&desk.router, request).await).await).unwrap();
    assert!(info["database_path"].is_string(), "{info}");
    // And a guest may not reach the admin routes at all.
    let request = Request::builder()
        .uri("/api/v1/auth/tokens")
        .header(header::AUTHORIZATION, format!("Bearer {guest}"))
        .body(Body::empty())
        .unwrap();
    assert_eq!(
        send(&desk.router, request).await.status(),
        StatusCode::FORBIDDEN
    );
}

#[tokio::test]
async fn pairing_labels_are_bounded_and_a_refused_label_spends_nothing() {
    let desk = open_desk(|_| {}).await;
    let start = desk
        .application
        .start_pairing("", TokenKind::Guest)
        .await
        .unwrap();
    let long = serde_json::json!({ "code": start.code, "label": "x".repeat(81) });
    let request = json_post("/api/v1/auth/pair", long.clone())
        .body(Body::from(long.to_string()))
        .unwrap();
    assert_eq!(
        send(&desk.router, request).await.status(),
        StatusCode::UNPROCESSABLE_ENTITY
    );
    let fine = serde_json::json!({ "code": start.code, "label": "x".repeat(80) });
    let request = json_post("/api/v1/auth/pair", fine.clone())
        .body(Body::from(fine.to_string()))
        .unwrap();
    assert_eq!(
        send(&desk.router, request).await.status(),
        StatusCode::CREATED
    );
}

// ───────────────────────── rate limiting behind a proxy ─────────────────────────

#[tokio::test]
async fn a_client_behind_a_trusted_proxy_cannot_reset_its_limit_by_forging_hops() {
    let desk = open_desk(|config| {
        config.trusted_proxies = vec![IpNet::parse("127.0.0.1").unwrap()];
    })
    .await;
    let proxy: SocketAddr = "127.0.0.1:50000".parse().unwrap();
    let mut messages = Vec::new();
    for i in 0..11 {
        let body = serde_json::json!({ "code": "000000" });
        let mut request = json_post("/api/v1/auth/pair", body.clone())
            // The client's own, rotating, first hop; the proxy appended the real one.
            .header("x-forwarded-for", format!("10.0.0.{i}, 203.0.113.9"))
            .body(Body::from(body.to_string()))
            .unwrap();
        request.extensions_mut().insert(ConnectInfo(proxy));
        let response = send(&desk.router, request).await;
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        messages.push(text(response).await);
    }
    assert!(
        messages[..10].iter().all(|m| m.contains("not valid")),
        "{messages:?}"
    );
    assert!(messages[10].contains("too many attempts"), "{messages:?}");
}

#[tokio::test]
async fn repeated_forwarding_fields_do_not_let_a_client_reset_its_limit() {
    let desk = open_desk(|config| {
        config.trusted_proxies = vec![IpNet::parse("127.0.0.1").unwrap()];
    })
    .await;
    let proxy: SocketAddr = "127.0.0.1:50000".parse().unwrap();
    let mut last = String::new();
    for i in 0..11 {
        let body = serde_json::json!({ "code": "000000" });
        let mut request = json_post("/api/v1/auth/pair", body.clone())
            .header("x-forwarded-for", format!("10.0.0.{i}"))
            .header("x-forwarded-for", "203.0.113.9")
            .body(Body::from(body.to_string()))
            .unwrap();
        request.extensions_mut().insert(ConnectInfo(proxy));
        let response = send(&desk.router, request).await;
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        last = text(response).await;
        if i < 10 {
            assert!(last.contains("not valid"), "{last}");
        }
    }
    assert!(last.contains("too many attempts"), "{last}");
}

#[tokio::test]
async fn mcp_http_routes_require_pairing_authentication() {
    let desk = open_desk(|_| {}).await;
    for path in ["/mcp", "/api/v1/mcp"] {
        for authorization in [None, Some("Bearer invalid-audit-token")] {
            let mut request = Request::builder()
                .method("POST")
                .uri(path)
                .header(header::CONTENT_TYPE, "application/json");
            if let Some(value) = authorization {
                request = request.header(header::AUTHORIZATION, value);
            }
            let response = send(
                &desk.router,
                request
                    .body(Body::from(
                        r#"{"jsonrpc":"2.0","id":1,"method":"initialize"}"#,
                    ))
                    .unwrap(),
            )
            .await;
            assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        }
        let token = pair_from_the_desk(&desk).await;
        let response = send(
            &desk.router,
            Request::builder()
                .method("POST")
                .uri(path)
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, format!("Bearer {token}"))
                .body(Body::from(
                    r#"{"jsonrpc":"2.0","id":1,"method":"initialize"}"#,
                ))
                .unwrap(),
        )
        .await;
        assert_eq!(response.status(), StatusCode::OK);
    }
}
