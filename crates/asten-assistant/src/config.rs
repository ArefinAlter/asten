//! Settings the assistant keeps on disk: the master switch, offline mode, the
//! provider ladder with each provider's consent, the spend caps and the price
//! table. Keys are never in this file (see [`crate::keys`]); only the last four
//! characters, which the interface shows.

use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::RwLock,
};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::error::{AssistantError, Result};

/// Bump when the "what is sent" notice changes; consent given under an older
/// notice must be given again.
pub const NOTICE_VERSION: u32 = 1;

/// The wire format a provider speaks.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Protocol {
    /// Anthropic Messages API.
    Anthropic,
    /// OpenAI-style Chat Completions (OpenAI and every compatible service).
    Openai,
    /// Google Gemini `generateContent`.
    Gemini,
}

/// Whether a provider runs on this machine or is a cloud service.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    /// A model on this machine (nothing leaves it).
    Local,
    /// A cloud service, used only with the writer's own key.
    Cloud,
}

/// How tools are offered to the model.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolMode {
    /// The provider's own tool calling.
    #[default]
    Native,
    /// For models without tool calling: the tools are described in the prompt
    /// and the model answers with one JSON object.
    Json,
}

/// The writer's consent for one provider.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Consent {
    /// When it was given.
    pub accepted_at: DateTime<Utc>,
    /// The host the requests go to; a different host needs new consent.
    pub host: String,
    /// The version of the notice the writer saw.
    pub notice_version: u32,
}

/// One rung of the ladder.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ProviderConfig {
    /// Stable id (`anthropic`, `openai`, `ollama`, or `custom-…`).
    pub id: String,
    /// Display name.
    pub name: String,
    /// Local or cloud.
    pub kind: Kind,
    /// The wire format.
    pub protocol: Protocol,
    /// Which quirks table applies (`openai`, `mistral`, `deepseek`, `qwen`,
    /// `kimi`, `glm`, `ollama`, `custom`, `anthropic`, `gemini`).
    pub family: String,
    /// Model name sent in requests.
    pub model: String,
    /// Base URL; the adapter appends the fixed path.
    pub endpoint: String,
    /// Switched on by the writer (needs consent and, for cloud, a key).
    pub enabled: bool,
    /// Consent, if given.
    #[serde(default)]
    pub consent: Option<Consent>,
    /// Last four characters of the stored key, for display.
    #[serde(default)]
    pub key_last4: Option<String>,
    /// Native tool calling or the JSON protocol.
    #[serde(default)]
    pub tool_mode: ToolMode,
    /// Upper bound on tokens the model may write per call.
    #[serde(default = "default_max_output")]
    pub max_output_tokens: u32,
}

fn default_max_output() -> u32 {
    4096
}

/// Spend limits. Zero means no model call.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Caps {
    /// Tokens (in plus out) per day.
    pub daily_tokens: u64,
    /// Tokens per conversation thread.
    pub thread_tokens: u64,
    /// Estimated dollars per day (counted only where a price is known).
    pub daily_usd: f64,
}

impl Default for Caps {
    fn default() -> Self {
        Self {
            daily_tokens: 200_000,
            thread_tokens: 60_000,
            daily_usd: 1.0,
        }
    }
}

/// Price per million tokens, in dollars.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub struct Price {
    /// Input tokens.
    pub input_per_mtok: f64,
    /// Output tokens.
    pub output_per_mtok: f64,
}

/// Everything persisted.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AssistantConfig {
    /// File format version.
    #[serde(default = "one")]
    pub version: u32,
    /// Master switch. Off means no provider is ever contacted.
    #[serde(default)]
    pub enabled: bool,
    /// Offline mode: only providers on this machine may be called.
    #[serde(default)]
    pub offline: bool,
    /// Try providers on this machine before any cloud provider.
    #[serde(default = "yes")]
    pub local_first: bool,
    /// The ladder, in order.
    #[serde(default = "default_providers")]
    pub providers: Vec<ProviderConfig>,
    /// Spend caps.
    #[serde(default)]
    pub caps: Caps,
    /// Writer-edited prices, keyed `provider-id/model`.
    #[serde(default)]
    pub prices: BTreeMap<String, Price>,
}

fn one() -> u32 {
    1
}
fn yes() -> bool {
    true
}

impl Default for AssistantConfig {
    fn default() -> Self {
        Self {
            version: 1,
            enabled: false,
            offline: false,
            local_first: true,
            providers: default_providers(),
            caps: Caps::default(),
            prices: BTreeMap::new(),
        }
    }
}

fn provider(
    id: &str,
    name: &str,
    kind: Kind,
    protocol: Protocol,
    family: &str,
    model: &str,
    endpoint: &str,
) -> ProviderConfig {
    ProviderConfig {
        id: id.into(),
        name: name.into(),
        kind,
        protocol,
        family: family.into(),
        model: model.into(),
        endpoint: endpoint.into(),
        enabled: false,
        consent: None,
        key_last4: None,
        tool_mode: ToolMode::Native,
        max_output_tokens: default_max_output(),
    }
}

/// The ladder a new install starts with: local first, then the cloud services.
#[must_use]
pub fn default_providers() -> Vec<ProviderConfig> {
    use Kind::{Cloud, Local};
    use Protocol::{Anthropic, Gemini, Openai};
    vec![
        provider(
            "ollama",
            "Ollama (this machine)",
            Local,
            Openai,
            "ollama",
            "llama3.2:3b",
            "http://127.0.0.1:11434/v1",
        ),
        provider(
            "anthropic",
            "Anthropic",
            Cloud,
            Anthropic,
            "anthropic",
            "claude-haiku-5-5",
            "https://api.anthropic.com",
        ),
        provider(
            "openai",
            "OpenAI",
            Cloud,
            Openai,
            "openai",
            "gpt-6-luna",
            "https://api.openai.com/v1",
        ),
        provider(
            "google",
            "Google Gemini",
            Cloud,
            Gemini,
            "gemini",
            "gemini-3.8-flash",
            "https://generativelanguage.googleapis.com",
        ),
        provider(
            "mistral",
            "Mistral",
            Cloud,
            Openai,
            "mistral",
            "mistral-small-latest",
            "https://api.mistral.ai/v1",
        ),
        provider(
            "deepseek",
            "DeepSeek",
            Cloud,
            Openai,
            "deepseek",
            "deepseek-flash",
            "https://api.deepseek.com",
        ),
        provider(
            "qwen",
            "Qwen (Alibaba Cloud)",
            Cloud,
            Openai,
            "qwen",
            "qwen-plus",
            "https://dashscope-intl.aliyuncs.com/compatible-mode/v1",
        ),
        provider(
            "kimi",
            "Kimi (Moonshot)",
            Cloud,
            Openai,
            "kimi",
            "kimi-k2.6",
            "https://api.moonshot.ai/v1",
        ),
        provider(
            "glm",
            "GLM (Zhipu / Z.ai)",
            Cloud,
            Openai,
            "glm",
            "glm-5.3",
            "https://api.z.ai/api/paas/v4",
        ),
        provider(
            "codex-subscription",
            "Codex (existing sign-in)",
            Cloud,
            Openai,
            "subscription",
            "default",
            "https://codex.local-agent.invalid/v1",
        ),
        provider(
            "claude-subscription",
            "Claude Code (existing sign-in)",
            Cloud,
            Openai,
            "subscription",
            "default",
            "https://claude.local-agent.invalid/v1",
        ),
        provider(
            "cursor-subscription",
            "Cursor (existing sign-in)",
            Cloud,
            Openai,
            "subscription",
            "default",
            "https://cursor.local-agent.invalid/v1",
        ),
        provider(
            "antigravity-subscription",
            "Antigravity (existing sign-in)",
            Cloud,
            Openai,
            "subscription",
            "default",
            "https://antigravity.local-agent.invalid/v1",
        ),
    ]
}

/// Model names offered as suggestions in Settings (the writer may type any).
#[must_use]
pub fn suggested_models(family: &str) -> &'static [&'static str] {
    match family {
        "anthropic" => &[
            "claude-haiku-5-5",
            "claude-sonnet-5-5",
            "claude-opus-5-5",
            "claude-fable-5-1",
        ],
        "openai" => &["gpt-6-luna", "gpt-6.1-sol", "gpt-6-astra"],
        "gemini" => &[
            "gemini-3.8-flash",
            "gemini-3.5-flash-lite",
            "gemini-4-argon",
        ],
        "mistral" => &["mistral-small-latest", "mistral-large-latest"],
        "deepseek" => &["deepseek-flash", "deepseek-v4-pro"],
        "qwen" => &["qwen-plus", "qwen-max", "qwen-turbo", "qwen3.8-max"],
        "kimi" => &["kimi-k2.6", "kimi-k3", "kimi-k3.1-pro"],
        "glm" => &["glm-5.3", "glm-5.2", "glm-4.7"],
        "ollama" => &["llama3.2:3b", "qwen3:8b", "mistral:7b"],
        _ => &[],
    }
}

/// Built-in prices (dollars per million tokens) taken from the providers'
/// published pages on 26 October 2026. Unknown models have no estimate.
#[must_use]
pub fn builtin_price(family: &str, model: &str) -> Option<Price> {
    let (input, output) = match (family, model) {
        ("anthropic", "claude-fable-5-1") => (10.0, 50.0),
        ("anthropic", "claude-opus-5-5") => (4.0, 20.0),
        ("anthropic", "claude-sonnet-5-5") => (2.0, 10.0),
        ("anthropic", "claude-haiku-5-5") => (0.10, 0.50),
        ("openai", "gpt-6-astra") => (10.0, 50.0),
        ("openai", "gpt-6.1-sol") => (2.0, 10.0),
        ("openai", "gpt-6-luna") => (0.10, 0.50),
        ("ollama", _) => (0.0, 0.0),
        _ => return None,
    };
    Some(Price {
        input_per_mtok: input,
        output_per_mtok: output,
    })
}

impl AssistantConfig {
    /// Look a provider up by id.
    #[must_use]
    pub fn provider(&self, id: &str) -> Option<&ProviderConfig> {
        self.providers.iter().find(|p| p.id == id)
    }

    /// The price for a provider's current model: the writer's table first.
    #[must_use]
    pub fn price_for(&self, provider: &ProviderConfig) -> Option<Price> {
        self.prices
            .get(&format!("{}/{}", provider.id, provider.model))
            .copied()
            .or_else(|| builtin_price(&provider.family, &provider.model))
    }
}

impl ProviderConfig {
    /// Uses an installed agent's own sign-in; still remote for consent and offline mode.
    pub fn is_subscription(&self) -> bool {
        crate::provider::local_agent::agent_for_url(&self.endpoint).is_some()
    }

    /// The host requests go to.
    #[must_use]
    pub fn host(&self) -> String {
        Url::parse(&self.endpoint)
            .ok()
            .and_then(|u| u.host_str().map(str::to_owned))
            .unwrap_or_default()
    }

    /// True when the endpoint is on this machine (loopback).
    #[must_use]
    pub fn is_loopback(&self) -> bool {
        host_is_loopback(&self.host())
    }

    /// True when the writer's consent covers the current endpoint host and
    /// the current notice.
    #[must_use]
    pub fn has_valid_consent(&self) -> bool {
        self.consent
            .as_ref()
            .is_some_and(|c| c.notice_version == NOTICE_VERSION && c.host == self.host())
    }

    /// Requests go nowhere but this machine.
    #[must_use]
    pub fn stays_local(&self) -> bool {
        self.kind == Kind::Local && self.is_loopback()
    }
}

/// Whether a host name is this machine.
#[must_use]
pub fn host_is_loopback(host: &str) -> bool {
    let host = host.trim_matches(|c| c == '[' || c == ']');
    host.eq_ignore_ascii_case("localhost")
        || host
            .parse::<std::net::IpAddr>()
            .is_ok_and(|ip| ip.is_loopback())
}

/// Check an endpoint before it is saved or used: HTTPS everywhere except
/// loopback, no credentials in the URL, no query string, no fragment.
pub fn validate_endpoint(endpoint: &str) -> Result<Url> {
    let url = Url::parse(endpoint.trim())
        .map_err(|_| AssistantError::Invalid("The endpoint is not a valid URL".into()))?;
    let host = url
        .host_str()
        .ok_or_else(|| AssistantError::Invalid("The endpoint has no host".into()))?
        .to_owned();
    if !url.username().is_empty() || url.password().is_some() {
        return Err(AssistantError::Invalid(
            "The endpoint must not contain a user name or password".into(),
        ));
    }
    if url.query().is_some() || url.fragment().is_some() {
        return Err(AssistantError::Invalid(
            "The endpoint must not contain a query string or fragment; keys go in a header, not a URL"
                .into(),
        ));
    }
    match url.scheme() {
        "https" => {}
        "http" if host_is_loopback(&host) => {}
        _ => {
            return Err(AssistantError::Invalid(
                "The endpoint must use https (plain http is allowed only for this machine)".into(),
            ));
        }
    }
    Ok(url)
}

/// Strip a trailing slash and a pasted `/chat/completions` or `/v1/messages`.
#[must_use]
pub fn normalise_endpoint(endpoint: &str) -> String {
    let mut text = endpoint.trim().trim_end_matches('/').to_owned();
    for suffix in ["/chat/completions", "/v1/messages"] {
        if let Some(stripped) = text.strip_suffix(suffix) {
            text = stripped.to_owned();
        }
    }
    text
}

/// The file-backed store.
pub struct ConfigStore {
    path: PathBuf,
    inner: RwLock<AssistantConfig>,
}

impl ConfigStore {
    /// Load `path`, or start from defaults when it does not exist or cannot be
    /// read (the broken file is kept beside it as `.bad`).
    #[must_use]
    pub fn open(path: &Path) -> Self {
        let config = match std::fs::read_to_string(path) {
            Ok(text) => match serde_json::from_str::<AssistantConfig>(&text) {
                Ok(config) => config,
                Err(error) => {
                    tracing::warn!(%error, "assistant settings unreadable; starting from defaults");
                    let _ = std::fs::rename(path, path.with_extension("json.bad"));
                    AssistantConfig::default()
                }
            },
            Err(_) => AssistantConfig::default(),
        };
        Self {
            path: path.to_owned(),
            inner: RwLock::new(sanitise(config)),
        }
    }

    /// A copy of the current settings.
    #[must_use]
    pub fn snapshot(&self) -> AssistantConfig {
        self.inner
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }

    /// Change the settings and write them to disk.
    pub fn update<R>(&self, change: impl FnOnce(&mut AssistantConfig) -> Result<R>) -> Result<R> {
        let mut guard = self
            .inner
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let mut next = guard.clone();
        let out = change(&mut next)?;
        let next = sanitise(next);
        save(&self.path, &next)?;
        *guard = next;
        Ok(out)
    }
}

/// Make a loaded or edited config safe: every endpoint valid, consent dropped
/// where the host or the notice changed, and a provider that lost its consent
/// switched off.
fn sanitise(mut config: AssistantConfig) -> AssistantConfig {
    let defaults = default_providers();
    for default in &defaults {
        if !config.providers.iter().any(|p| p.id == default.id) {
            config.providers.push(default.clone());
        }
    }
    for p in &mut config.providers {
        p.endpoint = normalise_endpoint(&p.endpoint);
        if validate_endpoint(&p.endpoint).is_err() {
            if let Some(default) = defaults.iter().find(|d| d.id == p.id) {
                p.endpoint = default.endpoint.clone();
            }
            p.enabled = false;
        }
        if !p.has_valid_consent() {
            p.consent = None;
            p.enabled = false;
        }
        p.max_output_tokens = p.max_output_tokens.clamp(256, 32_768);
    }
    config
}

fn save(path: &Path, config: &AssistantConfig) -> Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let text = serde_json::to_string_pretty(config)
        .map_err(|error| AssistantError::Io(error.to_string()))?;
    let temp = path.with_extension("json.tmp");
    std::fs::write(&temp, text)?;
    std::fs::rename(&temp, path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn endpoints_are_validated() {
        assert!(validate_endpoint("https://api.openai.com/v1").is_ok());
        assert!(validate_endpoint("http://127.0.0.1:11434/v1").is_ok());
        assert!(validate_endpoint("http://localhost:11434").is_ok());
        assert!(validate_endpoint("http://example.org/v1").is_err());
        assert!(validate_endpoint("https://user:pw@example.org/v1").is_err());
        assert!(validate_endpoint("https://example.org/v1?key=abc").is_err());
        assert!(validate_endpoint("ftp://example.org").is_err());
        assert!(validate_endpoint("not a url").is_err());
    }

    #[test]
    fn pasted_paths_are_stripped() {
        assert_eq!(
            normalise_endpoint("https://api.openai.com/v1/chat/completions/"),
            "https://api.openai.com/v1"
        );
        assert_eq!(
            normalise_endpoint("https://api.anthropic.com/v1/messages"),
            "https://api.anthropic.com"
        );
    }

    #[test]
    fn consent_is_tied_to_the_host_and_the_notice() {
        let mut p = default_providers().remove(1);
        assert!(!p.has_valid_consent());
        p.consent = Some(Consent {
            accepted_at: Utc::now(),
            host: p.host(),
            notice_version: NOTICE_VERSION,
        });
        assert!(p.has_valid_consent());
        p.endpoint = "https://proxy.example.org".into();
        assert!(!p.has_valid_consent());
    }

    #[test]
    fn the_store_round_trips_and_drops_stale_consent() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("assistant.json");
        let store = ConfigStore::open(&path);
        store
            .update(|c| {
                c.enabled = true;
                let p = c
                    .providers
                    .iter_mut()
                    .find(|p| p.id == "anthropic")
                    .unwrap();
                p.consent = Some(Consent {
                    accepted_at: Utc::now(),
                    host: "api.anthropic.com".into(),
                    notice_version: NOTICE_VERSION,
                });
                p.enabled = true;
                Ok(())
            })
            .unwrap();
        let again = ConfigStore::open(&path).snapshot();
        assert!(again.enabled);
        assert!(again.provider("anthropic").unwrap().enabled);
        // Pointing the provider elsewhere removes the consent and switches it off.
        store
            .update(|c| {
                c.providers
                    .iter_mut()
                    .find(|p| p.id == "anthropic")
                    .unwrap()
                    .endpoint = "https://elsewhere.example.org".into();
                Ok(())
            })
            .unwrap();
        let after = store.snapshot();
        let p = after.provider("anthropic").unwrap();
        assert!(p.consent.is_none());
        assert!(!p.enabled);
    }

    #[test]
    fn the_defaults_start_switched_off() {
        let c = AssistantConfig::default();
        assert!(!c.enabled);
        assert!(
            c.providers
                .iter()
                .all(|p| !p.enabled && p.consent.is_none())
        );
        assert_eq!(c.providers.len(), 13);
        let subscriptions: Vec<_> = c.providers.iter().filter(|p| p.is_subscription()).collect();
        assert_eq!(subscriptions.len(), 4);
        assert!(subscriptions.iter().all(|p| p.family == "subscription"));
    }
}
