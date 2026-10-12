//! Authentication middleware and the pairing, token and owner routes.

use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;

use asten_application::AuthMode;
use asten_application::auth::{AuthStatus, PairingComplete};
use asten_domain::{AuthToken, AuthTokenId, ProjectId, TokenKind};
use axum::{
    Extension, Json, Router,
    body::Body,
    extract::{ConnectInfo, Path, Query, State},
    http::{HeaderMap, HeaderValue, Request, StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{ApiConfig, ApiError, ApiState};

pub const SESSION_COOKIE: &str = "asten_session";

/// A key that identifies the calling client for rate limiting: the peer address.
#[derive(Clone, Debug)]
pub struct ClientKey(pub String);

/// The authenticated caller, inserted by [`require_auth`].
#[derive(Clone, Debug)]
pub struct AuthenticatedClient(pub asten_application::auth::AuthenticatedClient);

/// Derive the client key from the connection, before authentication. Behind a
/// trusted reverse proxy the key is the first `X-Forwarded-For` hop; IPv6
/// clients are keyed by their /64 so a host cannot rotate through its block.
pub async fn client_key(
    State(config): State<Arc<ApiConfig>>,
    mut request: Request<Body>,
    next: Next,
) -> Response {
    let peer = request
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|info| info.0.ip());
    let forwarded = request
        .headers()
        .get("x-forwarded-for")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(',').next())
        .and_then(|first| first.trim().parse::<IpAddr>().ok());
    let client = match (peer, forwarded) {
        (Some(peer), Some(forwarded))
            if config.trusted_proxies.iter().any(|net| net.contains(peer)) =>
        {
            Some(forwarded)
        }
        (peer, _) => peer,
    };
    let key = client.map_or_else(|| "local".to_owned(), bucket_key);
    request.extensions_mut().insert(ClientKey(key));
    next.run(request).await
}

fn bucket_key(ip: IpAddr) -> String {
    match ip {
        IpAddr::V4(v4) => v4.to_string(),
        IpAddr::V6(v6) => {
            let segments = v6.segments();
            format!(
                "{:x}:{:x}:{:x}:{:x}::/64",
                segments[0], segments[1], segments[2], segments[3]
            )
        }
    }
}

/// Guests may not manage pairing, tokens or the passphrase.
fn require_admin(client: &AuthenticatedClient) -> Option<Response> {
    if client.0.kind.can_administer() {
        None
    } else {
        let body = serde_json::json!({
            "error": { "code": "403", "message": "a guest cannot manage pairing; use the desk" }
        });
        Some((StatusCode::FORBIDDEN, Json(body)).into_response())
    }
}

/// The project id in `/api/v1/projects/{id}/stream`, when the path is that.
fn stream_project(path: &str) -> Option<ProjectId> {
    let rest = path.strip_suffix("/stream")?;
    let (_, id) = rest.rsplit_once("/projects/")?;
    id.parse::<Uuid>().ok().map(ProjectId)
}

/// Did this request come from the page the API itself serves (or a proxy in
/// front of both)? Only then is a cookie useful; a cross-origin desk keeps its
/// bearer token, and a cookie would leak to every other port on the host.
fn same_origin(headers: &HeaderMap) -> bool {
    let Some(origin) = headers.get(header::ORIGIN).and_then(|v| v.to_str().ok()) else {
        return true;
    };
    let Some(host) = headers.get(header::HOST).and_then(|v| v.to_str().ok()) else {
        return false;
    };
    origin
        .split_once("://")
        .is_some_and(|(_, origin_host)| origin_host.eq_ignore_ascii_case(host))
}

fn presented_token(headers: &HeaderMap, query: Option<&str>) -> Option<String> {
    if let Some(value) = headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
    {
        if let Some(token) = asten_auth_bearer(value) {
            return Some(token.to_owned());
        }
    }
    if let Some(cookie) = headers
        .get(header::COOKIE)
        .and_then(|value| value.to_str().ok())
    {
        if let Some(token) = asten_auth::cookie_value(cookie, SESSION_COOKIE) {
            return Some(token.to_owned());
        }
    }
    let _ = query;
    None
}

/// `ticket=` from a query string.
fn query_ticket(query: Option<&str>) -> Option<String> {
    query.and_then(|query| {
        query
            .split('&')
            .find_map(|pair| pair.strip_prefix("ticket=").map(|ticket| ticket.to_owned()))
    })
}

fn asten_auth_bearer(value: &str) -> Option<&str> {
    asten_auth::bearer(value)
}

/// Reject unauthenticated requests with a JSON 401.
pub async fn require_auth(
    State(state): State<ApiState>,
    mut request: Request<Body>,
    next: Next,
) -> Response {
    if request.method() == axum::http::Method::OPTIONS {
        return next.run(request).await;
    }
    // EventSource cannot set headers, so the stream route also accepts a
    // one-minute single-use ticket minted by an authenticated call.
    if let Some(project) = stream_project(request.uri().path()) {
        if let Some(ticket) = query_ticket(request.uri().query()) {
            return match state.application.redeem_stream_ticket(&ticket, project) {
                Some(client) => {
                    request.extensions_mut().insert(AuthenticatedClient(client));
                    next.run(request).await
                }
                None => unauthorized("that stream ticket is spent or expired"),
            };
        }
    }
    let presented = presented_token(request.headers(), None);
    match state.application.authenticate(presented.as_deref()).await {
        Ok(Some(client)) => {
            request.extensions_mut().insert(AuthenticatedClient(client));
            next.run(request).await
        }
        Ok(None) => unauthorized("pair this client with the desk first"),
        Err(error) => ApiError(error).into_response(),
    }
}

fn unauthorized(message: &str) -> Response {
    let body = serde_json::json!({ "error": { "code": "401", "message": message } });
    let mut response = (StatusCode::UNAUTHORIZED, Json(body)).into_response();
    response.headers_mut().insert(
        header::WWW_AUTHENTICATE,
        HeaderValue::from_static("Bearer realm=\"asten\""),
    );
    response
}

fn session_cookie(token: &str, secure: bool, max_age_days: i64) -> HeaderValue {
    let secure = if secure { "; Secure" } else { "" };
    let value = format!(
        "{SESSION_COOKIE}={token}; Path=/; HttpOnly; SameSite=Strict; Max-Age={}{secure}",
        max_age_days * 86_400
    );
    HeaderValue::from_str(&value).unwrap_or_else(|_| HeaderValue::from_static(""))
}

fn clear_cookie() -> HeaderValue {
    HeaderValue::from_static("asten_session=; Path=/; HttpOnly; SameSite=Strict; Max-Age=0")
}

/// Routes that need no token.
pub fn public_router() -> Router<ApiState> {
    Router::new()
        .route("/auth/status", get(status))
        .route("/auth/pair", post(pair))
        .route("/auth/login", post(login))
        .route("/auth/owner/setup", post(setup_owner))
}

/// Routes behind authentication.
pub fn protected_router() -> Router<ApiState> {
    Router::new()
        .route("/auth/me", get(me))
        .route("/auth/logout", post(logout))
        .route("/auth/pairing-codes", post(start_pairing))
        .route("/auth/tokens", get(list_tokens))
        .route(
            "/auth/tokens/{token_id}",
            axum::routing::delete(revoke_token),
        )
        .route(
            "/auth/owner/passphrase",
            axum::routing::put(change_passphrase),
        )
        .route("/projects/{project_id}/stream-ticket", post(stream_ticket))
}

async fn stream_ticket(
    State(state): State<ApiState>,
    Path(project_id): Path<Uuid>,
    Extension(AuthenticatedClient(client)): Extension<AuthenticatedClient>,
) -> Result<
    (
        StatusCode,
        Json<asten_application::auth::StreamTicketIssued>,
    ),
    ApiError,
> {
    let issued = state
        .application
        .issue_stream_ticket(ProjectId(project_id), &client)
        .await?;
    Ok((StatusCode::CREATED, Json(issued)))
}

async fn status(State(state): State<ApiState>) -> Result<Json<AuthStatus>, ApiError> {
    Ok(Json(state.application.auth_status().await?))
}

#[derive(Deserialize)]
struct PairRequest {
    code: String,
    #[serde(default)]
    label: String,
}

#[derive(Serialize)]
struct PairResponse {
    #[serde(flatten)]
    paired: PairingComplete,
    /// The session cookie was also set, for browser clients.
    cookie: bool,
}

async fn pair(
    State(state): State<ApiState>,
    Extension(ClientKey(key)): Extension<ClientKey>,
    headers: HeaderMap,
    Json(input): Json<PairRequest>,
) -> Result<Response, ApiError> {
    let paired = state
        .application
        .complete_pairing(&input.code, &input.label, &key)
        .await?;
    let set_cookie = same_origin(&headers);
    let cookie = session_cookie(&paired.token, state.config.cookie_secure, 365);
    let mut response = (
        StatusCode::CREATED,
        Json(PairResponse {
            paired,
            cookie: set_cookie,
        }),
    )
        .into_response();
    if set_cookie {
        response.headers_mut().insert(header::SET_COOKIE, cookie);
    }
    Ok(response)
}

#[derive(Deserialize)]
struct LoginRequest {
    passphrase: String,
}

async fn login(
    State(state): State<ApiState>,
    Extension(ClientKey(key)): Extension<ClientKey>,
    headers: HeaderMap,
    Json(input): Json<LoginRequest>,
) -> Result<Response, ApiError> {
    if state.config.auth_mode != AuthMode::Owner {
        return Err(ApiError(asten_application::ApplicationError::Unauthorized(
            "owner login is not enabled on this server".into(),
        )));
    }
    let session = state
        .application
        .login_owner(&input.passphrase, &key)
        .await?;
    let set_cookie = same_origin(&headers);
    let cookie = session_cookie(&session.token, state.config.cookie_secure, 30);
    let mut response = (
        StatusCode::CREATED,
        Json(PairResponse {
            paired: session,
            cookie: set_cookie,
        }),
    )
        .into_response();
    if set_cookie {
        response.headers_mut().insert(header::SET_COOKIE, cookie);
    }
    Ok(response)
}

#[derive(Deserialize)]
struct SetupOwnerRequest {
    passphrase: String,
}

/// First-run setup of the owner passphrase. Allowed without a token only while
/// no passphrase exists and the server runs in owner mode; afterwards it needs
/// an authenticated caller, and the change route asks for the current one.
async fn setup_owner(
    State(state): State<ApiState>,
    Extension(ClientKey(key)): Extension<ClientKey>,
    headers: HeaderMap,
    Json(input): Json<SetupOwnerRequest>,
) -> Result<Response, ApiError> {
    let status = state.application.auth_status().await?;
    if status.owner_configured {
        return Err(ApiError(asten_application::ApplicationError::Conflict(
            "an owner passphrase is already set".into(),
        )));
    }
    if state.config.auth_mode != AuthMode::Owner {
        let presented = presented_token(&headers, None);
        let Some(client) = state.application.authenticate(presented.as_deref()).await? else {
            return Err(ApiError(asten_application::ApplicationError::Unauthorized(
                "pair first, then set the passphrase".into(),
            )));
        };
        if let Some(forbidden) = require_admin(&AuthenticatedClient(client)) {
            return Ok(forbidden);
        }
    }
    state
        .application
        .setup_owner(&input.passphrase, &key)
        .await?;
    Ok(StatusCode::CREATED.into_response())
}

#[derive(Deserialize)]
struct ChangePassphraseRequest {
    current: String,
    next: String,
}

async fn change_passphrase(
    State(state): State<ApiState>,
    Extension(ClientKey(key)): Extension<ClientKey>,
    Extension(client): Extension<AuthenticatedClient>,
    Json(input): Json<ChangePassphraseRequest>,
) -> Result<Response, ApiError> {
    if let Some(forbidden) = require_admin(&client) {
        return Ok(forbidden);
    }
    state
        .application
        .change_owner_passphrase(&input.current, &input.next, &key)
        .await?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

async fn me(
    Extension(AuthenticatedClient(client)): Extension<AuthenticatedClient>,
) -> Json<asten_application::auth::AuthenticatedClient> {
    Json(client)
}

async fn logout(State(state): State<ApiState>, headers: HeaderMap) -> Result<Response, ApiError> {
    if let Some(presented) = presented_token(&headers, None) {
        state.application.logout(&presented).await?;
    }
    let mut response = StatusCode::NO_CONTENT.into_response();
    response
        .headers_mut()
        .insert(header::SET_COOKIE, clear_cookie());
    Ok(response)
}

#[derive(Deserialize, Default)]
struct StartPairingRequest {
    #[serde(default)]
    label: String,
    /// `guest` (default) or `desk`. A desk code grants a token that may manage
    /// pairing; a guest code grants one that may only read and write the story.
    #[serde(default)]
    grants: Option<TokenKind>,
}

async fn start_pairing(
    State(state): State<ApiState>,
    Extension(client): Extension<AuthenticatedClient>,
    Json(input): Json<StartPairingRequest>,
) -> Result<Response, ApiError> {
    if let Some(forbidden) = require_admin(&client) {
        return Ok(forbidden);
    }
    let grants = match input.grants.unwrap_or(TokenKind::Guest) {
        TokenKind::Desk => TokenKind::Desk,
        _ => TokenKind::Guest,
    };
    let start = state
        .application
        .start_pairing(&input.label, grants)
        .await?;
    Ok((StatusCode::CREATED, Json(start)).into_response())
}

#[derive(Serialize)]
struct TokenView {
    id: AuthTokenId,
    kind: asten_domain::TokenKind,
    label: String,
    created_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    expires_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    revoked_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    last_used_at: Option<String>,
}

impl From<AuthToken> for TokenView {
    fn from(token: AuthToken) -> Self {
        Self {
            id: token.id,
            kind: token.kind,
            label: token.label,
            created_at: token.created_at.to_rfc3339(),
            expires_at: token.expires_at.map(|at| at.to_rfc3339()),
            revoked_at: token.revoked_at.map(|at| at.to_rfc3339()),
            last_used_at: token.last_used_at.map(|at| at.to_rfc3339()),
        }
    }
}

#[derive(Deserialize, Default)]
struct TokenQuery {
    #[serde(default)]
    include_revoked: bool,
}

async fn list_tokens(
    State(state): State<ApiState>,
    Extension(client): Extension<AuthenticatedClient>,
    Query(query): Query<TokenQuery>,
) -> Result<Response, ApiError> {
    if let Some(forbidden) = require_admin(&client) {
        return Ok(forbidden);
    }
    let tokens = state.application.list_tokens().await?;
    let views: Vec<TokenView> = tokens
        .into_iter()
        .filter(|token| query.include_revoked || token.revoked_at.is_none())
        .map(TokenView::from)
        .collect();
    Ok(Json(views).into_response())
}

/// A guest may revoke only itself (which `/auth/logout` also does); a desk may
/// revoke anything.
async fn revoke_token(
    State(state): State<ApiState>,
    Extension(client): Extension<AuthenticatedClient>,
    Path(token_id): Path<Uuid>,
) -> Result<Response, ApiError> {
    let own = client.0.token_id == Some(AuthTokenId(token_id));
    if !own {
        if let Some(forbidden) = require_admin(&client) {
            return Ok(forbidden);
        }
    }
    state
        .application
        .revoke_token(AuthTokenId(token_id))
        .await?;
    Ok(StatusCode::NO_CONTENT.into_response())
}
