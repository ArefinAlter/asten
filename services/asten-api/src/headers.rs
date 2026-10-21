//! Security headers for every response: a Content-Security-Policy fitted to
//! the desk the API serves, a stricter one for API responses, HSTS behind TLS,
//! and cache rules that keep manuscripts out of disk caches while letting the
//! content-addressed assets under `/assets/` live there for a year.

use std::{path::Path, sync::Arc};

use axum::{
    http::{HeaderName, HeaderValue, StatusCode, header},
    response::Response,
};
use base64::Engine as _;
use sha2::{Digest, Sha256};

use crate::{ApiConfig, origin_allowed};

/// The desk's policy. Scripts come from the bundle only, plus the one inline
/// bootstrap in `index.html` that applies the saved theme before paint, which
/// is allowed by hash rather than by `'unsafe-inline'`. Styles keep
/// `'unsafe-inline'` because the editor (Tiptap) injects a `<style>` element
/// at start-up; React's `style` props would not need it. Fonts and images may
/// be inlined by the bundler as `data:` URLs, and image previews are `blob:`.
/// The desk talks only to its own origin, is never framed, and submits no
/// form anywhere else.
const DESK_POLICY: &str = "default-src 'self'; \
    script-src 'self'{scripts}; \
    style-src 'self' 'unsafe-inline'; \
    font-src 'self' data:; \
    img-src 'self' data: blob:; \
    connect-src 'self'; \
    manifest-src 'self'; \
    worker-src 'self'; \
    frame-src 'none'; \
    object-src 'none'; \
    base-uri 'none'; \
    form-action 'self'; \
    frame-ancestors 'none'";

/// API responses are data, never a document: nothing may load from them and
/// nothing may frame them.
const API_POLICY: &str = "default-src 'none'; frame-ancestors 'none'";

/// One year, which is what `immutable` assets and HSTS both want.
const ONE_YEAR: &str = "31536000";

/// Headers decided once at start-up and stamped on every response.
#[derive(Debug)]
pub struct SecurityPolicy {
    config: Arc<ApiConfig>,
    desk_csp: HeaderValue,
    api_csp: HeaderValue,
    hsts: Option<HeaderValue>,
    serves_desk: bool,
}

impl SecurityPolicy {
    /// Build the policy for a configuration. When the API serves the desk,
    /// `index.html` is read once so its inline scripts can be allowed by hash.
    #[must_use]
    pub fn for_config(config: Arc<ApiConfig>) -> Self {
        let hashes = config
            .serve_desk
            .as_deref()
            .map(|desk| inline_script_hashes_in(&desk.join("index.html")))
            .unwrap_or_default();
        let desk_csp = desk_policy(&hashes);
        Self {
            desk_csp: HeaderValue::from_str(&desk_csp)
                .unwrap_or_else(|_| HeaderValue::from_static(API_POLICY)),
            api_csp: HeaderValue::from_static(API_POLICY),
            hsts: config.cookie_secure.then(|| {
                HeaderValue::from_str(&format!("max-age={ONE_YEAR}")).expect("static header")
            }),
            serves_desk: config.serve_desk.is_some(),
            config,
        }
    }

    /// Stamp the headers for a response to `path`, sent by a page at `origin`.
    pub fn apply(&self, path: &str, origin: Option<&str>, response: &mut Response) {
        let is_api = path.starts_with("/api/") || path == "/health";
        let is_asset = path.starts_with("/assets/") && response.status() == StatusCode::OK;
        let headers = response.headers_mut();
        headers.insert(
            header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        );
        headers.insert(
            header::REFERRER_POLICY,
            HeaderValue::from_static("no-referrer"),
        );
        headers.insert(header::X_FRAME_OPTIONS, HeaderValue::from_static("DENY"));
        headers.insert(
            HeaderName::from_static("permissions-policy"),
            HeaderValue::from_static("camera=(), microphone=(), geolocation=(), payment=()"),
        );
        headers.insert(
            header::CONTENT_SECURITY_POLICY,
            if is_api || !self.serves_desk {
                self.api_csp.clone()
            } else {
                self.desk_csp.clone()
            },
        );
        headers.insert(
            header::CACHE_CONTROL,
            if is_api || !self.serves_desk {
                // Manuscripts, canon and tokens never land in a disk cache.
                HeaderValue::from_static("no-store")
            } else if is_asset {
                // Hashed file names: a new build has new names, so the old
                // bytes may live in the cache until the browser forgets them.
                HeaderValue::from_static("public, max-age=31536000, immutable")
            } else {
                // The shell: index.html, the service worker, the manifest and
                // icons. Cached, but revalidated on every load so a new build
                // reaches the browser at once.
                HeaderValue::from_static("no-cache")
            },
        );
        if let Some(hsts) = &self.hsts {
            headers.insert(header::STRICT_TRANSPORT_SECURITY, hsts.clone());
        }
        if origin.is_some_and(|origin| origin_allowed(&self.config, origin)) {
            headers.insert(
                HeaderName::from_static("access-control-allow-private-network"),
                HeaderValue::from_static("true"),
            );
        }
    }
}

fn desk_policy(hashes: &[String]) -> String {
    let scripts: String = hashes
        .iter()
        .map(|hash| format!(" '{hash}'"))
        .collect::<Vec<_>>()
        .join("");
    DESK_POLICY.replace("{scripts}", &scripts)
}

/// CSP source expressions for the inline scripts in an HTML file, or none
/// when the file cannot be read (then inline scripts are simply blocked).
fn inline_script_hashes_in(index: &Path) -> Vec<String> {
    std::fs::read_to_string(index)
        .map(|html| inline_script_hashes(&html))
        .unwrap_or_default()
}

/// `'sha256-…'` for every `<script>` element without a `src`. The hash covers
/// the exact bytes between the tags, as the CSP specification requires.
fn inline_script_hashes(html: &str) -> Vec<String> {
    let lower = html.to_ascii_lowercase();
    let mut hashes = Vec::new();
    let mut cursor = 0;
    while let Some(start) = lower[cursor..].find("<script") {
        let open = cursor + start;
        let Some(tag_end) = lower[open..].find('>') else {
            break;
        };
        let body_start = open + tag_end + 1;
        let Some(close) = lower[body_start..].find("</script") else {
            break;
        };
        let body_end = body_start + close;
        let tag = &lower[open..body_start];
        if !tag.contains("src=") {
            let digest = Sha256::digest(&html.as_bytes()[body_start..body_end]);
            let encoded = base64::engine::general_purpose::STANDARD.encode(digest);
            hashes.push(format!("sha256-{encoded}"));
        }
        cursor = body_end;
    }
    hashes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inline_scripts_are_hashed_and_external_ones_are_not() {
        let html = r#"<html><head>
            <script>console.log("theme")</script>
            <script type="module" crossorigin src="/assets/index-abc.js"></script>
            <SCRIPT>second()</SCRIPT>
        </head></html>"#;
        let hashes = inline_script_hashes(html);
        assert_eq!(hashes.len(), 2);
        let expected = base64::engine::general_purpose::STANDARD
            .encode(Sha256::digest(b"console.log(\"theme\")"));
        assert_eq!(hashes[0], format!("sha256-{expected}"));
        assert!(hashes[1].starts_with("sha256-"));
        assert!(inline_script_hashes("<p>no scripts</p>").is_empty());
    }

    #[test]
    fn the_desk_policy_lists_the_hashes_after_self() {
        let policy = desk_policy(&["sha256-abc".to_owned()]);
        assert!(policy.contains("script-src 'self' 'sha256-abc'; "));
        assert!(policy.contains("frame-ancestors 'none'"));
        assert!(!policy.contains("{scripts}"));
        assert!(desk_policy(&[]).contains("script-src 'self'; "));
    }
}
