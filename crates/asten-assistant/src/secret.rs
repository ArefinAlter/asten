//! The only type that holds a key.
//!
//! No `Serialize`, no `Display`, a redacted `Debug`, and the memory is
//! overwritten when the value is dropped. Code that needs the text asks for it
//! with [`Secret::expose`] at the last moment, to put it in an HTTP header.

use std::fmt;

use zeroize::Zeroizing;

/// A provider key or a passphrase.
#[derive(Clone)]
pub struct Secret(Zeroizing<String>);

impl Secret {
    /// Wrap a value, trimming surrounding white space.
    #[must_use]
    pub fn new(value: String) -> Self {
        let trimmed = value.trim();
        if trimmed.len() == value.len() {
            return Self(Zeroizing::new(value));
        }
        let copy = trimmed.to_owned();
        let _wipe = Zeroizing::new(value);
        Self(Zeroizing::new(copy))
    }

    /// The secret text. Use it only to build a request header.
    #[must_use]
    pub fn expose(&self) -> &str {
        self.0.as_str()
    }

    /// True when nothing is held.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// The last four characters, which the interface may show.
    #[must_use]
    pub fn last4(&self) -> String {
        let chars: Vec<char> = self.0.chars().collect();
        let from = chars.len().saturating_sub(4);
        chars[from..].iter().collect()
    }

    /// Replace every occurrence of this secret in `text` with `***`.
    #[must_use]
    pub fn scrub(&self, text: &str) -> String {
        if self.0.len() < 6 {
            return text.to_owned();
        }
        text.replace(self.0.as_str(), "***")
    }
}

impl fmt::Debug for Secret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Secret(<redacted>)")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_never_shows_the_value() {
        let secret = Secret::new("  ASTEN-CANARY-1234567890  ".into());
        assert_eq!(secret.expose(), "ASTEN-CANARY-1234567890");
        assert!(!format!("{secret:?}").contains("CANARY"));
        assert_eq!(secret.last4(), "7890");
    }

    #[test]
    fn scrub_removes_the_value_from_text() {
        let secret = Secret::new("ASTEN-CANARY-abcdef".into());
        let text = "bad key ASTEN-CANARY-abcdef given";
        assert_eq!(secret.scrub(text), "bad key *** given");
    }
}
