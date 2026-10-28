//! Tauri commands for the optional assistant.
//!
//! The story API stays HTTP (D6). Keys, consent, provider calls and the spend
//! ledger live here instead, in this process, because the page must never hold
//! a key and has no permission to reach a provider: the webview's connection
//! policy allows loopback only. The page types a key once into
//! `ai_set_key`; nothing ever returns it.

use std::{
    collections::{BTreeMap, HashMap},
    path::Path,
    sync::{Arc, Mutex},
};

use asten_application::Application;
use asten_assistant::{
    CancellationToken, Secret,
    config::{
        AssistantConfig, Caps, ConfigStore, Consent, Kind, NOTICE_VERSION, Price, Protocol,
        ProviderConfig, ToolMode, normalise_endpoint, suggested_models, validate_endpoint,
    },
    error::AssistantError,
    keys::{self, KeyStore, VaultState},
    ledger::{Entry, Ledger, Totals},
    policy,
    provider::transport::Transport,
    runner::{Assistant, Event, EventSink, Preview, RunRequest, RunSummary},
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tauri::{State, ipc::Channel};

/// What the commands share.
pub struct AssistantState {
    assistant: Assistant,
    runs: Mutex<HashMap<String, CancellationToken>>,
}

/// Open the assistant's files under `data` and wire it to the engine.
pub async fn open(app: Application, data: &Path) -> Result<AssistantState, AssistantError> {
    let config = Arc::new(ConfigStore::open(&data.join("assistant.json")));
    let service = std::env::var("ASTEN_KEYRING_SERVICE").unwrap_or_else(|_| "Asten".into());
    let key_file = data.join("keys.enc");
    // The credential store can block on a system bus; keep it off the runtime.
    let keys: Arc<dyn KeyStore> =
        tokio::task::spawn_blocking(move || keys::detect(&service, &key_file))
            .await
            .map_err(|error| AssistantError::Keys(error.to_string()))?;
    let ledger = Arc::new(Ledger::open(&data.join("ledger.jsonl")));
    let transport = Transport::new()?;
    Ok(AssistantState {
        assistant: Assistant::new(config, keys, ledger, transport, app),
        runs: Mutex::new(HashMap::new()),
    })
}

struct ChannelSink(Channel<Event>);

impl EventSink for ChannelSink {
    fn send(&self, event: Event) {
        let _ = self.0.send(event);
    }
}

fn message(error: AssistantError) -> String {
    error.to_string()
}

// ───────────────────────── status ─────────────────────────

/// A provider as the settings screen sees it. No key is ever in here.
#[derive(Serialize)]
pub struct ProviderView {
    id: String,
    name: String,
    kind: Kind,
    protocol: Protocol,
    family: String,
    model: String,
    endpoint: String,
    host: String,
    enabled: bool,
    consent: Option<ConsentView>,
    consent_valid: bool,
    key_stored: bool,
    key_last4: Option<String>,
    tool_mode: ToolMode,
    max_output_tokens: u32,
    suggested_models: Vec<&'static str>,
    /// Nothing leaves this machine when this provider is used.
    local: bool,
    price: Option<Price>,
    /// Why it cannot be used now; `None` when it is ready.
    not_ready: Option<String>,
    custom: bool,
    /// Executable discovery only; the provider retains sign-in and entitlement checks.
    installed: Option<bool>,
}

#[derive(Serialize)]
pub struct ConsentView {
    accepted_at: String,
    host: String,
}

/// Everything the settings screen and the assistant panel need.
#[derive(Serialize)]
pub struct Status {
    enabled: bool,
    offline: bool,
    local_first: bool,
    caps: Caps,
    providers: Vec<ProviderView>,
    vault: VaultState,
    totals: Totals,
    notice_version: u32,
    rules: &'static [&'static str],
    tools: Vec<ToolView>,
}

#[derive(Serialize)]
pub struct ToolView {
    name: &'static str,
    title: &'static str,
    description: &'static str,
    open: bool,
    closed_because: &'static str,
}

fn view(config: &AssistantConfig, p: &ProviderConfig) -> ProviderView {
    ProviderView {
        id: p.id.clone(),
        name: p.name.clone(),
        kind: p.kind,
        protocol: p.protocol,
        family: p.family.clone(),
        model: p.model.clone(),
        endpoint: p.endpoint.clone(),
        host: p.host(),
        enabled: p.enabled,
        consent: p.consent.as_ref().map(|c| ConsentView {
            accepted_at: c.accepted_at.to_rfc3339(),
            host: c.host.clone(),
        }),
        consent_valid: p.has_valid_consent(),
        key_stored: p.key_last4.is_some(),
        key_last4: p.key_last4.clone(),
        tool_mode: p.tool_mode,
        max_output_tokens: p.max_output_tokens,
        suggested_models: suggested_models(&p.family).to_vec(),
        local: p.stays_local(),
        price: config.price_for(p),
        not_ready: Assistant::not_ready_reason(config, p),
        custom: p.family == "custom",
        installed: asten_assistant::provider::local_agent::agent_for_url(&p.endpoint)
            .map(asten_assistant::provider::local_agent::installed),
    }
}

fn status_of(assistant: &Assistant, thread: Option<&str>) -> Status {
    let config = assistant.config.snapshot();
    Status {
        enabled: config.enabled,
        offline: config.offline,
        local_first: config.local_first,
        caps: config.caps.clone(),
        providers: config.providers.iter().map(|p| view(&config, p)).collect(),
        vault: assistant.keys.state(),
        totals: assistant.ledger.totals(thread),
        notice_version: NOTICE_VERSION,
        rules: policy::FIXED_RULES,
        tools: policy::TOOLS
            .iter()
            .map(|t| ToolView {
                name: t.name,
                title: t.title,
                description: t.description,
                open: matches!(t.needs, policy::Needs::Access(_)),
                closed_because: t.closed_because,
            })
            .collect(),
    }
}

/// The assistant's settings, key status (never the keys), caps and spend.
#[tauri::command]
pub fn ai_status(state: State<'_, AssistantState>, thread_id: Option<String>) -> Status {
    status_of(&state.assistant, thread_id.as_deref())
}

// ───────────────────────── settings ─────────────────────────

/// One provider's changes.
#[derive(Deserialize)]
pub struct ProviderPatch {
    id: String,
    name: Option<String>,
    model: Option<String>,
    endpoint: Option<String>,
    enabled: Option<bool>,
    tool_mode: Option<ToolMode>,
    max_output_tokens: Option<u32>,
}

/// Changes to the settings.
#[derive(Deserialize, Default)]
pub struct Patch {
    enabled: Option<bool>,
    offline: Option<bool>,
    local_first: Option<bool>,
    caps: Option<Caps>,
    /// The ladder, by provider id.
    order: Option<Vec<String>>,
    /// Prices by `provider-id/model`; `null` removes the writer's own price.
    prices: Option<BTreeMap<String, Option<Price>>>,
    providers: Option<Vec<ProviderPatch>>,
}

/// Change settings. Switching a provider on needs consent first.
#[tauri::command]
pub fn ai_update(state: State<'_, AssistantState>, patch: Patch) -> Result<Status, String> {
    let assistant = &state.assistant;
    assistant
        .config
        .update(|config| {
            if let Some(value) = patch.enabled {
                config.enabled = value;
            }
            if let Some(value) = patch.offline {
                config.offline = value;
            }
            if let Some(value) = patch.local_first {
                config.local_first = value;
            }
            if let Some(caps) = patch.caps.clone() {
                if !caps.daily_usd.is_finite() || caps.daily_usd < 0.0 {
                    return Err(AssistantError::Invalid(
                        "The daily spend cap is not a valid amount".into(),
                    ));
                }
                config.caps = caps;
            }
            if let Some(order) = &patch.order {
                let mut sorted = Vec::new();
                for id in order {
                    if let Some(p) = config.providers.iter().find(|p| &p.id == id) {
                        if !sorted.iter().any(|q: &ProviderConfig| q.id == p.id) {
                            sorted.push(p.clone());
                        }
                    }
                }
                for p in &config.providers {
                    if !sorted.iter().any(|q| q.id == p.id) {
                        sorted.push(p.clone());
                    }
                }
                config.providers = sorted;
            }
            if let Some(prices) = &patch.prices {
                for (key, price) in prices {
                    match price {
                        Some(price)
                            if price.input_per_mtok.is_finite()
                                && price.output_per_mtok.is_finite()
                                && price.input_per_mtok >= 0.0
                                && price.output_per_mtok >= 0.0 =>
                        {
                            config.prices.insert(key.clone(), *price);
                        }
                        Some(_) => {
                            return Err(AssistantError::Invalid(
                                "A price must be a number of dollars, zero or more".into(),
                            ));
                        }
                        None => {
                            config.prices.remove(key);
                        }
                    }
                }
            }
            for change in patch.providers.iter().flatten() {
                let provider = config
                    .providers
                    .iter_mut()
                    .find(|p| p.id == change.id)
                    .ok_or_else(|| {
                        AssistantError::NotReady(format!("No provider called {}", change.id))
                    })?;
                if let Some(name) = &change.name {
                    if provider.family == "custom" && !name.trim().is_empty() {
                        provider.name = name.trim().chars().take(60).collect();
                    }
                }
                if let Some(model) = &change.model {
                    if model.trim().is_empty() || model.len() > 120 {
                        return Err(AssistantError::Invalid(
                            "The model name is empty or too long".into(),
                        ));
                    }
                    provider.model = model.trim().to_owned();
                }
                if let Some(endpoint) = &change.endpoint {
                    let endpoint = normalise_endpoint(endpoint);
                    validate_endpoint(&endpoint)?;
                    provider.endpoint = endpoint;
                }
                if let Some(mode) = change.tool_mode {
                    provider.tool_mode = mode;
                }
                if let Some(tokens) = change.max_output_tokens {
                    provider.max_output_tokens = tokens;
                }
                if let Some(enabled) = change.enabled {
                    if enabled && !provider.has_valid_consent() {
                        return Err(AssistantError::NotReady(format!(
                            "Give your consent for {} first",
                            provider.name
                        )));
                    }
                    provider.enabled = enabled;
                }
            }
            Ok(())
        })
        .map_err(message)?;
    Ok(status_of(assistant, None))
}

/// Add an OpenAI-compatible service by base URL.
#[tauri::command]
pub fn ai_add_provider(
    state: State<'_, AssistantState>,
    name: String,
    endpoint: String,
    model: String,
) -> Result<Status, String> {
    let assistant = &state.assistant;
    assistant
        .config
        .update(|config| {
            let endpoint = normalise_endpoint(&endpoint);
            let url = validate_endpoint(&endpoint)?;
            if name.trim().is_empty() || model.trim().is_empty() {
                return Err(AssistantError::Invalid(
                    "A name and a model are needed".into(),
                ));
            }
            let local =
                asten_assistant::config::host_is_loopback(url.host_str().unwrap_or_default());
            let mut n = 1;
            let id = loop {
                let id = format!("custom-{n}");
                if config.provider(&id).is_none() {
                    break id;
                }
                n += 1;
            };
            config.providers.push(ProviderConfig {
                id,
                name: name.trim().chars().take(60).collect(),
                kind: if local { Kind::Local } else { Kind::Cloud },
                protocol: Protocol::Openai,
                family: "custom".into(),
                model: model.trim().to_owned(),
                endpoint,
                enabled: false,
                consent: None,
                key_last4: None,
                tool_mode: ToolMode::Native,
                max_output_tokens: 4096,
            });
            Ok(())
        })
        .map_err(message)?;
    Ok(status_of(assistant, None))
}

/// Remove an added provider and its key.
#[tauri::command]
pub async fn ai_remove_provider(
    state: State<'_, AssistantState>,
    id: String,
) -> Result<Status, String> {
    let assistant = state.assistant.clone();
    let keys = assistant.keys.clone();
    let key_id = id.clone();
    let _ = tokio::task::spawn_blocking(move || keys.delete(&key_id)).await;
    assistant
        .config
        .update(|config| {
            let at = config
                .providers
                .iter()
                .position(|p| p.id == id && p.family == "custom")
                .ok_or_else(|| {
                    AssistantError::Invalid("Only a provider you added can be removed".into())
                })?;
            config.providers.remove(at);
            Ok(())
        })
        .map_err(message)?;
    Ok(status_of(&assistant, None))
}

// ───────────────────────── keys ─────────────────────────

/// Store a key. This is the only command that receives one, and nothing returns it.
#[tauri::command]
pub async fn ai_set_key(
    state: State<'_, AssistantState>,
    provider: String,
    key: String,
) -> Result<Status, String> {
    let assistant = state.assistant.clone();
    let secret = Secret::new(key);
    if secret.is_empty() {
        return Err("Paste a key first".into());
    }
    if assistant.config.snapshot().provider(&provider).is_none() {
        return Err(format!("No provider called {provider}"));
    }
    let last4 = secret.last4();
    let keys = assistant.keys.clone();
    let id = provider.clone();
    tokio::task::spawn_blocking(move || keys.set(&id, &secret))
        .await
        .map_err(|error| error.to_string())?
        .map_err(message)?;
    assistant
        .config
        .update(|config| {
            if let Some(p) = config.providers.iter_mut().find(|p| p.id == provider) {
                p.key_last4 = Some(last4.clone());
            }
            Ok(())
        })
        .map_err(message)?;
    Ok(status_of(&assistant, None))
}

/// Forget a key.
#[tauri::command]
pub async fn ai_forget_key(
    state: State<'_, AssistantState>,
    provider: String,
) -> Result<Status, String> {
    let assistant = state.assistant.clone();
    let keys = assistant.keys.clone();
    let id = provider.clone();
    tokio::task::spawn_blocking(move || keys.delete(&id))
        .await
        .map_err(|error| error.to_string())?
        .map_err(message)?;
    assistant
        .config
        .update(|config| {
            if let Some(p) = config.providers.iter_mut().find(|p| p.id == provider) {
                p.key_last4 = None;
                p.enabled = false;
            }
            Ok(())
        })
        .map_err(message)?;
    Ok(status_of(&assistant, None))
}

/// Unlock (or create) the encrypted key file with a passphrase.
#[tauri::command]
pub async fn ai_unlock(
    state: State<'_, AssistantState>,
    passphrase: String,
) -> Result<Status, String> {
    let assistant = state.assistant.clone();
    let keys = assistant.keys.clone();
    let secret = Secret::new(passphrase);
    tokio::task::spawn_blocking(move || keys.unlock(&secret))
        .await
        .map_err(|error| error.to_string())?
        .map_err(message)?;
    Ok(status_of(&assistant, None))
}

// ───────────────────────── consent ─────────────────────────

/// Give or withdraw consent for one provider. Giving it also switches the
/// provider on; withdrawing it switches it off.
#[tauri::command]
pub fn ai_consent(
    state: State<'_, AssistantState>,
    provider: String,
    accept: bool,
) -> Result<Status, String> {
    let assistant = &state.assistant;
    assistant
        .config
        .update(|config| {
            let p = config
                .providers
                .iter_mut()
                .find(|p| p.id == provider)
                .ok_or_else(|| {
                    AssistantError::NotReady(format!("No provider called {provider}"))
                })?;
            if accept {
                p.consent = Some(Consent {
                    accepted_at: Utc::now(),
                    host: p.host(),
                    notice_version: NOTICE_VERSION,
                });
                p.enabled = true;
            } else {
                p.consent = None;
                p.enabled = false;
            }
            Ok(())
        })
        .map_err(message)?;
    Ok(status_of(assistant, None))
}

/// Switch everything off: every consent withdrawn, every provider off, every
/// key forgotten, and the master switch off.
#[tauri::command]
pub async fn ai_withdraw_all(state: State<'_, AssistantState>) -> Result<Status, String> {
    let assistant = state.assistant.clone();
    let ids: Vec<String> = assistant
        .config
        .snapshot()
        .providers
        .iter()
        .map(|p| p.id.clone())
        .collect();
    let keys = assistant.keys.clone();
    let _ = tokio::task::spawn_blocking(move || {
        for id in ids {
            let _ = keys.delete(&id);
        }
    })
    .await;
    assistant
        .config
        .update(|config| {
            config.enabled = false;
            for p in &mut config.providers {
                p.consent = None;
                p.enabled = false;
                p.key_last4 = None;
            }
            Ok(())
        })
        .map_err(message)?;
    Ok(status_of(&assistant, None))
}

// ───────────────────────── running ─────────────────────────

/// What would be sent for this request, exactly. Nothing is sent.
#[tauri::command]
pub async fn ai_preview(
    state: State<'_, AssistantState>,
    request: RunRequest,
) -> Result<Preview, String> {
    let assistant = state.assistant.clone();
    assistant.preview(&request).await.map_err(message)
}

/// Run a request. Events stream to `on_event`; the summary returns at the end.
#[tauri::command]
pub async fn ai_run(
    state: State<'_, AssistantState>,
    request: RunRequest,
    on_event: Channel<Event>,
) -> Result<RunSummary, String> {
    let assistant = state.assistant.clone();
    let cancel = CancellationToken::new();
    state
        .runs
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .insert(request.run_id.clone(), cancel.clone());
    let run_id = request.run_id.clone();
    let sink = ChannelSink(on_event);
    let result = assistant.run(request, &sink, cancel).await;
    state
        .runs
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .remove(&run_id);
    result.map_err(message)
}

/// Stop a run.
#[tauri::command]
pub fn ai_cancel(state: State<'_, AssistantState>, run_id: String) {
    if let Some(token) = state
        .runs
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .get(&run_id)
    {
        token.cancel();
    }
}

/// A one-word request to see that a provider answers (counts against the caps).
#[tauri::command]
pub async fn ai_test(state: State<'_, AssistantState>, provider: String) -> Result<String, String> {
    struct Quiet;
    impl EventSink for Quiet {
        fn send(&self, _event: Event) {}
    }
    let assistant = state.assistant.clone();
    assistant
        .test_provider(&provider, &Quiet, CancellationToken::new())
        .await
        .map_err(message)
}

/// Models installed on a local Ollama-style server.
#[tauri::command]
pub async fn ai_local_models(
    state: State<'_, AssistantState>,
    provider: String,
) -> Result<Vec<String>, String> {
    let assistant = state.assistant.clone();
    assistant.local_models(&provider).await.map_err(message)
}

// ───────────────────────── spend ─────────────────────────

/// The ledger: totals and the most recent calls.
#[derive(Serialize)]
pub struct LedgerView {
    totals: Totals,
    recent: Vec<Entry>,
}

/// Spend so far.
#[tauri::command]
pub fn ai_ledger(state: State<'_, AssistantState>, thread_id: Option<String>) -> LedgerView {
    LedgerView {
        totals: state.assistant.ledger.totals(thread_id.as_deref()),
        recent: state.assistant.ledger.recent(50),
    }
}

/// Forget the spend history.
#[tauri::command]
pub fn ai_clear_ledger(state: State<'_, AssistantState>) -> Result<(), String> {
    state.assistant.ledger.clear().map_err(message)
}
