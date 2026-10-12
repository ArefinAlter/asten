//! Pairing, tokens and the owner passphrase. See `asten-auth` for the model.

use asten_auth::{AuthMode, TokenKind as AuthTokenKind};
use asten_domain::{
    AuthToken, AuthTokenId, OwnerCredential, PairingCode, PairingCodeId, ProjectId, TokenKind,
};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

use crate::{Application, ApplicationError, Result};

const PAIRING_TTL_MINUTES: i64 = 10;
const PAIRING_MAX_ATTEMPTS: i64 = 5;
const SESSION_DAYS: i64 = 30;
const STREAM_TICKET_SECONDS: i64 = 60;

/// A short-lived, single-use credential for opening the change stream.
/// `EventSource` cannot send headers, so the browser puts the ticket in the
/// query string; a ticket is worth one connection for one minute, so it does
/// not matter that a reverse proxy may log it.
#[derive(Clone, Debug)]
pub struct StreamTicket {
    pub project: ProjectId,
    pub client: AuthenticatedClient,
    pub expires_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(
    feature = "ts",
    derive(ts_rs::TS),
    ts(export, export_to = "application/")
)]
pub struct StreamTicketIssued {
    pub ticket: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(
    feature = "ts",
    derive(ts_rs::TS),
    ts(export, export_to = "application/")
)]
pub struct AuthStatus {
    pub mode: AuthMode,
    pub owner_configured: bool,
    pub paired_clients: usize,
    /// Whether requests need a token at all.
    pub requires_auth: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(
    feature = "ts",
    derive(ts_rs::TS),
    ts(export, export_to = "application/")
)]
pub struct PairingStart {
    pub id: PairingCodeId,
    /// Shown once in the desk; never stored in clear.
    pub code: String,
    pub label: String,
    /// The kind of token this code will grant.
    pub grants: TokenKind,
    pub expires_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(
    feature = "ts",
    derive(ts_rs::TS),
    ts(export, export_to = "application/")
)]
pub struct PairingComplete {
    /// Given to the client once.
    pub token: String,
    pub token_id: AuthTokenId,
    pub kind: TokenKind,
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(
    feature = "ts",
    derive(ts_rs::TS),
    ts(export, export_to = "application/")
)]
pub struct AuthenticatedClient {
    pub token_id: Option<AuthTokenId>,
    pub kind: TokenKind,
    pub label: String,
}

fn to_domain_kind(kind: AuthTokenKind) -> TokenKind {
    match kind {
        AuthTokenKind::Desk => TokenKind::Desk,
        AuthTokenKind::Guest => TokenKind::Guest,
        AuthTokenKind::Session => TokenKind::Session,
    }
}

impl Application {
    pub async fn auth_status(&self) -> Result<AuthStatus> {
        let tokens = self.repository.list_tokens().await?;
        let now = Utc::now();
        Ok(AuthStatus {
            mode: self.auth_mode,
            owner_configured: self.repository.owner_credential().await?.is_some(),
            paired_clients: tokens.iter().filter(|token| token.is_valid_at(now)).count(),
            requires_auth: self.auth_mode != AuthMode::Off,
        })
    }

    /// Too many attempts from one client key (an address, a code id), or too
    /// many on this route from everyone together? The key is `route:client`.
    #[must_use]
    pub fn rate_check(&self, key: &str) -> bool {
        let route = key.split(':').next().unwrap_or(key);
        let per_client = self.rate_limiter.check(key);
        let global = self.global_limiter.check(route);
        per_client && global
    }

    /// Mint a pairing code the desk shows to a guest (or to another desk).
    pub async fn start_pairing(&self, label: &str, grants: TokenKind) -> Result<PairingStart> {
        let label = label.trim();
        let label = if label.is_empty() {
            if grants == TokenKind::Desk {
                "Another desk"
            } else {
                "A guest"
            }
        } else {
            label
        };
        let code = asten_auth::generate_pairing_code();
        let salt = asten_auth::new_salt();
        let now = Utc::now();
        let record = PairingCode {
            id: PairingCodeId::new(),
            code_hash: asten_auth::hash_pairing_code(&code, &salt),
            salt,
            label: label.to_owned(),
            grants,
            expires_at: now + Duration::minutes(PAIRING_TTL_MINUTES),
            consumed_at: None,
            attempts: 0,
            created_at: now,
        };
        self.repository.insert_pairing_code(&record).await?;
        Ok(PairingStart {
            id: record.id,
            code,
            label: record.label,
            grants,
            expires_at: record.expires_at,
        })
    }

    /// If no client has ever paired, mint a first code for the API log.
    pub async fn bootstrap_pairing_code(&self) -> Result<Option<PairingStart>> {
        if self.auth_mode == AuthMode::Off {
            return Ok(None);
        }
        let tokens = self.repository.list_tokens().await?;
        if tokens.iter().any(|token| token.is_valid_at(Utc::now())) {
            return Ok(None);
        }
        let active = self
            .repository
            .list_pairing_codes()
            .await?
            .into_iter()
            .any(|code| code.consumed_at.is_none() && code.expires_at > Utc::now());
        if active {
            return Ok(None);
        }
        // The first client to pair is the author's own desk.
        self.start_pairing("First pairing", TokenKind::Desk)
            .await
            .map(Some)
    }

    /// Exchange a code for a token. Codes are single-use and lock after a few failures.
    pub async fn complete_pairing(
        &self,
        code: &str,
        label: &str,
        client_key: &str,
    ) -> Result<PairingComplete> {
        if !self.rate_check(&format!("pair:{client_key}")) {
            return Err(ApplicationError::Unauthorized(
                "too many attempts; wait a minute".into(),
            ));
        }
        let now = Utc::now();
        let codes = self.repository.list_pairing_codes().await?;
        let mut matched: Option<PairingCode> = None;
        let mut active: Vec<PairingCode> = Vec::new();
        for record in codes {
            if record.consumed_at.is_some()
                || record.expires_at <= now
                || record.attempts >= PAIRING_MAX_ATTEMPTS
            {
                continue;
            }
            if asten_auth::verify_pairing_code(code, &record.salt, &record.code_hash) {
                matched = Some(record.clone());
            }
            active.push(record);
        }
        let Some(record) = matched else {
            // A wrong guess counts against every open code, so a parallel
            // search cannot try each code five times.
            for record in active {
                self.repository.record_pairing_failure(record.id).await?;
            }
            return Err(ApplicationError::Unauthorized(
                "that code is not valid".into(),
            ));
        };
        // Single use is enforced by the store, not by what we read a moment ago.
        if !self.repository.consume_pairing_code(record.id, now).await? {
            return Err(ApplicationError::Unauthorized(
                "that code is not valid".into(),
            ));
        }
        let label = {
            let trimmed = label.trim();
            if trimmed.is_empty() {
                record.label.clone()
            } else {
                trimmed.to_owned()
            }
        };
        self.issue_token(record.grants, &label, None).await
    }

    async fn issue_token(
        &self,
        kind: TokenKind,
        label: &str,
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<PairingComplete> {
        let minted = asten_auth::generate_token();
        let token = AuthToken {
            id: AuthTokenId::new(),
            hash: minted.hash,
            kind,
            label: label.to_owned(),
            created_at: Utc::now(),
            expires_at,
            revoked_at: None,
            last_used_at: None,
        };
        self.repository.insert_token(&token).await?;
        Ok(PairingComplete {
            token: minted.plaintext,
            token_id: token.id,
            kind,
            label: token.label,
            expires_at,
        })
    }

    /// Issue a desk token directly (for the in-process shell to hand to a browser view).
    pub async fn issue_desk_token(&self, label: &str) -> Result<PairingComplete> {
        self.issue_token(TokenKind::Desk, label, None).await
    }

    /// Resolve a presented bearer token or session cookie. In `Off` mode every
    /// request is the desk.
    pub async fn authenticate(
        &self,
        presented: Option<&str>,
    ) -> Result<Option<AuthenticatedClient>> {
        if self.auth_mode == AuthMode::Off {
            return Ok(Some(AuthenticatedClient {
                token_id: None,
                kind: TokenKind::Desk,
                label: "desk (auth off)".into(),
            }));
        }
        let Some(presented) = presented else {
            return Ok(None);
        };
        let hash = asten_auth::hash_token(presented);
        let Some(mut token) = self.repository.find_token_by_hash(&hash).await? else {
            return Ok(None);
        };
        if !asten_auth::verify_token(presented, &token.hash) || !token.is_valid_at(Utc::now()) {
            return Ok(None);
        }
        // Touch last_used at most once a minute to keep writes rare.
        let stale = token
            .last_used_at
            .is_none_or(|used| Utc::now() - used > Duration::minutes(1));
        if stale {
            token.last_used_at = Some(Utc::now());
            let _ = self.repository.update_token(&token).await;
        }
        Ok(Some(AuthenticatedClient {
            token_id: Some(token.id),
            kind: token.kind,
            label: token.label,
        }))
    }

    pub async fn list_tokens(&self) -> Result<Vec<AuthToken>> {
        self.repository.list_tokens().await
    }

    pub async fn revoke_token(&self, id: AuthTokenId) -> Result<()> {
        let tokens = self.repository.list_tokens().await?;
        let Some(mut token) = tokens.into_iter().find(|token| token.id == id) else {
            return Err(ApplicationError::NotFound { entity: "token" });
        };
        token.revoked_at = Some(Utc::now());
        self.repository.update_token(&token).await
    }

    /// Set the owner passphrase, once. The store enforces "once": two racing
    /// callers cannot both succeed.
    pub async fn setup_owner(&self, passphrase: &str, client_key: &str) -> Result<()> {
        if !self.rate_check(&format!("setup:{client_key}")) {
            return Err(ApplicationError::Unauthorized(
                "too many attempts; wait a minute".into(),
            ));
        }
        let phc = asten_auth::hash_passphrase(passphrase)
            .map_err(|error| ApplicationError::Validation(error.to_string()))?;
        let created = self
            .repository
            .create_owner_credential(&OwnerCredential {
                phc,
                created_at: Utc::now(),
            })
            .await?;
        if created {
            Ok(())
        } else {
            Err(ApplicationError::Conflict(
                "an owner passphrase is already set".into(),
            ))
        }
    }

    /// Replace the owner passphrase after proving the current one.
    pub async fn change_owner_passphrase(
        &self,
        current: &str,
        next: &str,
        client_key: &str,
    ) -> Result<()> {
        if !self.rate_check(&format!("passphrase:{client_key}")) {
            return Err(ApplicationError::Unauthorized(
                "too many attempts; wait a minute".into(),
            ));
        }
        let Some(credential) = self.repository.owner_credential().await? else {
            return Err(ApplicationError::NotFound {
                entity: "owner passphrase",
            });
        };
        if !asten_auth::verify_passphrase(current, &credential.phc) {
            return Err(ApplicationError::Unauthorized(
                "the current passphrase is wrong".into(),
            ));
        }
        let phc = asten_auth::hash_passphrase(next)
            .map_err(|error| ApplicationError::Validation(error.to_string()))?;
        self.repository
            .set_owner_credential(&OwnerCredential {
                phc,
                created_at: Utc::now(),
            })
            .await
    }

    /// Owner login: a session token that expires.
    pub async fn login_owner(&self, passphrase: &str, client_key: &str) -> Result<PairingComplete> {
        if !self.rate_check(&format!("login:{client_key}")) {
            return Err(ApplicationError::Unauthorized(
                "too many attempts; wait a minute".into(),
            ));
        }
        let Some(credential) = self.repository.owner_credential().await? else {
            return Err(ApplicationError::Unauthorized(
                "no owner passphrase is set".into(),
            ));
        };
        if !asten_auth::verify_passphrase(passphrase, &credential.phc) {
            return Err(ApplicationError::Unauthorized(
                "the passphrase is wrong".into(),
            ));
        }
        self.issue_token(
            TokenKind::Session,
            "Owner session",
            Some(Utc::now() + Duration::days(SESSION_DAYS)),
        )
        .await
    }

    /// Revoke the presented token.
    pub async fn logout(&self, presented: &str) -> Result<()> {
        let hash = asten_auth::hash_token(presented);
        if let Some(mut token) = self.repository.find_token_by_hash(&hash).await? {
            token.revoked_at = Some(Utc::now());
            self.repository.update_token(&token).await?;
        }
        Ok(())
    }

    /// Issue a one-minute, single-use ticket for the project's change stream.
    pub async fn issue_stream_ticket(
        &self,
        project: ProjectId,
        client: &AuthenticatedClient,
    ) -> Result<StreamTicketIssued> {
        self.get_project(project).await?;
        let minted = asten_auth::generate_token();
        let expires_at = Utc::now() + Duration::seconds(STREAM_TICKET_SECONDS);
        let mut tickets = self
            .stream_tickets
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let now = Utc::now();
        tickets.retain(|_, ticket| ticket.expires_at > now);
        if tickets.len() >= 10_000 {
            tickets.clear();
        }
        tickets.insert(
            minted.hash,
            StreamTicket {
                project,
                client: client.clone(),
                expires_at,
            },
        );
        Ok(StreamTicketIssued {
            ticket: minted.plaintext,
            expires_at,
        })
    }

    /// Spend a stream ticket. Returns the client it was issued to when the
    /// ticket is live and was minted for this project.
    #[must_use]
    pub fn redeem_stream_ticket(
        &self,
        ticket: &str,
        project: ProjectId,
    ) -> Option<AuthenticatedClient> {
        let hash = asten_auth::hash_token(ticket);
        let mut tickets = self
            .stream_tickets
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let issued = tickets.remove(&hash)?;
        if issued.project != project || issued.expires_at <= Utc::now() {
            return None;
        }
        Some(issued.client)
    }

    /// Remove expired and consumed pairing codes.
    pub async fn sweep_pairing_codes(&self) -> Result<usize> {
        let now = Utc::now();
        let mut removed = 0;
        for code in self.repository.list_pairing_codes().await? {
            if code.consumed_at.is_some() || code.expires_at <= now {
                self.repository.delete_pairing_code(code.id).await?;
                removed += 1;
            }
        }
        Ok(removed)
    }
}

#[allow(dead_code)]
fn _kind_bridge(kind: AuthTokenKind) -> TokenKind {
    to_domain_kind(kind)
}
