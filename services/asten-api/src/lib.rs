//! The loopback HTTP transport. Every client that is not the in-process desk
//! talks to this: the browser desk, the Word task pane, the Docs sidebar, and
//! the desktop shell's own webview. It carries the same application methods as
//! the Tauri commands, behind pairing-token authentication, an origin
//! allowlist, security headers, rate limits and a change stream.

mod auth;
mod extra;
mod headers;
mod history;
mod story;

use std::{net::IpAddr, net::SocketAddr, path::PathBuf, sync::Arc, time::Duration};

use asten_application::{
    Application, ApplicationError, AuthMode, CreateDocument, CreateFact, CreateProject,
    CreateWorldEntry, DeskInfo, UpdateDocument, UpdateFact, UpdateWorldEntry,
};
use asten_domain::{
    Document, DocumentId, Fact, FactId, Project, ProjectId, WorldEntry, WorldEntryId, WorldPulse,
};
use axum::{
    Extension, Json, Router,
    body::Body,
    extract::{DefaultBodyLimit, Path, Query, State},
    http::{HeaderName, HeaderValue, Method, Request, StatusCode, header},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::get,
};
use serde::{Deserialize, Serialize};
use tower_http::{
    catch_panic::CatchPanicLayer,
    compression::CompressionLayer,
    cors::{AllowOrigin, CorsLayer},
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    services::{ServeDir, ServeFile},
    timeout::TimeoutLayer,
    trace::TraceLayer,
};
use uuid::Uuid;

pub use auth::{AuthenticatedClient, ClientKey};

/// Runtime configuration, from the environment or the desktop shell.
#[derive(Clone, Debug)]
pub struct ApiConfig {
    pub auth_mode: AuthMode,
    /// Exact origins, or `https://*.example.com` suffix patterns.
    pub allowed_origins: Vec<String>,
    /// Serve the built desk from this directory at `/`.
    pub serve_desk: Option<PathBuf>,
    /// Mark the session cookie `Secure` (behind TLS).
    pub cookie_secure: bool,
    /// Reverse proxies whose `X-Forwarded-For` is believed, as addresses or
    /// CIDR blocks. Empty means the TCP peer is the client.
    pub trusted_proxies: Vec<IpNet>,
    /// Where the SQLite file lives, shown to desk and owner sessions.
    pub database_path: Option<String>,
    /// True when a desktop or mobile shell hosts this API in-process.
    pub embedded: bool,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            auth_mode: AuthMode::Pairing,
            allowed_origins: default_origins(),
            serve_desk: None,
            cookie_secure: false,
            trusted_proxies: Vec::new(),
            database_path: None,
            embedded: false,
        }
    }
}

/// An address or a CIDR block, for the trusted-proxy list.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IpNet {
    address: IpAddr,
    prefix: u8,
}

impl IpNet {
    /// Parse `10.0.0.5`, `10.0.0.0/8` or `fd00::/8`.
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        let (address, prefix) = match text.split_once('/') {
            Some((address, prefix)) => (address.trim(), Some(prefix.trim())),
            None => (text.trim(), None),
        };
        let address: IpAddr = address.parse().ok()?;
        let max = if address.is_ipv4() { 32 } else { 128 };
        let prefix = match prefix {
            Some(prefix) => prefix.parse::<u8>().ok().filter(|p| *p <= max)?,
            None => max,
        };
        Some(Self { address, prefix })
    }

    #[must_use]
    pub fn contains(&self, candidate: IpAddr) -> bool {
        fn bits(address: IpAddr) -> u128 {
            match address {
                IpAddr::V4(v4) => u128::from(u32::from(v4)),
                IpAddr::V6(v6) => u128::from(v6),
            }
        }
        if self.address.is_ipv4() != candidate.is_ipv4() {
            return false;
        }
        let width = if self.address.is_ipv4() { 32 } else { 128 };
        if self.prefix == 0 {
            return true;
        }
        let shift = width - u32::from(self.prefix);
        (bits(self.address) >> shift) == (bits(candidate) >> shift)
    }
}

impl ApiConfig {
    /// `ASTEN_AUTH`, `ASTEN_ALLOWED_ORIGINS`, `ASTEN_SERVE_DESK`, `ASTEN_COOKIE_SECURE`,
    /// `ASTEN_TRUSTED_PROXIES`.
    #[must_use]
    pub fn from_env() -> Self {
        let auth_mode = std::env::var("ASTEN_AUTH")
            .map(|value| AuthMode::parse(&value))
            .unwrap_or(AuthMode::Pairing);
        let mut allowed_origins = default_origins();
        if let Ok(extra) = std::env::var("ASTEN_ALLOWED_ORIGINS") {
            allowed_origins.extend(
                extra
                    .split(',')
                    .map(str::trim)
                    .filter(|origin| !origin.is_empty())
                    .map(str::to_owned),
            );
        }
        Self {
            auth_mode,
            allowed_origins,
            serve_desk: std::env::var("ASTEN_SERVE_DESK")
                .ok()
                .filter(|value| !value.is_empty())
                .map(PathBuf::from),
            cookie_secure: std::env::var("ASTEN_COOKIE_SECURE")
                .map(|value| value == "1" || value.eq_ignore_ascii_case("true"))
                .unwrap_or(false),
            trusted_proxies: std::env::var("ASTEN_TRUSTED_PROXIES")
                .map(|value| {
                    value
                        .split(',')
                        .filter_map(|item| {
                            let item = item.trim();
                            if item.is_empty() {
                                return None;
                            }
                            let parsed = IpNet::parse(item);
                            if parsed.is_none() {
                                tracing::warn!(%item, "ignoring an unparseable ASTEN_TRUSTED_PROXIES entry");
                            }
                            parsed
                        })
                        .collect()
                })
                .unwrap_or_default(),
            database_path: Some(
                std::env::var("ASTEN_DATABASE_PATH").unwrap_or_else(|_| "data/asten.db".into()),
            ),
            embedded: false,
        }
    }
}

/// Origins that may call the API from a browser by default: the dev desk, the
/// desktop webview, the Word task pane, and Google Apps Script sidebars.
#[must_use]
pub fn default_origins() -> Vec<String> {
    [
        "http://localhost:1420",
        "http://127.0.0.1:1420",
        "http://localhost:4317",
        "http://127.0.0.1:4317",
        "https://localhost:3000",
        "tauri://localhost",
        "http://tauri.localhost",
        "https://tauri.localhost",
        "https://*.googleusercontent.com",
    ]
    .iter()
    .map(|origin| (*origin).to_owned())
    .collect()
}

#[derive(Clone)]
pub struct ApiState {
    pub application: Application,
    pub config: Arc<ApiConfig>,
}

fn origin_allowed(config: &ApiConfig, origin: &str) -> bool {
    config
        .allowed_origins
        .iter()
        .any(|allowed| origin_matches(allowed, origin))
}

/// Exact match, or a one-level wildcard such as `https://*.googleusercontent.com`.
/// The wildcard stands for one or more labels and requires the dot, so
/// `https://evil-googleusercontent.com` and `https://googleusercontent.com`
/// do not match, and neither does a host with a port, a path or userinfo.
fn origin_matches(allowed: &str, origin: &str) -> bool {
    let Some((scheme, pattern_host)) = allowed.split_once("://") else {
        return allowed == origin;
    };
    let Some(suffix) = pattern_host.strip_prefix("*.") else {
        return allowed == origin;
    };
    let Some(host) = origin
        .strip_prefix(scheme)
        .and_then(|rest| rest.strip_prefix("://"))
    else {
        return false;
    };
    let Some(prefix) = host.strip_suffix(suffix).and_then(|p| p.strip_suffix('.')) else {
        return false;
    };
    !prefix.is_empty()
        && prefix
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '.')
        && !prefix.starts_with('.')
        && !prefix.contains("..")
}

pub fn router(application: Application, config: ApiConfig) -> Router {
    let config = Arc::new(config);
    let request_id = HeaderName::from_static("x-request-id");
    let cors_config = config.clone();
    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::predicate(move |origin: &HeaderValue, _| {
            origin
                .to_str()
                .is_ok_and(|origin| origin_allowed(&cors_config, origin))
        }))
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION, header::ACCEPT])
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_credentials(true)
        .max_age(Duration::from_secs(600));

    let state = ApiState {
        application,
        config: config.clone(),
    };

    let api = Router::new()
        .route("/desk", get(desk_info))
        .route("/projects", get(list_projects).post(create_project))
        .route("/projects/{project_id}", get(get_project))
        .route(
            "/projects/{project_id}/documents",
            get(list_documents).post(create_document),
        )
        .route(
            "/projects/{project_id}/documents/{document_id}",
            get(get_document)
                .put(update_document)
                .delete(delete_document),
        )
        .route(
            "/projects/{project_id}/facts",
            get(list_facts).post(create_fact),
        )
        .route(
            "/projects/{project_id}/facts/{fact_id}",
            axum::routing::put(update_fact).delete(delete_fact),
        )
        .route("/projects/{project_id}/diagnostics", get(list_diagnostics))
        .route(
            "/projects/{project_id}/world-entries",
            get(list_world_entries).post(create_world_entry),
        )
        .route(
            "/projects/{project_id}/world-entries/{entry_id}",
            axum::routing::put(update_world_entry),
        )
        .route("/projects/{project_id}/world-pulse", get(world_pulse))
        .merge(extra::router())
        .merge(history::router())
        .merge(story::router())
        .merge(auth::protected_router())
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth::require_auth,
        ))
        .merge(auth::public_router());

    let mut app = Router::new()
        .route("/health", get(health))
        .nest("/api/v1", api);

    if let Some(desk) = &config.serve_desk {
        // The hashed assets get a directory service of their own, so a missing
        // asset is a 404 rather than index.html: the browser and the service
        // worker would otherwise keep that page under the asset's name for a
        // year.
        let index = desk.join("index.html");
        app = app
            .nest_service("/assets", ServeDir::new(desk.join("assets")))
            .fallback_service(ServeDir::new(desk).not_found_service(ServeFile::new(index)));
    }

    app.with_state(state)
        .layer(DefaultBodyLimit::max(10 * 1024 * 1024))
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            Duration::from_secs(30),
        ))
        .layer(CompressionLayer::new())
        .layer(cors)
        .layer(middleware::from_fn_with_state(
            Arc::new(headers::SecurityPolicy::for_config(config.clone())),
            security_headers,
        ))
        .layer(middleware::from_fn_with_state(
            config.clone(),
            auth::client_key,
        ))
        .layer(PropagateRequestIdLayer::new(request_id.clone()))
        .layer(SetRequestIdLayer::new(request_id, MakeRequestUuid))
        // Spans name the path only: the stream route carries its token in the
        // query string, and that must never reach a log.
        .layer(
            TraceLayer::new_for_http().make_span_with(|request: &Request<Body>| {
                tracing::debug_span!(
                    "request",
                    method = %request.method(),
                    path = %request.uri().path(),
                )
            }),
        )
        .layer(CatchPanicLayer::new())
}

/// Security headers on every response: see [`headers::SecurityPolicy`]. The
/// private-network header is granted only to origins on the allowlist, so a
/// random page cannot reach loopback.
async fn security_headers(
    State(policy): State<Arc<headers::SecurityPolicy>>,
    request: Request<Body>,
    next: Next,
) -> Response {
    let origin = request
        .headers()
        .get(header::ORIGIN)
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned);
    let path = request.uri().path().to_owned();
    let mut response = next.run(request).await;
    policy.apply(&path, origin.as_deref(), &mut response);
    response
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: "asten-api",
        version: env!("CARGO_PKG_VERSION"),
    })
}

/// What the desk shows about the machine it runs on. The database path is
/// for desk and owner sessions; a guest learns only the version and host.
async fn desk_info(
    State(state): State<ApiState>,
    Extension(auth::AuthenticatedClient(client)): Extension<auth::AuthenticatedClient>,
) -> Json<DeskInfo> {
    let privileged = client.kind.can_administer();
    Json(DeskInfo {
        version: env!("CARGO_PKG_VERSION").to_owned(),
        host: format!("{} {}", std::env::consts::OS, std::env::consts::ARCH),
        embedded: state.config.embedded,
        auth_mode: state.config.auth_mode,
        database_path: if privileged {
            state.config.database_path.clone()
        } else {
            None
        },
    })
}

async fn list_projects(State(state): State<ApiState>) -> Result<Json<Vec<Project>>, ApiError> {
    Ok(Json(state.application.list_projects().await?))
}

async fn get_project(
    State(state): State<ApiState>,
    Path(project_id): Path<Uuid>,
) -> Result<Json<Project>, ApiError> {
    Ok(Json(
        state.application.get_project(ProjectId(project_id)).await?,
    ))
}

async fn create_project(
    State(state): State<ApiState>,
    Json(input): Json<CreateProject>,
) -> Result<(StatusCode, Json<Project>), ApiError> {
    let project = state.application.create_project(input).await?;
    Ok((StatusCode::CREATED, Json(project)))
}

/// `?fields=summary` lists documents without their bodies (plus a server-side
/// word count); the default carries everything.
#[derive(Debug, Default, Deserialize)]
struct ListDocumentsQuery {
    #[serde(default)]
    fields: Option<String>,
}

async fn list_documents(
    State(state): State<ApiState>,
    Path(project_id): Path<Uuid>,
    Query(query): Query<ListDocumentsQuery>,
) -> Result<Response, ApiError> {
    let project_id = ProjectId(project_id);
    match query.fields.as_deref() {
        Some("summary") => Ok(Json(
            state
                .application
                .list_document_summaries(project_id)
                .await?,
        )
        .into_response()),
        None | Some("full") => {
            Ok(Json(state.application.list_documents(project_id).await?).into_response())
        }
        Some(other) => Err(ApplicationError::Validation(format!(
            "unknown fields selection {other:?}; use \"summary\" or \"full\""
        ))
        .into()),
    }
}

async fn get_document(
    State(state): State<ApiState>,
    Path((project_id, document_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<Document>, ApiError> {
    Ok(Json(
        state
            .application
            .get_project_document(ProjectId(project_id), DocumentId(document_id))
            .await?,
    ))
}

async fn create_document(
    State(state): State<ApiState>,
    Path(project_id): Path<Uuid>,
    Json(mut input): Json<CreateDocument>,
) -> Result<(StatusCode, Json<Document>), ApiError> {
    input.project_id = ProjectId(project_id);
    let document = state.application.create_document(input).await?;
    Ok((StatusCode::CREATED, Json(document)))
}

async fn update_document(
    State(state): State<ApiState>,
    Path((_, document_id)): Path<(Uuid, Uuid)>,
    Json(input): Json<UpdateDocument>,
) -> Result<Json<Document>, ApiError> {
    Ok(Json(
        state
            .application
            .update_document(DocumentId(document_id), input)
            .await?,
    ))
}

async fn delete_document(
    State(state): State<ApiState>,
    Path((_, document_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    state
        .application
        .delete_document(DocumentId(document_id))
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn list_facts(
    State(state): State<ApiState>,
    Path(project_id): Path<Uuid>,
) -> Result<Json<Vec<Fact>>, ApiError> {
    Ok(Json(
        state.application.list_facts(ProjectId(project_id)).await?,
    ))
}

async fn create_fact(
    State(state): State<ApiState>,
    Path(project_id): Path<Uuid>,
    Json(mut input): Json<CreateFact>,
) -> Result<(StatusCode, Json<Fact>), ApiError> {
    input.project_id = ProjectId(project_id);
    let fact = state.application.create_fact(input).await?;
    Ok((StatusCode::CREATED, Json(fact)))
}

async fn update_fact(
    State(state): State<ApiState>,
    Path((_, fact_id)): Path<(Uuid, Uuid)>,
    Json(input): Json<UpdateFact>,
) -> Result<Json<Fact>, ApiError> {
    Ok(Json(
        state
            .application
            .update_fact(FactId(fact_id), input)
            .await?,
    ))
}

async fn delete_fact(
    State(state): State<ApiState>,
    Path((_, fact_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    state.application.delete_fact(FactId(fact_id)).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn list_diagnostics(
    State(state): State<ApiState>,
    Path(project_id): Path<Uuid>,
) -> Result<Json<asten_ls::DiagnosticsSnapshot>, ApiError> {
    Ok(Json(
        state
            .application
            .analyze_project(ProjectId(project_id))
            .await?,
    ))
}

async fn list_world_entries(
    State(state): State<ApiState>,
    Path(project_id): Path<Uuid>,
) -> Result<Json<Vec<WorldEntry>>, ApiError> {
    Ok(Json(
        state
            .application
            .list_world_entries(ProjectId(project_id))
            .await?,
    ))
}

async fn create_world_entry(
    State(state): State<ApiState>,
    Path(project_id): Path<Uuid>,
    Json(mut input): Json<CreateWorldEntry>,
) -> Result<(StatusCode, Json<WorldEntry>), ApiError> {
    input.project_id = ProjectId(project_id);
    let entry = state.application.create_world_entry(input).await?;
    Ok((StatusCode::CREATED, Json(entry)))
}

async fn update_world_entry(
    State(state): State<ApiState>,
    Path((_, entry_id)): Path<(Uuid, Uuid)>,
    Json(input): Json<UpdateWorldEntry>,
) -> Result<Json<WorldEntry>, ApiError> {
    Ok(Json(
        state
            .application
            .update_world_entry(WorldEntryId(entry_id), input)
            .await?,
    ))
}

async fn world_pulse(
    State(state): State<ApiState>,
    Path(project_id): Path<Uuid>,
) -> Result<Json<WorldPulse>, ApiError> {
    Ok(Json(
        state.application.world_pulse(ProjectId(project_id)).await?,
    ))
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    service: &'static str,
    version: &'static str,
}

pub struct ApiError(pub ApplicationError);

impl From<ApplicationError> for ApiError {
    fn from(value: ApplicationError) -> Self {
        Self(value)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = match self.0 {
            ApplicationError::NotFound { .. } => StatusCode::NOT_FOUND,
            ApplicationError::Validation(_) => StatusCode::UNPROCESSABLE_ENTITY,
            ApplicationError::Storage(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApplicationError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            ApplicationError::Conflict(_) => StatusCode::CONFLICT,
        };
        let message = match &self.0 {
            // Never leak storage internals to clients.
            ApplicationError::Storage(_) => {
                "the local store could not complete the request".to_owned()
            }
            other => other.to_string(),
        };
        let body = Json(ErrorResponse {
            error: ErrorBody {
                code: status.as_str().to_owned(),
                message,
            },
        });
        (status, body).into_response()
    }
}

#[derive(Serialize)]
struct ErrorResponse {
    error: ErrorBody,
}

#[derive(Serialize)]
struct ErrorBody {
    code: String,
    message: String,
}

pub async fn application_for_database(
    path: &str,
    auth_mode: AuthMode,
) -> anyhow::Result<Application> {
    let repository = asten_storage::SqliteRepository::open(path).await?;
    Ok(Application::with_auth(Arc::new(repository), auth_mode))
}

/// Bind the preferred address, falling back to an ephemeral port on the same host.
pub async fn bind_with_fallback(preferred: SocketAddr) -> anyhow::Result<tokio::net::TcpListener> {
    match tokio::net::TcpListener::bind(preferred).await {
        Ok(listener) => Ok(listener),
        Err(_) => {
            let fallback = SocketAddr::new(preferred.ip(), 0);
            Ok(tokio::net::TcpListener::bind(fallback).await?)
        }
    }
}

/// Serve until `shutdown` resolves.
pub async fn serve(
    listener: tokio::net::TcpListener,
    application: Application,
    config: ApiConfig,
    shutdown: impl std::future::Future<Output = ()> + Send + 'static,
) -> anyhow::Result<()> {
    let app = router(application, config);
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown)
    .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config_with(origins: &[&str]) -> ApiConfig {
        ApiConfig {
            allowed_origins: origins.iter().map(|o| (*o).to_owned()).collect(),
            ..ApiConfig::default()
        }
    }

    #[test]
    fn ip_nets_parse_and_match() {
        let v4 = IpNet::parse("172.16.0.0/12").unwrap();
        assert!(v4.contains("172.31.5.9".parse().unwrap()));
        assert!(!v4.contains("172.32.0.1".parse().unwrap()));
        assert!(!v4.contains("::1".parse().unwrap()));
        let host = IpNet::parse("10.0.0.5").unwrap();
        assert!(host.contains("10.0.0.5".parse().unwrap()));
        assert!(!host.contains("10.0.0.6".parse().unwrap()));
        let v6 = IpNet::parse("fd00::/8").unwrap();
        assert!(v6.contains("fd12:3456::1".parse().unwrap()));
        assert!(!v6.contains("fe80::1".parse().unwrap()));
        assert!(IpNet::parse("10.0.0.0/33").is_none());
        assert!(IpNet::parse("nonsense").is_none());
    }

    #[test]
    fn wildcard_origins_require_a_subdomain_and_the_dot() {
        let config = config_with(&["https://*.googleusercontent.com", "http://localhost:1420"]);
        assert!(origin_allowed(
            &config,
            "https://n-abc123.docs.googleusercontent.com"
        ));
        assert!(origin_allowed(
            &config,
            "https://script.googleusercontent.com"
        ));
        assert!(origin_allowed(&config, "http://localhost:1420"));
        assert!(!origin_allowed(&config, "https://googleusercontent.com"));
        assert!(!origin_allowed(
            &config,
            "https://evil-googleusercontent.com"
        ));
        assert!(!origin_allowed(
            &config,
            "https://x.googleusercontent.com.evil.net"
        ));
        assert!(!origin_allowed(
            &config,
            "https://evil.net/.googleusercontent.com"
        ));
        assert!(!origin_allowed(
            &config,
            "https://x.googleusercontent.com:8443"
        ));
        assert!(!origin_allowed(&config, "http://x.googleusercontent.com"));
        assert!(!origin_allowed(
            &config,
            "https://a@b.googleusercontent.com"
        ));
        assert!(!origin_allowed(&config, "http://localhost:14200"));
        assert!(!origin_allowed(&config, "null"));
    }
}
