//! Hosting for the public pre-alpha of the Asten workbench.
//!
//! Every anonymous visitor gets a sandbox of their own: a separate SQLite
//! file behind a separate, real Asten API ([`asten_api::router`]) in pairing
//! mode. The gateway hosts each API in-process the way the desktop shell does:
//! it revokes the sandbox's earlier desk tokens, issues a fresh one, and
//! presents it on the visitor's behalf. The browser only ever holds an opaque
//! `asten_visitor` cookie; it never sees the token.
//!
//! Beside the API it keeps a small store (`/svc/v1/…`) for what the API does
//! not hold: workbench preferences, assistant threads, console history and
//! layout. Provider keys are never sent here; the desk keeps them in memory.
//!
//! Limits: a visitor and everything they made are deleted 24 hours after the
//! first visit; new visitors per client address per hour, visitors at once,
//! the size of a sandbox and of the stored state are all capped. A request
//! whose `Origin` is not this host, or that a browser marks as cross-site, is
//! refused before it reaches a sandbox (the API's own cookie rule, D24, applied
//! to the visitor cookie).

use std::{
    collections::HashMap,
    net::{IpAddr, SocketAddr},
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicI64, Ordering},
    },
    time::Duration,
};

use asten_api::{ApiConfig, IpNet};
use asten_application::{Application, AuthMode};
use axum::{
    Json, Router,
    body::{Body, Bytes},
    extract::{ConnectInfo, Path as UrlPath, State},
    http::{HeaderMap, HeaderValue, Method, Request, StatusCode, header},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{any, get, post, put},
};
use base64::Engine as _;
use rand::RngCore;
use serde::Serialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sqlx::{
    Row, SqlitePool,
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions},
};
use tokio::sync::Mutex;
use tower::ServiceExt;
use tower_http::services::{ServeDir, ServeFile};

/// The cookie that names a visitor. Its value is 32 random bytes; only their
/// SHA-256 is stored.
pub const VISITOR_COOKIE: &str = "asten_visitor";

const BODY_LIMIT: usize = 10 * 1024 * 1024;
const TOKEN_LABEL: &str = "Hosted desk";

/// Limits and paths. Every number has an environment variable (see
/// [`Config::from_env`]) and a default fitted to a shared host.
#[derive(Clone, Debug)]
pub struct Config {
    /// Holds `visitors.db` and `sandboxes/`.
    pub data_dir: PathBuf,
    /// A built desk (`apps/web` in `--mode hosted`), served at `/`.
    pub desk_dir: Option<PathBuf>,
    /// How long a visitor lives, counted from the first visit.
    pub ttl_seconds: i64,
    /// Visitors alive at once; a new one beyond this is turned away.
    pub max_visitors: i64,
    /// Sandboxes held open in memory; the least recently used closes first.
    pub max_loaded: usize,
    /// New visitors per client address per hour.
    pub new_per_client_per_hour: i64,
    /// Size of a sandbox database (with its write-ahead log) beyond which
    /// writes are refused.
    pub max_sandbox_bytes: u64,
    /// One stored state value, serialised.
    pub max_state_value_bytes: usize,
    /// All of a visitor's state values together.
    pub max_state_total_bytes: usize,
    /// Distinct state keys per visitor.
    pub max_state_keys: i64,
    /// Close a sandbox that has not been used for this long.
    pub idle_seconds: i64,
    /// Mark the visitor cookie `Secure` (behind TLS).
    pub cookie_secure: bool,
    /// Proxies whose `X-Forwarded-For` is believed for the per-client limit.
    pub trusted_proxies: Vec<IpNet>,
}

impl Config {
    /// Defaults for a directory, for tests and as the base of [`Config::from_env`].
    #[must_use]
    pub fn with_data_dir(data_dir: impl Into<PathBuf>) -> Self {
        Self {
            data_dir: data_dir.into(),
            desk_dir: None,
            ttl_seconds: 24 * 60 * 60,
            max_visitors: 400,
            max_loaded: 48,
            new_per_client_per_hour: 20,
            max_sandbox_bytes: 48 * 1024 * 1024,
            max_state_value_bytes: 512 * 1024,
            max_state_total_bytes: 4 * 1024 * 1024,
            max_state_keys: 64,
            idle_seconds: 15 * 60,
            cookie_secure: false,
            trusted_proxies: Vec::new(),
        }
    }

    /// `PREALPHA_DATA`, `PREALPHA_DESK`, `PREALPHA_TTL_HOURS`,
    /// `PREALPHA_MAX_VISITORS`, `PREALPHA_MAX_LOADED`,
    /// `PREALPHA_NEW_PER_CLIENT_PER_HOUR`, `PREALPHA_MAX_SANDBOX_MB`,
    /// `PREALPHA_COOKIE_SECURE`, `PREALPHA_TRUSTED_PROXIES`.
    #[must_use]
    pub fn from_env() -> Self {
        fn num<T: std::str::FromStr>(name: &str) -> Option<T> {
            std::env::var(name)
                .ok()
                .and_then(|value| value.trim().parse().ok())
        }
        let mut config =
            Self::with_data_dir(std::env::var("PREALPHA_DATA").unwrap_or_else(|_| "data".into()));
        config.desk_dir = std::env::var("PREALPHA_DESK")
            .ok()
            .filter(|value| !value.is_empty())
            .map(PathBuf::from);
        if let Some(hours) = num::<i64>("PREALPHA_TTL_HOURS") {
            config.ttl_seconds = hours.max(1) * 3600;
        }
        if let Some(value) = num("PREALPHA_MAX_VISITORS") {
            config.max_visitors = value;
        }
        if let Some(value) = num("PREALPHA_MAX_LOADED") {
            config.max_loaded = value;
        }
        if let Some(value) = num("PREALPHA_NEW_PER_CLIENT_PER_HOUR") {
            config.new_per_client_per_hour = value;
        }
        if let Some(megabytes) = num::<u64>("PREALPHA_MAX_SANDBOX_MB") {
            config.max_sandbox_bytes = megabytes * 1024 * 1024;
        }
        config.cookie_secure = std::env::var("PREALPHA_COOKIE_SECURE")
            .map(|value| value == "1" || value.eq_ignore_ascii_case("true"))
            .unwrap_or(false);
        config.trusted_proxies = std::env::var("PREALPHA_TRUSTED_PROXIES")
            .map(|value| value.split(',').filter_map(IpNet::parse).collect())
            .unwrap_or_default();
        config
    }

    fn sandboxes(&self) -> PathBuf {
        self.data_dir.join("sandboxes")
    }
}

/// One visitor's API, open in memory.
struct Sandbox {
    application: Application,
    /// The sandbox's own pool, closed before its files are deleted.
    pool: SqlitePool,
    router: Router,
    token: std::sync::Mutex<String>,
    last_used: AtomicI64,
}

impl Sandbox {
    fn token(&self) -> String {
        self.token
            .lock()
            .map(|token| token.clone())
            .unwrap_or_default()
    }
}

/// The gateway: visitors, their sandboxes and their stored state.
pub struct Gateway {
    config: Arc<Config>,
    store: SqlitePool,
    loaded: Mutex<HashMap<String, Arc<Sandbox>>>,
}

fn now() -> i64 {
    chrono::Utc::now().timestamp()
}

fn rfc3339(seconds: i64) -> String {
    chrono::DateTime::from_timestamp(seconds, 0)
        .unwrap_or_default()
        .to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}

fn error(status: StatusCode, message: &str) -> Response {
    let body = json!({ "error": { "code": status.as_u16().to_string(), "message": message } });
    (status, Json(body)).into_response()
}

/// [`error`], boxed: a refusal travels in a `Result` without making every
/// `Ok` as large as a response.
fn refuse(status: StatusCode, message: &str) -> Box<Response> {
    Box::new(error(status, message))
}

/// A visitor id: the hex SHA-256 of the cookie value.
fn visitor_id(cookie: &str) -> String {
    hex::encode(Sha256::digest(cookie.as_bytes()))
}

fn valid_cookie_value(value: &str) -> bool {
    value.len() == 43
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

fn read_cookie(headers: &HeaderMap) -> Option<String> {
    headers
        .get_all(header::COOKIE)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .flat_map(|value| value.split(';'))
        .filter_map(|pair| pair.trim().split_once('='))
        .find(|(name, _)| *name == VISITOR_COOKIE)
        .map(|(_, value)| value.trim().to_owned())
        .filter(|value| valid_cookie_value(value))
}

/// Every cookie but the visitor's, so the API never sees it.
fn strip_visitor_cookie(headers: &mut HeaderMap) {
    let kept: Vec<String> = headers
        .get_all(header::COOKIE)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .flat_map(|value| value.split(';'))
        .map(str::trim)
        .filter(|pair| !pair.is_empty())
        .filter(|pair| {
            pair.split_once('=')
                .is_none_or(|(name, _)| name != VISITOR_COOKIE)
        })
        .map(str::to_owned)
        .collect();
    headers.remove(header::COOKIE);
    if !kept.is_empty()
        && let Ok(value) = HeaderValue::from_str(&kept.join("; "))
    {
        headers.insert(header::COOKIE, value);
    }
}

/// Refuse what a browser sends on another site's behalf: an `Origin` that is
/// not this host, or a fetch it labels cross-site or same-site. A sibling
/// subdomain is "same-site" to a browser, so `SameSite=Strict` alone would let
/// it ride the visitor cookie.
fn foreign(headers: &HeaderMap) -> bool {
    if let Some(site) = headers
        .get("sec-fetch-site")
        .and_then(|value| value.to_str().ok())
        && site != "same-origin"
        && site != "none"
    {
        return true;
    }
    let Some(origin) = headers.get(header::ORIGIN).and_then(|v| v.to_str().ok()) else {
        return false;
    };
    let host = headers
        .get(header::HOST)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();
    let named = origin
        .strip_prefix("https://")
        .or_else(|| origin.strip_prefix("http://"))
        .unwrap_or("");
    host.is_empty() || !named.eq_ignore_ascii_case(host)
}

impl Gateway {
    /// Open (or create) the store under `config.data_dir`.
    ///
    /// # Errors
    /// The directory or the store cannot be opened.
    pub async fn open(config: Config) -> anyhow::Result<Arc<Self>> {
        tokio::fs::create_dir_all(config.sandboxes()).await?;
        let options = SqliteConnectOptions::new()
            .filename(config.data_dir.join("visitors.db"))
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal)
            .busy_timeout(Duration::from_secs(5));
        let store = SqlitePoolOptions::new()
            .max_connections(4)
            .connect_with(options)
            .await?;
        for statement in [
            "CREATE TABLE IF NOT EXISTS visitors (id TEXT PRIMARY KEY, created_at INTEGER NOT NULL, last_seen INTEGER NOT NULL, client TEXT NOT NULL)",
            "CREATE INDEX IF NOT EXISTS visitors_created ON visitors (created_at)",
            "CREATE INDEX IF NOT EXISTS visitors_client ON visitors (client, created_at)",
            "CREATE TABLE IF NOT EXISTS state (visitor TEXT NOT NULL, key TEXT NOT NULL, value TEXT NOT NULL, updated_at INTEGER NOT NULL, PRIMARY KEY (visitor, key))",
        ] {
            sqlx::query(statement).execute(&store).await?;
        }
        Ok(Arc::new(Self {
            config: Arc::new(config),
            store,
            loaded: Mutex::new(HashMap::new()),
        }))
    }

    /// The configuration in force.
    #[must_use]
    pub fn config(&self) -> &Config {
        &self.config
    }

    fn sandbox_path(&self, id: &str) -> PathBuf {
        self.config.sandboxes().join(format!("{id}.db"))
    }

    /// The client address the per-client limit counts: the TCP peer, or
    /// behind a trusted proxy the rightmost forwarded hop it did not add.
    fn client_of(&self, peer: Option<IpAddr>, headers: &HeaderMap) -> String {
        let trusted = |ip: IpAddr| {
            self.config
                .trusted_proxies
                .iter()
                .any(|net| net.contains(ip))
        };
        let Some(mut client) = peer.map(|ip| ip.to_canonical()) else {
            return "local".into();
        };
        if trusted(client) {
            let forwarded = headers
                .get_all("x-forwarded-for")
                .iter()
                .filter_map(|value| value.to_str().ok())
                .collect::<Vec<_>>()
                .join(",");
            for hop in forwarded.rsplit(',') {
                match hop.trim().parse::<IpAddr>() {
                    Ok(ip) if trusted(ip.to_canonical()) => {}
                    Ok(ip) => {
                        client = ip.to_canonical();
                        break;
                    }
                    Err(_) => break,
                }
            }
        }
        match client {
            IpAddr::V4(v4) => v4.to_string(),
            IpAddr::V6(v6) => {
                let s = v6.segments();
                format!("{:x}:{:x}:{:x}:{:x}::/64", s[0], s[1], s[2], s[3])
            }
        }
    }

    /// The visitor a request belongs to, creating one when it carries no live
    /// cookie. Returns the id and, for a new visitor, the cookie to set.
    async fn visitor(
        &self,
        headers: &HeaderMap,
        peer: Option<IpAddr>,
    ) -> Result<(String, i64, Option<String>), Box<Response>> {
        let at = now();
        if let Some(cookie) = read_cookie(headers) {
            let id = visitor_id(&cookie);
            let row = sqlx::query("SELECT created_at FROM visitors WHERE id = ?")
                .bind(&id)
                .fetch_optional(&self.store)
                .await
                .map_err(|_| {
                    refuse(
                        StatusCode::SERVICE_UNAVAILABLE,
                        "the visitor store is unavailable",
                    )
                })?;
            if let Some(row) = row {
                let created: i64 = row.get(0);
                if created + self.config.ttl_seconds > at {
                    let _ = sqlx::query(
                        "UPDATE visitors SET last_seen = ? WHERE id = ? AND last_seen < ?",
                    )
                    .bind(at)
                    .bind(&id)
                    .bind(at - 60)
                    .execute(&self.store)
                    .await;
                    return Ok((id, created, None));
                }
            }
        }
        let client = self.client_of(peer, headers);
        let live: i64 = sqlx::query("SELECT COUNT(*) FROM visitors WHERE created_at > ?")
            .bind(at - self.config.ttl_seconds)
            .fetch_one(&self.store)
            .await
            .map(|row| row.get(0))
            .unwrap_or(i64::MAX);
        if live >= self.config.max_visitors {
            return Err(refuse(
                StatusCode::SERVICE_UNAVAILABLE,
                "The hosted pre-alpha is full right now. Try again later, or download the desktop app.",
            ));
        }
        let recent: i64 =
            sqlx::query("SELECT COUNT(*) FROM visitors WHERE client = ? AND created_at > ?")
                .bind(&client)
                .bind(at - 3600)
                .fetch_one(&self.store)
                .await
                .map(|row| row.get(0))
                .unwrap_or(i64::MAX);
        if recent >= self.config.new_per_client_per_hour {
            return Err(refuse(
                StatusCode::TOO_MANY_REQUESTS,
                "Too many new sessions from this address; try again within the hour.",
            ));
        }
        let mut bytes = [0_u8; 32];
        rand::rng().fill_bytes(&mut bytes);
        let cookie = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes);
        let id = visitor_id(&cookie);
        sqlx::query("INSERT INTO visitors (id, created_at, last_seen, client) VALUES (?, ?, ?, ?)")
            .bind(&id)
            .bind(at)
            .bind(at)
            .bind(&client)
            .execute(&self.store)
            .await
            .map_err(|_| refuse(StatusCode::SERVICE_UNAVAILABLE, "could not start a session"))?;
        tracing::info!(live = live + 1, "new visitor");
        Ok((id, at, Some(cookie)))
    }

    fn set_cookie(&self, cookie: &str) -> HeaderValue {
        let secure = if self.config.cookie_secure {
            "; Secure"
        } else {
            ""
        };
        HeaderValue::from_str(&format!(
            "{VISITOR_COOKIE}={cookie}; Path=/; Max-Age={}; HttpOnly; SameSite=Strict{secure}",
            self.config.ttl_seconds
        ))
        .unwrap_or_else(|_| HeaderValue::from_static(""))
    }

    /// The visitor's API, opened on first use. Like the desktop shell, each
    /// opening revokes the earlier desk tokens and issues a fresh one.
    async fn sandbox(&self, id: &str) -> anyhow::Result<Arc<Sandbox>> {
        let mut loaded = self.loaded.lock().await;
        if let Some(sandbox) = loaded.get(id) {
            sandbox.last_used.store(now(), Ordering::Relaxed);
            return Ok(sandbox.clone());
        }
        let path = self.sandbox_path(id);
        let repository = asten_storage::SqliteRepository::open(&path).await?;
        let pool = repository.pool().clone();
        let application = Application::with_auth(Arc::new(repository), AuthMode::Pairing);
        let token = fresh_desk_token(&application).await?;
        let config = ApiConfig {
            auth_mode: AuthMode::Pairing,
            cookie_secure: self.config.cookie_secure,
            trusted_proxies: self.config.trusted_proxies.clone(),
            database_path: None,
            ..ApiConfig::default()
        };
        let sandbox = Arc::new(Sandbox {
            router: asten_api::router(application.clone(), config),
            application,
            pool,
            token: std::sync::Mutex::new(token),
            last_used: AtomicI64::new(now()),
        });
        if loaded.len() >= self.config.max_loaded
            && let Some(oldest) = loaded
                .iter()
                .min_by_key(|(_, s)| s.last_used.load(Ordering::Relaxed))
                .map(|(key, _)| key.clone())
            && let Some(evicted) = loaded.remove(&oldest)
        {
            tokio::spawn(async move { evicted.pool.close().await });
        }
        loaded.insert(id.to_owned(), sandbox.clone());
        Ok(sandbox)
    }

    /// Bytes on disk for a visitor's sandbox (database and write-ahead log).
    async fn sandbox_bytes(&self, id: &str) -> u64 {
        let base = self.sandbox_path(id);
        let mut total = 0;
        for suffix in ["", "-wal"] {
            let path = PathBuf::from(format!("{}{suffix}", base.display()));
            if let Ok(meta) = tokio::fs::metadata(&path).await {
                total += meta.len();
            }
        }
        total
    }

    async fn remove_sandbox_files(&self, id: &str) {
        let open = self.loaded.lock().await.remove(id);
        if let Some(sandbox) = open {
            sandbox.pool.close().await;
        }
        let base = self.sandbox_path(id);
        for suffix in ["", "-wal", "-shm"] {
            let _ = tokio::fs::remove_file(format!("{}{suffix}", base.display())).await;
        }
    }

    /// Delete visitors older than the time-to-live, with their sandboxes and
    /// state, forget client addresses older than an hour, close idle
    /// sandboxes, and remove sandbox files no visitor owns.
    /// Returns the number of visitors deleted.
    ///
    /// # Errors
    /// The store cannot be read.
    pub async fn sweep_at(&self, at: i64) -> anyhow::Result<usize> {
        let expired: Vec<String> = sqlx::query("SELECT id FROM visitors WHERE created_at <= ?")
            .bind(at - self.config.ttl_seconds)
            .fetch_all(&self.store)
            .await?
            .into_iter()
            .map(|row| row.get(0))
            .collect();
        // The client address is kept only as long as the per-address limit
        // needs it: one hour.
        sqlx::query("UPDATE visitors SET client = '' WHERE created_at <= ? AND client <> ''")
            .bind(at - 3600)
            .execute(&self.store)
            .await?;
        for id in &expired {
            self.remove_sandbox_files(id).await;
            sqlx::query("DELETE FROM state WHERE visitor = ?")
                .bind(id)
                .execute(&self.store)
                .await?;
            sqlx::query("DELETE FROM visitors WHERE id = ?")
                .bind(id)
                .execute(&self.store)
                .await?;
        }
        {
            let mut loaded = self.loaded.lock().await;
            let idle: Vec<String> = loaded
                .iter()
                .filter(|(_, s)| {
                    s.last_used.load(Ordering::Relaxed) + self.config.idle_seconds <= at
                })
                .map(|(key, _)| key.clone())
                .collect();
            for key in idle {
                if let Some(sandbox) = loaded.remove(&key) {
                    tokio::spawn(async move { sandbox.pool.close().await });
                }
            }
        }
        let mut entries = tokio::fs::read_dir(self.config.sandboxes()).await?;
        while let Some(entry) = entries.next_entry().await? {
            let name = entry.file_name().to_string_lossy().into_owned();
            let Some(id) = name.split('.').next().filter(|id| id.len() == 64) else {
                continue;
            };
            let owned = sqlx::query("SELECT 1 FROM visitors WHERE id = ?")
                .bind(id)
                .fetch_optional(&self.store)
                .await?
                .is_some();
            let fresh = entry
                .metadata()
                .await
                .ok()
                .and_then(|m| m.modified().ok())
                .and_then(|m| m.elapsed().ok())
                .is_some_and(|age| age < Duration::from_secs(120));
            if !owned && !fresh {
                let _ = tokio::fs::remove_file(entry.path()).await;
            }
        }
        if !expired.is_empty() {
            tracing::info!(count = expired.len(), "deleted expired visitors");
        }
        Ok(expired.len())
    }

    /// Sweep every five minutes, for as long as the process runs.
    pub fn spawn_sweeper(self: &Arc<Self>) {
        let gateway = self.clone();
        tokio::spawn(async move {
            let mut tick = tokio::time::interval(Duration::from_secs(300));
            loop {
                tick.tick().await;
                if let Err(err) = gateway.sweep_at(now()).await {
                    tracing::warn!(error = %err, "sweep failed");
                }
            }
        });
    }
}

async fn fresh_desk_token(application: &Application) -> anyhow::Result<String> {
    for token in application.list_tokens().await? {
        if token.kind == asten_domain::TokenKind::Desk && token.revoked_at.is_none() {
            let _ = application.revoke_token(token.id).await;
        }
    }
    Ok(application.issue_desk_token(TOKEN_LABEL).await?.token)
}

fn peer_of(request: &Request<Body>) -> Option<IpAddr> {
    request
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|info| info.0.ip())
}

fn with_cookie(gateway: &Gateway, mut response: Response, cookie: Option<String>) -> Response {
    if let Some(cookie) = cookie {
        response
            .headers_mut()
            .append(header::SET_COOKIE, gateway.set_cookie(&cookie));
    }
    response
}

/// `/api/…`: find the visitor, open their sandbox, and hand the request to
/// its API with the sandbox's desk token.
async fn api(State(gateway): State<Arc<Gateway>>, request: Request<Body>) -> Response {
    if foreign(request.headers()) {
        return error(
            StatusCode::FORBIDDEN,
            "cross-origin requests are not accepted",
        );
    }
    let peer = peer_of(&request);
    let (id, _, cookie) = match gateway.visitor(request.headers(), peer).await {
        Ok(found) => found,
        Err(response) => return *response,
    };
    let sandbox = match gateway.sandbox(&id).await {
        Ok(sandbox) => sandbox,
        Err(err) => {
            tracing::error!(error = %err, "could not open a sandbox");
            return with_cookie(
                &gateway,
                error(
                    StatusCode::SERVICE_UNAVAILABLE,
                    "could not open your sandbox",
                ),
                cookie,
            );
        }
    };
    let writes = !matches!(
        *request.method(),
        Method::GET | Method::HEAD | Method::OPTIONS
    );
    if writes && gateway.sandbox_bytes(&id).await > gateway.config.max_sandbox_bytes {
        return with_cookie(
            &gateway,
            error(
                StatusCode::INSUFFICIENT_STORAGE,
                "This sandbox has reached its size limit. Start over from About, or use the desktop app.",
            ),
            cookie,
        );
    }
    let (mut parts, body) = request.into_parts();
    // The gateway's own router left its matched path parameters (`rest`) in
    // the extensions; the sandbox's router must route the path afresh.
    parts.extensions = http::Extensions::new();
    let Ok(body) = axum::body::to_bytes(body, BODY_LIMIT).await else {
        return with_cookie(
            &gateway,
            error(StatusCode::PAYLOAD_TOO_LARGE, "request body too large"),
            cookie,
        );
    };
    strip_visitor_cookie(&mut parts.headers);
    let injected = !parts.headers.contains_key(header::AUTHORIZATION);
    let build = |parts: &http::request::Parts, body: Bytes, token: &str| {
        let mut request = Request::from_parts(parts.clone(), Body::from(body));
        if injected && let Ok(value) = HeaderValue::from_str(&format!("Bearer {token}")) {
            request.headers_mut().insert(header::AUTHORIZATION, value);
        }
        if let Some(peer) = peer {
            request
                .extensions_mut()
                .insert(ConnectInfo(SocketAddr::new(peer, 0)));
        }
        request
    };
    let first = build(&parts, body.clone(), &sandbox.token());
    let mut response = match sandbox.router.clone().oneshot(first).await {
        Ok(response) => response,
        Err(never) => match never {},
    };
    // The desk can sign out, which revokes the token the gateway presents;
    // issue a fresh one, as the desktop shell does at launch, and try once more.
    if injected && response.status() == StatusCode::UNAUTHORIZED {
        if let Ok(token) = fresh_desk_token(&sandbox.application).await {
            if let Ok(mut held) = sandbox.token.lock() {
                held.clone_from(&token);
            }
            let retry = build(&parts, body, &token);
            response = match sandbox.router.clone().oneshot(retry).await {
                Ok(response) => response,
                Err(never) => match never {},
            };
        }
    }
    with_cookie(&gateway, response, cookie)
}

/// What the gateway itself answers about the visitor and the limits.
#[derive(Serialize)]
struct SessionView {
    created_at: String,
    expires_at: String,
    sandbox_bytes: u64,
    limits: LimitsView,
    /// Which surface serves which part of the workbench.
    served_by: ServedBy,
}

#[derive(Serialize)]
struct LimitsView {
    ttl_hours: i64,
    max_sandbox_bytes: u64,
    max_state_value_bytes: usize,
    max_state_total_bytes: usize,
    max_state_keys: i64,
}

#[derive(Serialize)]
struct ServedBy {
    api: &'static [&'static str],
    service: &'static [&'static str],
    browser_memory: &'static [&'static str],
}

const SERVED_BY: ServedBy = ServedBy {
    api: &[
        "projects, binder and documents",
        "saving, comments and tracked changes",
        "snapshots, history, verify and blame",
        "drafts and merge",
        "facts, cast, events, world board and routes",
        "marks, repairs, suppressions and baselines",
        "packs and project settings",
        "compile (Markdown, text, Fountain, HTML, DOCX, EPUB, FDX) and import",
        "pairing codes and tokens",
    ],
    service: &[
        "workbench preferences, appearance and keybindings",
        "assistant threads, skills and access",
        "provider and connector settings (never keys)",
        "console history and layout",
    ],
    browser_memory: &["provider keys"],
};

async fn svc_visitor(
    gateway: &Gateway,
    request_headers: &HeaderMap,
    peer: Option<IpAddr>,
) -> Result<(String, i64, Option<String>), Box<Response>> {
    if foreign(request_headers) {
        return Err(refuse(
            StatusCode::FORBIDDEN,
            "cross-origin requests are not accepted",
        ));
    }
    gateway.visitor(request_headers, peer).await
}

async fn session(State(gateway): State<Arc<Gateway>>, request: Request<Body>) -> Response {
    let (id, created, cookie) =
        match svc_visitor(&gateway, request.headers(), peer_of(&request)).await {
            Ok(found) => found,
            Err(response) => return *response,
        };
    let c = &gateway.config;
    let view = SessionView {
        created_at: rfc3339(created),
        expires_at: rfc3339(created + c.ttl_seconds),
        sandbox_bytes: gateway.sandbox_bytes(&id).await,
        limits: LimitsView {
            ttl_hours: c.ttl_seconds / 3600,
            max_sandbox_bytes: c.max_sandbox_bytes,
            max_state_value_bytes: c.max_state_value_bytes,
            max_state_total_bytes: c.max_state_total_bytes,
            max_state_keys: c.max_state_keys,
        },
        served_by: SERVED_BY,
    };
    with_cookie(&gateway, Json(view).into_response(), cookie)
}

async fn get_state(State(gateway): State<Arc<Gateway>>, request: Request<Body>) -> Response {
    let (id, _, cookie) = match svc_visitor(&gateway, request.headers(), peer_of(&request)).await {
        Ok(found) => found,
        Err(response) => return *response,
    };
    let rows =
        sqlx::query("SELECT key, value FROM state WHERE visitor = ? AND key NOT LIKE 'claim:%'")
            .bind(&id)
            .fetch_all(&gateway.store)
            .await
            .unwrap_or_default();
    let mut items = serde_json::Map::new();
    for row in rows {
        let key: String = row.get(0);
        let value: String = row.get(1);
        items.insert(key, serde_json::from_str(&value).unwrap_or(Value::Null));
    }
    with_cookie(
        &gateway,
        Json(json!({ "items": items })).into_response(),
        cookie,
    )
}

fn valid_key(key: &str) -> bool {
    (1..=80).contains(&key.len())
        && !key.starts_with("claim:")
        && key
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b':' | b'.' | b'_' | b'-'))
}

async fn put_state(
    State(gateway): State<Arc<Gateway>>,
    UrlPath(key): UrlPath<String>,
    request: Request<Body>,
) -> Response {
    let (id, _, cookie) = match svc_visitor(&gateway, request.headers(), peer_of(&request)).await {
        Ok(found) => found,
        Err(response) => return *response,
    };
    if !valid_key(&key) {
        return with_cookie(
            &gateway,
            error(StatusCode::BAD_REQUEST, "invalid key"),
            cookie,
        );
    }
    let limit = gateway.config.max_state_value_bytes;
    let Ok(bytes) = axum::body::to_bytes(request.into_body(), limit + 1).await else {
        return with_cookie(
            &gateway,
            error(StatusCode::PAYLOAD_TOO_LARGE, "value too large"),
            cookie,
        );
    };
    if bytes.len() > limit {
        return with_cookie(
            &gateway,
            error(StatusCode::PAYLOAD_TOO_LARGE, "value too large"),
            cookie,
        );
    }
    let Ok(value) = serde_json::from_slice::<Value>(&bytes) else {
        return with_cookie(
            &gateway,
            error(StatusCode::BAD_REQUEST, "the value must be JSON"),
            cookie,
        );
    };
    let text = value.to_string();
    let others = sqlx::query(
        "SELECT COUNT(*), COALESCE(SUM(LENGTH(value)), 0) FROM state WHERE visitor = ? AND key <> ?",
    )
    .bind(&id)
    .bind(&key)
    .fetch_one(&gateway.store)
    .await;
    let (count, total): (i64, i64) = match others {
        Ok(row) => (row.get(0), row.get(1)),
        Err(_) => return error(StatusCode::SERVICE_UNAVAILABLE, "the store is unavailable"),
    };
    if count + 1 > gateway.config.max_state_keys
        || usize::try_from(total).unwrap_or(usize::MAX) + text.len()
            > gateway.config.max_state_total_bytes
    {
        return with_cookie(
            &gateway,
            error(
                StatusCode::INSUFFICIENT_STORAGE,
                "stored state is at its limit",
            ),
            cookie,
        );
    }
    let saved = sqlx::query(
        "INSERT INTO state (visitor, key, value, updated_at) VALUES (?, ?, ?, ?) \
         ON CONFLICT (visitor, key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
    )
    .bind(&id)
    .bind(&key)
    .bind(&text)
    .bind(now())
    .execute(&gateway.store)
    .await;
    let response = match saved {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(_) => error(StatusCode::SERVICE_UNAVAILABLE, "the store is unavailable"),
    };
    with_cookie(&gateway, response, cookie)
}

async fn delete_state(
    State(gateway): State<Arc<Gateway>>,
    UrlPath(key): UrlPath<String>,
    request: Request<Body>,
) -> Response {
    let (id, _, cookie) = match svc_visitor(&gateway, request.headers(), peer_of(&request)).await {
        Ok(found) => found,
        Err(response) => return *response,
    };
    if !valid_key(&key) {
        return with_cookie(
            &gateway,
            error(StatusCode::BAD_REQUEST, "invalid key"),
            cookie,
        );
    }
    let _ = sqlx::query("DELETE FROM state WHERE visitor = ? AND key = ?")
        .bind(&id)
        .bind(&key)
        .execute(&gateway.store)
        .await;
    with_cookie(&gateway, StatusCode::NO_CONTENT.into_response(), cookie)
}

/// `POST /svc/v1/claim/{name}`: `{"first": true}` exactly once per visitor and
/// name, so two tabs do not both create the sample novel.
async fn claim(
    State(gateway): State<Arc<Gateway>>,
    UrlPath(name): UrlPath<String>,
    request: Request<Body>,
) -> Response {
    let (id, _, cookie) = match svc_visitor(&gateway, request.headers(), peer_of(&request)).await {
        Ok(found) => found,
        Err(response) => return *response,
    };
    if !valid_key(&name) {
        return with_cookie(
            &gateway,
            error(StatusCode::BAD_REQUEST, "invalid name"),
            cookie,
        );
    }
    let inserted = sqlx::query(
        "INSERT OR IGNORE INTO state (visitor, key, value, updated_at) VALUES (?, ?, 'true', ?)",
    )
    .bind(&id)
    .bind(format!("claim:{name}"))
    .bind(now())
    .execute(&gateway.store)
    .await
    .map(|done| done.rows_affected() == 1)
    .unwrap_or(false);
    with_cookie(
        &gateway,
        Json(json!({ "first": inserted })).into_response(),
        cookie,
    )
}

/// `POST /svc/v1/reset`: delete this visitor's sandbox and state. The next
/// request opens an empty sandbox; the session's expiry does not move.
async fn reset(State(gateway): State<Arc<Gateway>>, request: Request<Body>) -> Response {
    let (id, _, cookie) = match svc_visitor(&gateway, request.headers(), peer_of(&request)).await {
        Ok(found) => found,
        Err(response) => return *response,
    };
    gateway.remove_sandbox_files(&id).await;
    let _ = sqlx::query("DELETE FROM state WHERE visitor = ?")
        .bind(&id)
        .execute(&gateway.store)
        .await;
    with_cookie(&gateway, StatusCode::NO_CONTENT.into_response(), cookie)
}

async fn health() -> Json<Value> {
    Json(
        json!({ "status": "ok", "service": "asten-prealpha", "version": env!("CARGO_PKG_VERSION") }),
    )
}

const DESK_CSP: &str = "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; font-src 'self' data:; img-src 'self' data: blob:; connect-src 'self'; worker-src 'self'; manifest-src 'self'; frame-src 'none'; object-src 'none'; base-uri 'none'; form-action 'self'; frame-ancestors 'none'";

/// Headers for everything the gateway answers itself: the desk's files and
/// `/svc/`. The API sets its own.
async fn desk_headers(request: Request<Body>, next: Next) -> Response {
    let path = request.uri().path().to_owned();
    let mut response = next.run(request).await;
    if path.starts_with("/api/") {
        return response;
    }
    let ok = response.status().is_success();
    let headers = response.headers_mut();
    headers.insert(
        "x-content-type-options",
        HeaderValue::from_static("nosniff"),
    );
    headers.insert("x-frame-options", HeaderValue::from_static("DENY"));
    headers.insert("referrer-policy", HeaderValue::from_static("no-referrer"));
    headers.insert(
        "x-robots-tag",
        HeaderValue::from_static("noindex, nofollow"),
    );
    headers.insert(
        "permissions-policy",
        HeaderValue::from_static("camera=(), microphone=(), geolocation=(), payment=(), usb=()"),
    );
    if path.starts_with("/svc/") || path == "/health" {
        headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
        headers.insert(
            "content-security-policy",
            HeaderValue::from_static("default-src 'none'; frame-ancestors 'none'"),
        );
    } else {
        headers.insert(
            "content-security-policy",
            HeaderValue::from_static(DESK_CSP),
        );
        // Hashed assets are content-addressed; a missing one must not be
        // cached under its name, and the shell revalidates on every load.
        let cache = if path.starts_with("/assets/") && ok {
            "public, max-age=31536000, immutable"
        } else {
            "no-cache"
        };
        headers.insert(header::CACHE_CONTROL, HeaderValue::from_static(cache));
    }
    response
}

/// The gateway's router: `/api/…` to the visitor's sandbox, `/svc/v1/…` to the
/// state store, `/health`, and the built desk for everything else.
pub fn router(gateway: Arc<Gateway>) -> Router {
    let mut app = Router::new()
        .route("/health", get(health))
        .route("/api/{*rest}", any(api))
        .route("/svc/v1/session", get(session))
        .route("/svc/v1/state", get(get_state))
        .route("/svc/v1/state/{key}", put(put_state).delete(delete_state))
        .route("/svc/v1/claim/{name}", post(claim))
        .route("/svc/v1/reset", post(reset));
    if let Some(desk) = gateway.config.desk_dir.clone() {
        let index = desk.join("index.html");
        app = app
            .nest_service("/assets", ServeDir::new(desk.join("assets")))
            .fallback_service(ServeDir::new(&desk).not_found_service(ServeFile::new(index)));
    }
    app.with_state(gateway)
        .layer(middleware::from_fn(desk_headers))
}

/// Serve until `shutdown` resolves.
///
/// # Errors
/// The listener fails.
pub async fn serve(
    listener: tokio::net::TcpListener,
    gateway: Arc<Gateway>,
    shutdown: impl std::future::Future<Output = ()> + Send + 'static,
) -> anyhow::Result<()> {
    gateway.spawn_sweeper();
    axum::serve(
        listener,
        router(gateway).into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown)
    .await?;
    Ok(())
}
