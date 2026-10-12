//! Local authentication for a local-first product.
//!
//! The Asten API listens on loopback, which is reachable by any web page the
//! writer visits. Without a shared secret, a malicious site could read the
//! manuscript through `fetch("http://127.0.0.1:4317/...")`. The model here:
//!
//! - **Pairing.** The desk shows a short code; a guest (the Word pane, the Docs
//!   sidebar, a browser tab) types it once and receives a long random bearer
//!   token. Only the token's SHA-256 is stored. Codes expire in minutes, are
//!   single-use, and lock after a few failures.
//! - **Owner mode.** For a self-hosted deployment reachable from other machines,
//!   one passphrase (Argon2id) unlocks a session cookie.
//! - **Off.** For development only, and never the default outside `cargo run`.
//!
//! There is no account at Verne, no sync, and nothing here talks to a server.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use argon2::Argon2;
use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use base64::Engine as _;
use rand::{Rng, RngCore};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;
use thiserror::Error;

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthMode {
    /// No authentication. Development only.
    Off,
    /// Loopback pairing codes and bearer tokens. The default.
    #[default]
    Pairing,
    /// Pairing plus an owner passphrase for sessions from other machines.
    Owner,
}

impl AuthMode {
    /// Parse `ASTEN_AUTH`. Unknown values fall back to pairing, never to off.
    #[must_use]
    pub fn parse(value: &str) -> Self {
        match value.trim().to_ascii_lowercase().as_str() {
            "off" | "none" | "disabled" => AuthMode::Off,
            "owner" | "passphrase" => AuthMode::Owner,
            _ => AuthMode::Pairing,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TokenKind {
    /// The desk itself, issued at first run.
    Desk,
    /// A guest paired with a code: Word, Docs, a browser.
    Guest,
    /// An owner-mode session.
    Session,
}

/// A stored token: only the hash is kept.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct TokenRecord {
    pub id: String,
    pub hash: String,
    pub kind: TokenKind,
    pub label: String,
    pub created_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub revoked_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_used_at: Option<String>,
}

impl TokenRecord {
    /// Whether the token may be used at `now` (RFC 3339 strings compare lexically).
    #[must_use]
    pub fn is_valid_at(&self, now: &str) -> bool {
        if self.revoked_at.is_some() {
            return false;
        }
        match &self.expires_at {
            Some(expires) => expires.as_str() > now,
            None => true,
        }
    }
}

const TOKEN_PREFIX: &str = "ast_";

/// A freshly generated token: give `plaintext` to the client once; store `hash`.
#[derive(Clone, Debug)]
pub struct NewToken {
    pub plaintext: String,
    pub hash: String,
}

#[must_use]
pub fn generate_token() -> NewToken {
    let mut bytes = [0u8; 32];
    rand::rng().fill_bytes(&mut bytes);
    let plaintext = format!(
        "{TOKEN_PREFIX}{}",
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
    );
    let hash = hash_token(&plaintext);
    NewToken { plaintext, hash }
}

/// SHA-256 hex of a token. Tokens are high-entropy, so a plain hash suffices.
#[must_use]
pub fn hash_token(plaintext: &str) -> String {
    hex(&Sha256::digest(plaintext.trim().as_bytes()))
}

/// Constant-time comparison of a presented token against a stored hash.
#[must_use]
pub fn verify_token(plaintext: &str, stored_hash: &str) -> bool {
    let presented = hash_token(plaintext);
    presented.as_bytes().ct_eq(stored_hash.as_bytes()).into()
}

/// A short pairing code the desk displays. Six digits, spoken or typed once.
#[must_use]
pub fn generate_pairing_code() -> String {
    let value: u32 = rand::rng().random_range(0..1_000_000);
    format!("{value:06}")
}

/// Pairing codes are low-entropy, so they are hashed with a per-code salt and
/// protected by expiry, single use and attempt limits rather than by the hash.
#[must_use]
pub fn hash_pairing_code(code: &str, salt: &str) -> String {
    hex(&Sha256::digest(
        format!("{salt}:{}", normalise_code(code)).as_bytes(),
    ))
}

#[must_use]
pub fn verify_pairing_code(code: &str, salt: &str, stored_hash: &str) -> bool {
    hash_pairing_code(code, salt)
        .as_bytes()
        .ct_eq(stored_hash.as_bytes())
        .into()
}

fn normalise_code(code: &str) -> String {
    code.chars().filter(|c| c.is_ascii_digit()).collect()
}

#[must_use]
pub fn new_salt() -> String {
    let mut bytes = [0u8; 16];
    rand::rng().fill_bytes(&mut bytes);
    hex(&bytes)
}

#[derive(Debug, Error)]
pub enum PassphraseError {
    #[error("the passphrase must be at least 12 characters")]
    TooShort,
    #[error("could not hash the passphrase: {0}")]
    Hash(String),
}

/// Argon2id PHC string for an owner passphrase.
pub fn hash_passphrase(passphrase: &str) -> Result<String, PassphraseError> {
    if passphrase.chars().count() < 12 {
        return Err(PassphraseError::TooShort);
    }
    let mut bytes = [0u8; 16];
    rand::rng().fill_bytes(&mut bytes);
    let salt =
        SaltString::encode_b64(&bytes).map_err(|error| PassphraseError::Hash(error.to_string()))?;
    Argon2::default()
        .hash_password(passphrase.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|error| PassphraseError::Hash(error.to_string()))
}

#[must_use]
pub fn verify_passphrase(passphrase: &str, phc: &str) -> bool {
    let Ok(parsed) = PasswordHash::new(phc) else {
        return false;
    };
    Argon2::default()
        .verify_password(passphrase.as_bytes(), &parsed)
        .is_ok()
}

/// Extract a bearer token from an `Authorization` header value.
#[must_use]
pub fn bearer(header: &str) -> Option<&str> {
    let (scheme, token) = header.trim().split_once(' ')?;
    if !scheme.eq_ignore_ascii_case("bearer") {
        return None;
    }
    let token = token.trim();
    token.starts_with(TOKEN_PREFIX).then_some(token)
}

/// Extract a named cookie's value from a `Cookie` header.
#[must_use]
pub fn cookie_value<'a>(header: &'a str, name: &str) -> Option<&'a str> {
    header.split(';').map(str::trim).find_map(|pair| {
        let (key, value) = pair.split_once('=')?;
        (key.trim() == name).then_some(value.trim())
    })
}

// ───────────────────────── rate limiting ─────────────────────────

struct Bucket {
    tokens: f64,
    updated: Instant,
}

/// A token-bucket limiter keyed by client (an address, a code id).
const MAX_BUCKETS: usize = 10_000;

pub struct RateLimiter {
    capacity: f64,
    refill_per_second: f64,
    buckets: Mutex<HashMap<String, Bucket>>,
}

impl RateLimiter {
    /// Allow `capacity` bursts, refilling at `per_minute` per minute.
    #[must_use]
    pub fn new(capacity: u32, per_minute: u32) -> Self {
        Self {
            capacity: f64::from(capacity),
            refill_per_second: f64::from(per_minute) / 60.0,
            buckets: Mutex::new(HashMap::new()),
        }
    }

    /// Returns `true` when the request may proceed.
    pub fn check(&self, key: &str) -> bool {
        self.check_at(key, Instant::now())
    }

    fn check_at(&self, key: &str, now: Instant) -> bool {
        let mut buckets = self
            .buckets
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if buckets.len() >= MAX_BUCKETS && !buckets.contains_key(key) {
            // Forget stale keys rather than grow without bound, and if every
            // key is fresh, drop the one touched least recently.
            buckets
                .retain(|_, bucket| now.duration_since(bucket.updated) < Duration::from_secs(600));
            if buckets.len() >= MAX_BUCKETS {
                if let Some(oldest) = buckets
                    .iter()
                    .min_by_key(|(_, bucket)| bucket.updated)
                    .map(|(key, _)| key.clone())
                {
                    buckets.remove(&oldest);
                }
            }
        }
        let bucket = buckets.entry(key.to_owned()).or_insert(Bucket {
            tokens: self.capacity,
            updated: now,
        });
        let elapsed = now.duration_since(bucket.updated).as_secs_f64();
        bucket.tokens = (bucket.tokens + elapsed * self.refill_per_second).min(self.capacity);
        bucket.updated = now;
        if bucket.tokens >= 1.0 {
            bucket.tokens -= 1.0;
            true
        } else {
            false
        }
    }
}

fn hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokens_verify_only_against_their_hash() {
        let token = generate_token();
        assert!(token.plaintext.starts_with("ast_"));
        assert!(verify_token(&token.plaintext, &token.hash));
        assert!(!verify_token(&generate_token().plaintext, &token.hash));
        assert_eq!(
            bearer(&format!("Bearer {}", token.plaintext)),
            Some(token.plaintext.as_str())
        );
        assert_eq!(bearer("Basic abc"), None);
    }

    #[test]
    fn pairing_codes_are_six_digits_and_salted() {
        let code = generate_pairing_code();
        assert_eq!(code.len(), 6);
        let salt = new_salt();
        let hash = hash_pairing_code(&code, &salt);
        assert!(verify_pairing_code(
            &format!("{} {}", &code[..3], &code[3..]),
            &salt,
            &hash
        ));
        assert!(!verify_pairing_code(&code, &new_salt(), &hash));
    }

    #[test]
    fn passphrases_hash_and_verify() {
        let phc = hash_passphrase("a very long passphrase").unwrap();
        assert!(verify_passphrase("a very long passphrase", &phc));
        assert!(!verify_passphrase("wrong", &phc));
        assert!(matches!(
            hash_passphrase("short"),
            Err(PassphraseError::TooShort)
        ));
    }

    #[test]
    fn rate_limiter_stays_bounded() {
        let limiter = RateLimiter::new(1, 60);
        let start = Instant::now();
        for i in 0..(MAX_BUCKETS + 50) {
            assert!(limiter.check_at(&format!("k{i}"), start));
        }
        let buckets = limiter.buckets.lock().unwrap();
        assert!(buckets.len() <= MAX_BUCKETS);
    }

    #[test]
    fn rate_limiter_refills() {
        let limiter = RateLimiter::new(2, 60);
        let start = Instant::now();
        assert!(limiter.check_at("a", start));
        assert!(limiter.check_at("a", start));
        assert!(!limiter.check_at("a", start));
        assert!(limiter.check_at("b", start));
        assert!(limiter.check_at("a", start + Duration::from_secs(2)));
    }

    #[test]
    fn cookies_parse() {
        assert_eq!(
            cookie_value("a=1; asten_session=ast_xyz; b=2", "asten_session"),
            Some("ast_xyz")
        );
        assert_eq!(cookie_value("a=1", "asten_session"), None);
    }

    #[test]
    fn mode_never_falls_back_to_off() {
        assert_eq!(AuthMode::parse("OFF"), AuthMode::Off);
        assert_eq!(AuthMode::parse("garbage"), AuthMode::Pairing);
        assert_eq!(AuthMode::parse("owner"), AuthMode::Owner);
    }
}
