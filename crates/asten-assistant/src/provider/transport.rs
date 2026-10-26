//! Sending a request and reading the stream.
//!
//! One TLS client (rustls, no redirects, no proxy, no cookies) shared by every
//! provider. Connection errors and 408/409/429/5xx answers are retried with
//! exponential backoff and jitter, honouring `retry-after`; nothing is retried
//! once the answer has begun. The response is read as it arrives, with an idle
//! timeout and a size limit, and any await ends at once when the writer
//! cancels.

use std::{sync::Once, time::Duration};

use futures::StreamExt;
use reqwest::{
    StatusCode,
    header::{AUTHORIZATION, HeaderMap, HeaderName, HeaderValue},
    redirect::Policy,
};
use serde_json::Value;
use tokio_util::sync::CancellationToken;

use super::{
    Auth, Completion, Decoder, Fault, Piece, Wire,
    sse::{SseEvent, SseParser},
};
use crate::{
    error::{AssistantError, Result, truncate},
    secret::Secret,
};

/// Largest request body sent.
pub const MAX_REQUEST_BYTES: usize = 768 * 1024;
/// Largest answer read.
pub const MAX_RESPONSE_BYTES: usize = 8 * 1024 * 1024;

static TLS: Once = Once::new();

/// The shared HTTP client and its limits.
#[derive(Clone)]
pub struct Transport {
    client: reqwest::Client,
    backoff: Duration,
    idle: Duration,
    retries: u32,
}

impl Transport {
    /// A client with production limits.
    pub fn new() -> Result<Self> {
        Self::with_limits(Duration::from_millis(800), Duration::from_secs(90), 3)
    }

    /// A client with explicit limits (tests use tiny ones).
    pub fn with_limits(backoff: Duration, idle: Duration, retries: u32) -> Result<Self> {
        TLS.call_once(|| {
            let _ = rustls::crypto::ring::default_provider().install_default();
        });
        let client = reqwest::Client::builder()
            .redirect(Policy::none())
            .connect_timeout(Duration::from_secs(15))
            .user_agent(concat!("Asten/", env!("CARGO_PKG_VERSION")))
            .build()
            .map_err(|error| {
                AssistantError::Network(format!("could not start the HTTP client: {error}"))
            })?;
        Ok(Self {
            client,
            backoff,
            idle,
            retries,
        })
    }

    /// Send `wire` and decode the stream. `on_piece` sees text as it arrives.
    pub async fn stream(
        &self,
        provider_name: &str,
        wire: &Wire,
        key: Option<&Secret>,
        cancel: &CancellationToken,
        mut decoder: Box<dyn Decoder>,
        on_piece: &mut (dyn FnMut(Piece) + Send),
    ) -> Result<Completion> {
        let body = wire.body_bytes();
        if body.len() > MAX_REQUEST_BYTES {
            return Err(AssistantError::Invalid(format!(
                "The request is {} KB, over the {} KB limit. Pick a shorter passage.",
                body.len() / 1024,
                MAX_REQUEST_BYTES / 1024
            )));
        }
        let scrub = |text: &str| match key {
            Some(secret) => secret.scrub(text),
            None => text.to_owned(),
        };
        let response = self
            .send_with_retries(provider_name, wire, key, &body, cancel, &scrub)
            .await?;

        let is_json = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .is_some_and(|v| v.starts_with("application/json"));
        let to_provider_error = |fault: Fault| -> AssistantError {
            match fault {
                Fault::Api {
                    status, message, ..
                } => AssistantError::Provider {
                    provider: provider_name.to_owned(),
                    status,
                    message: truncate(&scrub(&message), 400),
                },
                Fault::Malformed(message) => AssistantError::Provider {
                    provider: provider_name.to_owned(),
                    status: 0,
                    message,
                },
            }
        };

        let mut pieces: Vec<Piece> = Vec::new();
        let mut total = 0usize;
        if is_json {
            let bytes = self.read_all(response, cancel, MAX_RESPONSE_BYTES).await?;
            let event = SseEvent {
                event: None,
                data: String::from_utf8_lossy(&bytes).into_owned(),
            };
            decoder
                .event(&event, &mut pieces)
                .map_err(to_provider_error)?;
            for piece in pieces.drain(..) {
                on_piece(piece);
            }
            return decoder.finish().map_err(to_provider_error);
        }

        let mut parser = SseParser::new();
        let mut stream = response.bytes_stream();
        loop {
            let next = tokio::select! {
                () = cancel.cancelled() => return Err(AssistantError::Cancelled),
                next = tokio::time::timeout(self.idle, stream.next()) => next,
            };
            let chunk = match next {
                Err(_) => {
                    return Err(AssistantError::Network(format!(
                        "{provider_name} stopped sending for {} seconds",
                        self.idle.as_secs()
                    )));
                }
                Ok(None) => break,
                Ok(Some(Err(error))) => {
                    return Err(AssistantError::Network(scrub(&format!(
                        "The connection to {provider_name} broke: {}",
                        error.without_url()
                    ))));
                }
                Ok(Some(Ok(bytes))) => bytes,
            };
            total += chunk.len();
            if total > MAX_RESPONSE_BYTES {
                return Err(AssistantError::Network(format!(
                    "{provider_name} sent more than {} MB; stopped",
                    MAX_RESPONSE_BYTES / 1024 / 1024
                )));
            }
            let events = parser
                .push(&chunk)
                .map_err(|error| AssistantError::Network(error.0.to_owned()))?;
            for event in events {
                decoder
                    .event(&event, &mut pieces)
                    .map_err(to_provider_error)?;
                for piece in pieces.drain(..) {
                    on_piece(piece);
                }
            }
        }
        if let Some(event) = parser.finish() {
            decoder
                .event(&event, &mut pieces)
                .map_err(to_provider_error)?;
            for piece in pieces.drain(..) {
                on_piece(piece);
            }
        }
        decoder.finish().map_err(to_provider_error)
    }

    async fn read_all(
        &self,
        response: reqwest::Response,
        cancel: &CancellationToken,
        limit: usize,
    ) -> Result<Vec<u8>> {
        let mut stream = response.bytes_stream();
        let mut out = Vec::new();
        loop {
            let next = tokio::select! {
                () = cancel.cancelled() => return Err(AssistantError::Cancelled),
                next = tokio::time::timeout(self.idle, stream.next()) => next,
            };
            match next {
                Err(_) => {
                    return Err(AssistantError::Network(
                        "The provider stopped sending".into(),
                    ));
                }
                Ok(None) => return Ok(out),
                Ok(Some(Err(error))) => {
                    return Err(AssistantError::Network(format!(
                        "The connection broke: {}",
                        error.without_url()
                    )));
                }
                Ok(Some(Ok(bytes))) => {
                    out.extend_from_slice(&bytes);
                    if out.len() > limit {
                        out.truncate(limit);
                        return Ok(out);
                    }
                }
            }
        }
    }

    async fn send_with_retries(
        &self,
        provider_name: &str,
        wire: &Wire,
        key: Option<&Secret>,
        body: &[u8],
        cancel: &CancellationToken,
        scrub: &(dyn Fn(&str) -> String + Sync),
    ) -> Result<reqwest::Response> {
        let mut headers = HeaderMap::new();
        for (name, value) in &wire.headers {
            if let (Ok(name), Ok(value)) = (
                HeaderName::from_bytes(name.as_bytes()),
                HeaderValue::from_str(value),
            ) {
                headers.insert(name, value);
            }
        }
        if let Some(secret) = key {
            let make = |text: String| {
                HeaderValue::from_str(&text)
                    .map(|mut value| {
                        value.set_sensitive(true);
                        value
                    })
                    .map_err(|_| {
                        AssistantError::Invalid(
                            "The key contains characters a header cannot carry".into(),
                        )
                    })
            };
            match wire.auth {
                Auth::None => {}
                Auth::Bearer => {
                    headers.insert(AUTHORIZATION, make(format!("Bearer {}", secret.expose()))?);
                }
                Auth::Header(name) => {
                    headers.insert(
                        HeaderName::from_static(name),
                        make(secret.expose().to_owned())?,
                    );
                }
            }
        }
        let mut attempt = 0u32;
        loop {
            let request = self
                .client
                .post(wire.url.clone())
                .headers(headers.clone())
                .body(body.to_vec());
            let sent = tokio::select! {
                () = cancel.cancelled() => return Err(AssistantError::Cancelled),
                sent = request.send() => sent,
            };
            let delay = match sent {
                Ok(response) if response.status().is_success() => return Ok(response),
                Ok(response) => {
                    let status = response.status();
                    let retry_after = retry_after(response.headers());
                    let text = self
                        .read_all(response, cancel, 16 * 1024)
                        .await
                        .unwrap_or_default();
                    let message = scrub(&error_message(&text));
                    if attempt < self.retries && retryable(status, &message) {
                        Some(retry_after.unwrap_or_else(|| self.delay(attempt)))
                    } else {
                        return Err(AssistantError::Provider {
                            provider: provider_name.to_owned(),
                            status: status.as_u16(),
                            message: truncate(&message, 400),
                        });
                    }
                }
                Err(error) => {
                    if attempt < self.retries && (error.is_connect() || error.is_timeout()) {
                        Some(self.delay(attempt))
                    } else {
                        let what = if error.is_connect() {
                            "could not connect"
                        } else if error.is_timeout() {
                            "timed out"
                        } else {
                            "failed"
                        };
                        return Err(AssistantError::Network(scrub(&format!(
                            "The request to {provider_name} {what}: {}",
                            error.without_url()
                        ))));
                    }
                }
            };
            if let Some(delay) = delay {
                tracing::debug!(attempt, ?delay, "retrying a provider request");
                tokio::select! {
                    () = cancel.cancelled() => return Err(AssistantError::Cancelled),
                    () = tokio::time::sleep(delay) => {}
                }
            }
            attempt += 1;
        }
    }

    fn delay(&self, attempt: u32) -> Duration {
        let base = self.backoff.saturating_mul(1 << attempt.min(6));
        let jitter = rand::random::<u64>() % (self.backoff.as_millis() as u64 / 2 + 1);
        (base + Duration::from_millis(jitter)).min(Duration::from_secs(30))
    }
}

fn retryable(status: StatusCode, message: &str) -> bool {
    if matches!(status.as_u16(), 408 | 409 | 500 | 502 | 503 | 504 | 529) {
        return true;
    }
    if status.as_u16() == 429 {
        // A spent quota does not come back by waiting.
        let lower = message.to_lowercase();
        return !(lower.contains("insufficient_quota")
            || lower.contains("billing")
            || lower.contains("exceeded your current quota"));
    }
    false
}

fn retry_after(headers: &HeaderMap) -> Option<Duration> {
    if let Some(ms) = headers
        .get("retry-after-ms")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.trim().parse::<u64>().ok())
    {
        return Some(Duration::from_millis(ms.min(30_000)));
    }
    headers
        .get(reqwest::header::RETRY_AFTER)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.trim().parse::<f64>().ok())
        .map(|secs| Duration::from_millis((secs.clamp(0.0, 30.0) * 1000.0) as u64))
}

/// Pull the provider's own message out of an error body.
#[must_use]
pub fn error_message(body: &[u8]) -> String {
    let text = String::from_utf8_lossy(body);
    if let Ok(json) = serde_json::from_str::<Value>(&text) {
        let candidates = [
            json.pointer("/error/message"),
            json.get("message"),
            json.pointer("/error/error/message"),
            json.pointer("/detail"),
            json.get("error").filter(|e| e.is_string()),
        ];
        for candidate in candidates.into_iter().flatten() {
            if let Some(message) = candidate.as_str() {
                let code = ["/error/type", "/error/code", "/error/status"]
                    .iter()
                    .find_map(|pointer| json.pointer(pointer).and_then(Value::as_str));
                return match code {
                    Some(code) => format!("{message} ({code})"),
                    None => message.to_owned(),
                };
            }
        }
    }
    let trimmed = text.trim();
    if trimmed.is_empty() {
        "no message".to_owned()
    } else {
        truncate(trimmed, 300)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_bodies_are_read_in_each_providers_shape() {
        assert_eq!(
            error_message(
                br#"{"type":"error","error":{"type":"overloaded_error","message":"Overloaded"}}"#
            ),
            "Overloaded (overloaded_error)"
        );
        assert_eq!(
            error_message(br#"{"error":{"message":"Incorrect API key","type":"invalid_request_error","code":"invalid_api_key"}}"#),
            "Incorrect API key (invalid_request_error)"
        );
        assert_eq!(
            error_message(br#"{"error":{"code":429,"message":"Quota exceeded","status":"RESOURCE_EXHAUSTED"}}"#),
            "Quota exceeded (RESOURCE_EXHAUSTED)"
        );
        assert_eq!(
            error_message(b"<html>bad gateway</html>"),
            "<html>bad gateway</html>"
        );
        assert_eq!(error_message(b""), "no message");
    }

    #[test]
    fn a_spent_quota_is_not_retried() {
        assert!(retryable(StatusCode::TOO_MANY_REQUESTS, "slow down"));
        assert!(!retryable(
            StatusCode::TOO_MANY_REQUESTS,
            "You exceeded your current quota"
        ));
        assert!(retryable(StatusCode::from_u16(529).unwrap(), "overloaded"));
        assert!(!retryable(StatusCode::UNAUTHORIZED, "bad key"));
        assert!(!retryable(StatusCode::BAD_REQUEST, "bad"));
    }

    #[test]
    fn retry_after_is_honoured_and_capped() {
        let mut headers = HeaderMap::new();
        headers.insert("retry-after", HeaderValue::from_static("2"));
        assert_eq!(retry_after(&headers), Some(Duration::from_secs(2)));
        headers.insert("retry-after", HeaderValue::from_static("9999"));
        assert_eq!(retry_after(&headers), Some(Duration::from_secs(30)));
        headers.insert("retry-after-ms", HeaderValue::from_static("150"));
        assert_eq!(retry_after(&headers), Some(Duration::from_millis(150)));
    }
}
