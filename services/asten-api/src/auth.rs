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
    http::{HeaderMap, HeaderValue, Request, StatusCode, Uri, header, uri::Authority},
    middleware::Next,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{ApiConfig, ApiError, ApiState, IpNet};

pub const SESSION_COOKIE: &str = "asten_session";

/// A key that identifies the calling client for rate limiting: the peer address.
#[derive(Clone, Debug)]
pub struct ClientKey(pub String);

/// The authenticated caller, inserted by [`require_auth`].
#[derive(Clone, Debug)]
pub struct AuthenticatedClient(pub asten_application::auth::AuthenticatedClient);

/// Derive the client key from the connection, before authentication. Behind a
/// trusted reverse proxy the key is the nearest `X-Forwarded-For` hop that is
/// not itself a trusted proxy; IPv6 clients are keyed by their /64 so a host
/// cannot rotate through its block.
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
        .get_all("x-forwarded-for")
        .iter()
        .map(|value| value.to_str())
        .collect::<Result<Vec<_>, _>>()
        .ok()
        .map(|values| values.join(","));
    let client = resolve_client(peer, forwarded.as_deref(), &config.trusted_proxies);
    let key = client.map_or_else(|| "local".to_owned(), bucket_key);
    request.extensions_mut().insert(ClientKey(key));
    next.run(request).await
}

/// The address a request is counted against: the TCP peer, unless the peer is
/// a trusted proxy. Then it is the rightmost `X-Forwarded-For` hop that is not
/// also a trusted proxy, because the hops a proxy appends are the only ones it
/// vouches for; the leftmost is whatever the client chose to send, and keying
/// on it would hand every client a fresh bucket per request.
fn resolve_client(
    peer: Option<IpAddr>,
    forwarded: Option<&str>,
    trusted: &[IpNet],
) -> Option<IpAddr> {
    let is_proxy = |ip: IpAddr| trusted.iter().any(|net| net.contains(ip));
    let peer = peer?.to_canonical();
    if !is_proxy(peer) {
        return Some(peer);
    }
    let hops = forwarded.into_iter().flat_map(|value| value.rsplit(','));
    for hop in hops {
        // An unknown hop is a trust boundary. Never skip it and accept a
        // potentially client-supplied address farther to the left.
        let Some(hop) = parse_forwarded_hop(hop) else {
            return Some(peer);
        };
        let hop = hop.to_canonical();
        if !is_proxy(hop) {
            return Some(hop);
        }
    }
    Some(peer)
}

/// One `X-Forwarded-For` entry: `203.0.113.9`, `203.0.113.9:4567`,
/// `2001:db8::1` or `[2001:db8::1]:4567`.
fn parse_forwarded_hop(hop: &str) -> Option<IpAddr> {
    let hop = hop.trim().trim_matches('"');
    if let Ok(ip) = hop.parse::<IpAddr>() {
        return Some(ip);
    }
    if let Some(inner) = hop.strip_prefix('[') {
        return inner.split_once(']')?.0.parse().ok();
    }
    let (host, port) = hop.rsplit_once(':')?;
    if port.is_empty() || !port.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    host.parse::<std::net::Ipv4Addr>().ok().map(IpAddr::V4)
}

/// IPv4 clients of a dual-stack listener arrive as `::ffff:a.b.c.d`; they are
/// keyed as the IPv4 address they are, not as one shared /64.
fn bucket_key(ip: IpAddr) -> String {
    match ip.to_canonical() {
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
/// front of both)? The `Origin` header must name the host the request was
/// sent to; without an `Origin` (a navigation, a non-browser client) the
/// answer is yes. Only then is a cookie set or honoured. A cross-origin desk
/// keeps its bearer token, and a same-site page on another port or subdomain,
/// whose form posts the browser would still attach a `SameSite=Strict` cookie
/// to, gets nothing from it.
fn same_origin(headers: &HeaderMap, uri: &Uri) -> bool {
    let Some(origin) = headers.get(header::ORIGIN).and_then(|v| v.to_str().ok()) else {
        return true;
    };
    // HTTP/2 carries the host in `:authority` rather than a `Host` header.
    let host = headers
        .get(header::HOST)
        .and_then(|v| v.to_str().ok())
        .or_else(|| uri.authority().map(Authority::as_str));
    let Some(host) = host else {
        return false;
    };
    origin
        .split_once("://")
        .is_some_and(|(_, origin_host)| origin_host.eq_ignore_ascii_case(host.trim()))
}

/// How a credential reached us. A cookie deserves less trust than a header:
/// the browser attaches it on its own, to requests the page did not write.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Carrier {
    Header,
    Cookie,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Presented {
    token: String,
    carrier: Carrier,
}

fn presented_token(headers: &HeaderMap) -> Option<Presented> {
    if let Some(token) = headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(asten_auth::bearer)
    {
        return Some(Presented {
            token: token.to_owned(),
            carrier: Carrier::Header,
        });
    }
    // HTTP/2 may split one cookie jar over several `Cookie` fields.
    headers
        .get_all(header::COOKIE)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .find_map(|cookie| asten_auth::cookie_value(cookie, SESSION_COOKIE))
        .map(|token| Presented {
            token: token.to_owned(),
            carrier: Carrier::Cookie,
        })
}

/// The credential a request may use: a bearer header always, the session
/// cookie only when the request is same-origin (see [`same_origin`]). A cookie
/// from anywhere else is answered with a 401 rather than ignored, so the
/// failure is visible instead of a silent fall-through.
fn credential(headers: &HeaderMap, uri: &Uri) -> Result<Option<String>, &'static str> {
    match presented_token(headers) {
        Some(Presented {
            carrier: Carrier::Cookie,
            ..
        }) if !same_origin(headers, uri) => {
            Err("the session cookie is honoured only from the desk's own origin")
        }
        Some(presented) => Ok(Some(presented.token)),
        None => Ok(None),
    }
}

/// `ticket=` from a query string.
fn query_ticket(query: Option<&str>) -> Option<String> {
    query.and_then(|query| {
        query
            .split('&')
            .find_map(|pair| pair.strip_prefix("ticket=").map(|ticket| ticket.to_owned()))
    })
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
            return match state
                .application
                .redeem_stream_ticket(&ticket, project)
                .await
            {
                Ok(Some(client)) => {
                    request.extensions_mut().insert(AuthenticatedClient(client));
                    next.run(request).await
                }
                Ok(None) => unauthorized("that stream ticket is spent or expired"),
                Err(error) => ApiError(error).into_response(),
            };
        }
    }
    let presented = match credential(request.headers(), request.uri()) {
        Ok(presented) => presented,
        Err(message) => return unauthorized(message),
    };
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

#[utoipa::path(
    post,
    path = "/projects/{project_id}/stream-ticket",
    tag = "auth",
    summary = "Mint a stream ticket",
    description = "A random ticket bound to this project, valid for sixty seconds and spent on first use, for `GET /projects/{project_id}/stream?ticket=…`. An EventSource cannot send headers, so the ticket stands in for the token; only the ticket ever travels in a URL. It inherits the caller's token kind.",
    params(("project_id" = Uuid, Path, description = "The project")),
    responses(
        (status = 201, description = "The ticket and when it expires", body = asten_application::auth::StreamTicketIssued),
        (status = 401, description = "No valid token or session cookie", body = crate::ErrorResponse),
        (status = 404, description = "No project with that id", body = crate::ErrorResponse)
    ),
    security(("asten_token" = []))
)]
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

#[utoipa::path(
    get,
    path = "/auth/status",
    tag = "auth",
    summary = "Authentication status",
    description = "The auth mode, whether an owner passphrase is set, how many clients are paired and whether a token is needed at all. Public.",
    responses(
        (status = 200, description = "The status", body = AuthStatus)
    )
)]
async fn status(State(state): State<ApiState>) -> Result<Json<AuthStatus>, ApiError> {
    Ok(Json(state.application.auth_status().await?))
}

#[derive(Deserialize, utoipa::ToSchema)]
struct PairRequest {
    /// The six-digit code the desk shows, or the API printed at start-up.
    code: String,
    /// A name for this client, shown in the Guests panel.
    #[serde(default)]
    label: String,
}

#[derive(Serialize, utoipa::ToSchema)]
struct PairResponse {
    #[serde(flatten)]
    paired: PairingComplete,
    /// The session cookie was also set, for browser clients.
    cookie: bool,
}

#[utoipa::path(
    post,
    path = "/auth/pair",
    tag = "auth",
    summary = "Redeem a pairing code",
    description = "Exchanges a six-digit code for a bearer token of the kind the code grants (`guest`, or `desk` for the bootstrap code and codes minted for another desk). The token is shown once. A same-origin caller also receives the `asten_session` cookie (`HttpOnly`, `SameSite=Strict`). Rate limited per client and per route.",
    request_body = PairRequest,
    responses(
        (status = 201, description = "The token, its id and kind, and whether a cookie was set", body = PairResponse),
        (status = 401, description = "The code is wrong, spent or expired, or too many attempts were made", body = crate::ErrorResponse)
    )
)]
async fn pair(
    State(state): State<ApiState>,
    Extension(ClientKey(key)): Extension<ClientKey>,
    headers: HeaderMap,
    uri: Uri,
    Json(input): Json<PairRequest>,
) -> Result<Response, ApiError> {
    let paired = state
        .application
        .complete_pairing(&input.code, &input.label, &key)
        .await?;
    let set_cookie = same_origin(&headers, &uri);
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

#[derive(Deserialize, utoipa::ToSchema)]
struct LoginRequest {
    passphrase: String,
}

#[utoipa::path(
    post,
    path = "/auth/login",
    tag = "auth",
    summary = "Log in with the owner passphrase",
    description = "Answered only in `owner` mode. Issues a `session` token that expires after thirty days; a same-origin caller also receives the cookie. Rate limited.",
    request_body = LoginRequest,
    responses(
        (status = 201, description = "The session token", body = PairResponse),
        (status = 401, description = "The passphrase is wrong or not set, the server is not in owner mode, or too many attempts were made", body = crate::ErrorResponse)
    )
)]
async fn login(
    State(state): State<ApiState>,
    Extension(ClientKey(key)): Extension<ClientKey>,
    headers: HeaderMap,
    uri: Uri,
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
    let set_cookie = same_origin(&headers, &uri);
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

#[derive(Deserialize, utoipa::ToSchema)]
struct SetupOwnerRequest {
    passphrase: String,
}

/// First-run setup of the owner passphrase. Allowed without a token only while
/// no passphrase exists and the server runs in owner mode; afterwards it needs
/// an authenticated caller, and the change route asks for the current one.
#[utoipa::path(
    post,
    path = "/auth/owner/setup",
    tag = "auth",
    summary = "Set the owner passphrase for the first time",
    description = "Allowed without a token only while no passphrase exists and the server runs in `owner` mode. In other modes the caller must present a desk or owner-session token. Once a passphrase is set, change it with `PUT /auth/owner/passphrase`. At least twelve characters; hashed with argon2id. Rate limited.",
    request_body = SetupOwnerRequest,
    responses(
        (status = 201, description = "The passphrase is set"),
        (status = 401, description = "No token where one is needed, or too many attempts were made", body = crate::ErrorResponse),
        (status = 403, description = "A guest token may not do this; use the desk", body = crate::ErrorResponse),
        (status = 409, description = "An owner passphrase is already set", body = crate::ErrorResponse),
        (status = 422, description = "The passphrase is shorter than twelve characters", body = crate::ErrorResponse)
    ),
    security((), ("asten_token" = []))
)]
async fn setup_owner(
    State(state): State<ApiState>,
    Extension(ClientKey(key)): Extension<ClientKey>,
    headers: HeaderMap,
    uri: Uri,
    Json(input): Json<SetupOwnerRequest>,
) -> Result<Response, ApiError> {
    let status = state.application.auth_status().await?;
    if status.owner_configured {
        return Err(ApiError(asten_application::ApplicationError::Conflict(
            "an owner passphrase is already set".into(),
        )));
    }
    if state.config.auth_mode != AuthMode::Owner {
        let presented = match credential(&headers, &uri) {
            Ok(presented) => presented,
            Err(message) => return Ok(unauthorized(message)),
        };
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

#[derive(Deserialize, utoipa::ToSchema)]
struct ChangePassphraseRequest {
    current: String,
    next: String,
}

#[utoipa::path(
    put,
    path = "/auth/owner/passphrase",
    tag = "auth",
    summary = "Change the owner passphrase",
    description = "Asks for the current passphrase. Desk and owner-session tokens only. Rate limited.",
    request_body = ChangePassphraseRequest,
    responses(
        (status = 204, description = "Changed"),
        (status = 401, description = "No valid token, the current passphrase is wrong, or too many attempts were made", body = crate::ErrorResponse),
        (status = 403, description = "A guest token may not do this; use the desk", body = crate::ErrorResponse),
        (status = 404, description = "No owner passphrase is set", body = crate::ErrorResponse),
        (status = 422, description = "The new passphrase is shorter than twelve characters", body = crate::ErrorResponse)
    ),
    security(("asten_token" = []))
)]
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

#[utoipa::path(
    get,
    path = "/auth/me",
    tag = "auth",
    summary = "Who am I",
    description = "The token id, kind and label behind the presented credential.",
    responses(
        (status = 200, description = "The caller", body = asten_application::auth::AuthenticatedClient),
        (status = 401, description = "No valid token or session cookie", body = crate::ErrorResponse)
    ),
    security(("asten_token" = []))
)]
async fn me(
    Extension(AuthenticatedClient(client)): Extension<AuthenticatedClient>,
) -> Json<asten_application::auth::AuthenticatedClient> {
    Json(client)
}

#[utoipa::path(
    post,
    path = "/auth/logout",
    tag = "auth",
    summary = "Log out",
    description = "Revokes the presented token and clears the session cookie.",
    responses(
        (status = 204, description = "Logged out"),
        (status = 401, description = "No valid token or session cookie", body = crate::ErrorResponse)
    ),
    security(("asten_token" = []))
)]
async fn logout(State(state): State<ApiState>, headers: HeaderMap) -> Result<Response, ApiError> {
    // `require_auth` has already refused a cookie from another origin.
    if let Some(presented) = presented_token(&headers) {
        state.application.logout(&presented.token).await?;
    }
    let mut response = StatusCode::NO_CONTENT.into_response();
    response
        .headers_mut()
        .insert(header::SET_COOKIE, clear_cookie());
    Ok(response)
}

#[derive(Deserialize, Default, utoipa::ToSchema)]
struct StartPairingRequest {
    #[serde(default)]
    label: String,
    /// `guest` (default) or `desk`. A desk code grants a token that may manage
    /// pairing; a guest code grants one that may only read and write the story.
    #[serde(default)]
    grants: Option<TokenKind>,
}

#[utoipa::path(
    post,
    path = "/auth/pairing-codes",
    tag = "auth",
    summary = "Mint a pairing code",
    description = "A six-digit code valid for ten minutes and single use; five wrong guesses lock it. `grants` chooses the kind of token the code yields: `guest` (default) or `desk`. Desk and owner-session tokens only.",
    request_body = StartPairingRequest,
    responses(
        (status = 201, description = "The code, shown once, and when it expires", body = asten_application::auth::PairingStart),
        (status = 401, description = "No valid token or session cookie", body = crate::ErrorResponse),
        (status = 403, description = "A guest token may not do this; use the desk", body = crate::ErrorResponse)
    ),
    security(("asten_token" = []))
)]
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

/// A paired client as the Guests panel lists it; never the token itself.
#[derive(Serialize, utoipa::ToSchema)]
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

#[derive(Deserialize, Default, utoipa::IntoParams)]
#[into_params(parameter_in = Query)]
struct TokenQuery {
    /// Also list tokens that were revoked.
    #[serde(default)]
    include_revoked: bool,
}

#[utoipa::path(
    get,
    path = "/auth/tokens",
    tag = "auth",
    summary = "List paired clients",
    description = "Desk and owner-session tokens only. Revoked tokens are hidden unless `include_revoked` is set. Hashes are never returned.",
    params(TokenQuery),
    responses(
        (status = 200, description = "The tokens", body = Vec<TokenView>),
        (status = 401, description = "No valid token or session cookie", body = crate::ErrorResponse),
        (status = 403, description = "A guest token may not do this; use the desk", body = crate::ErrorResponse)
    ),
    security(("asten_token" = []))
)]
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
#[utoipa::path(
    delete,
    path = "/auth/tokens/{token_id}",
    tag = "auth",
    summary = "Revoke a token",
    description = "A guest may revoke only itself (which `POST /auth/logout` also does); a desk or owner session may revoke anything. Revocation is immediate.",
    params(("token_id" = Uuid, Path, description = "The token")),
    responses(
        (status = 204, description = "Revoked"),
        (status = 401, description = "No valid token or session cookie", body = crate::ErrorResponse),
        (status = 403, description = "A guest token may not do this; use the desk", body = crate::ErrorResponse),
        (status = 404, description = "No token with that id", body = crate::ErrorResponse)
    ),
    security(("asten_token" = []))
)]
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

#[cfg(test)]
mod tests {
    use super::*;

    fn ip(text: &str) -> IpAddr {
        text.parse().unwrap()
    }

    fn nets(texts: &[&str]) -> Vec<IpNet> {
        texts.iter().map(|t| IpNet::parse(t).unwrap()).collect()
    }

    #[test]
    fn without_a_trusted_proxy_the_peer_is_the_client() {
        let forged = Some("203.0.113.9");
        assert_eq!(
            resolve_client(Some(ip("198.51.100.4")), forged, &[]),
            Some(ip("198.51.100.4"))
        );
        assert_eq!(resolve_client(None, forged, &[]), None);
    }

    #[test]
    fn behind_a_trusted_proxy_the_client_is_the_hop_the_proxy_appended() {
        let trusted = nets(&["10.0.0.0/8"]);
        let proxy = Some(ip("10.0.0.5"));
        // A client that sends its own X-Forwarded-For cannot pick its key.
        assert_eq!(
            resolve_client(proxy, Some("1.2.3.4, 203.0.113.9"), &trusted),
            Some(ip("203.0.113.9"))
        );
        // Two proxies in a row: skip both, take the hop before them.
        assert_eq!(
            resolve_client(proxy, Some("203.0.113.9, 10.0.0.7"), &trusted),
            Some(ip("203.0.113.9"))
        );
        // No header, or nothing but proxies: the proxy itself.
        assert_eq!(resolve_client(proxy, None, &trusted), proxy);
        assert_eq!(resolve_client(proxy, Some("10.0.0.7"), &trusted), proxy);
        // Entries with ports and brackets still parse.
        assert_eq!(
            resolve_client(proxy, Some("203.0.113.9:4567"), &trusted),
            Some(ip("203.0.113.9"))
        );
        assert_eq!(
            resolve_client(proxy, Some("[2001:db8::1]:4567"), &trusted),
            Some(ip("2001:db8::1"))
        );
        // An unparseable entry stops the trusted walk.
        assert_eq!(
            resolve_client(proxy, Some("203.0.113.9, unknown"), &trusted),
            proxy
        );
    }

    #[test]
    fn mapped_ipv4_peers_match_ipv4_proxy_rules_and_keys() {
        let trusted = nets(&["10.0.0.5"]);
        assert_eq!(
            resolve_client(Some(ip("::ffff:10.0.0.5")), Some("203.0.113.9"), &trusted),
            Some(ip("203.0.113.9"))
        );
        assert_eq!(bucket_key(ip("::ffff:203.0.113.9")), "203.0.113.9");
        assert_eq!(bucket_key(ip("203.0.113.9")), "203.0.113.9");
        assert_eq!(bucket_key(ip("2001:db8:1:2:3:4:5:6")), "2001:db8:1:2::/64");
        assert_eq!(
            bucket_key(ip("2001:db8:1:2:ffff::1")),
            bucket_key(ip("2001:db8:1:2::1"))
        );
    }

    fn headers(pairs: &[(&str, &str)]) -> HeaderMap {
        let mut map = HeaderMap::new();
        for (name, value) in pairs {
            map.append(
                axum::http::HeaderName::from_bytes(name.as_bytes()).unwrap(),
                HeaderValue::from_str(value).unwrap(),
            );
        }
        map
    }

    #[test]
    fn same_origin_compares_origin_with_host_or_authority() {
        let uri: Uri = "/api/v1/auth/pair".parse().unwrap();
        assert!(same_origin(&headers(&[("host", "localhost:4398")]), &uri));
        assert!(same_origin(
            &headers(&[
                ("host", "localhost:4398"),
                ("origin", "http://LocalHost:4398")
            ]),
            &uri
        ));
        assert!(!same_origin(
            &headers(&[
                ("host", "localhost:4398"),
                ("origin", "http://localhost:8080")
            ]),
            &uri
        ));
        assert!(!same_origin(
            &headers(&[
                ("host", "desk.example.net"),
                ("origin", "https://evil.example.net")
            ]),
            &uri
        ));
        assert!(!same_origin(
            &headers(&[("host", "localhost:4398"), ("origin", "null")]),
            &uri
        ));
        assert!(!same_origin(
            &headers(&[("origin", "http://localhost:4398")]),
            &uri
        ));
        let h2: Uri = "https://desk.example.net/api/v1/auth/pair".parse().unwrap();
        assert!(same_origin(
            &headers(&[("origin", "https://desk.example.net")]),
            &h2
        ));
        assert!(!same_origin(
            &headers(&[("origin", "https://desk.example.org")]),
            &h2
        ));
    }

    #[test]
    fn a_bearer_header_wins_and_a_split_cookie_jar_is_read() {
        let both = headers(&[
            ("authorization", "Bearer ast_header"),
            ("cookie", "asten_session=ast_cookie"),
        ]);
        assert_eq!(
            presented_token(&both),
            Some(Presented {
                token: "ast_header".into(),
                carrier: Carrier::Header,
            })
        );
        let split = headers(&[
            ("cookie", "theme=night"),
            ("cookie", "asten_session=ast_cookie"),
        ]);
        assert_eq!(
            presented_token(&split),
            Some(Presented {
                token: "ast_cookie".into(),
                carrier: Carrier::Cookie,
            })
        );
        let junk = headers(&[("authorization", "Bearer ast_x y"), ("cookie", "a=1")]);
        assert_eq!(presented_token(&junk), None);
    }

    #[test]
    fn a_cookie_counts_only_from_the_same_origin() {
        let uri: Uri = "/api/v1/auth/logout".parse().unwrap();
        let foreign = headers(&[
            ("host", "localhost:4398"),
            ("origin", "http://localhost:8080"),
            ("cookie", "asten_session=ast_cookie"),
        ]);
        assert!(
            credential(&foreign, &uri)
                .unwrap_err()
                .contains("own origin")
        );
        let own = headers(&[
            ("host", "localhost:4398"),
            ("origin", "http://localhost:4398"),
            ("cookie", "asten_session=ast_cookie"),
        ]);
        assert_eq!(
            credential(&own, &uri).unwrap().as_deref(),
            Some("ast_cookie")
        );
        let bearer = headers(&[
            ("host", "localhost:4398"),
            ("origin", "http://localhost:8080"),
            ("authorization", "Bearer ast_header"),
        ]);
        assert_eq!(
            credential(&bearer, &uri).unwrap().as_deref(),
            Some("ast_header")
        );
        assert_eq!(credential(&headers(&[]), &uri).unwrap(), None);
    }

    #[test]
    fn the_stream_route_and_its_ticket_are_recognised_exactly() {
        let id = Uuid::now_v7();
        assert_eq!(
            stream_project(&format!("/api/v1/projects/{id}/stream")),
            Some(ProjectId(id))
        );
        assert_eq!(
            stream_project(&format!("/projects/{id}/stream")),
            Some(ProjectId(id))
        );
        assert_eq!(
            stream_project(&format!("/api/v1/projects/{id}/stream-ticket")),
            None
        );
        assert_eq!(stream_project("/api/v1/projects/not-a-uuid/stream"), None);
        assert_eq!(
            query_ticket(Some("ticket=ast_abc")).as_deref(),
            Some("ast_abc")
        );
        assert_eq!(
            query_ticket(Some("a=1&ticket=ast_abc&b=2")).as_deref(),
            Some("ast_abc")
        );
        assert_eq!(query_ticket(Some("xticket=ast_abc")), None);
        assert_eq!(query_ticket(None), None);
    }
}
