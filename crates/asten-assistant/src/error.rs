//! Errors. Messages are shown to the writer and written to logs, so none of
//! them may carry a key, a URL query string or a response body in full.

use thiserror::Error;

/// Everything that can stop an assistant run or a configuration change.
#[derive(Debug, Error)]
pub enum AssistantError {
    /// The master switch is off; no provider is ever contacted.
    #[error("AI providers are switched off in Settings")]
    Disabled,
    /// Offline mode is on and the provider is not on this machine.
    #[error("Offline mode is on: nothing leaves this machine")]
    Offline,
    /// The provider is unknown, switched off or not consented to.
    #[error("{0}")]
    NotReady(String),
    /// A spend cap is reached or set to zero.
    #[error("{0}")]
    Cap(String),
    /// The configuration or a request is invalid.
    #[error("{0}")]
    Invalid(String),
    /// The provider answered with an error.
    #[error("{provider} answered {status}: {message}")]
    Provider {
        /// Provider display name.
        provider: String,
        /// HTTP status, or 0 for an error event inside a stream.
        status: u16,
        /// The provider's own message, truncated and scrubbed.
        message: String,
    },
    /// The network failed before or during the response.
    #[error("{0}")]
    Network(String),
    /// The run was cancelled by the writer.
    #[error("Cancelled")]
    Cancelled,
    /// The key store failed.
    #[error("{0}")]
    Keys(String),
    /// A file could not be read or written.
    #[error("{0}")]
    Io(String),
    /// The local engine failed.
    #[error("{0}")]
    Engine(String),
}

/// Shorthand used across the crate.
pub type Result<T, E = AssistantError> = std::result::Result<T, E>;

impl From<std::io::Error> for AssistantError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error.to_string())
    }
}

/// Cut a string to at most `max` characters on a character boundary.
#[must_use]
pub fn truncate(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_owned();
    }
    let mut out: String = text.chars().take(max).collect();
    out.push('…');
    out
}
