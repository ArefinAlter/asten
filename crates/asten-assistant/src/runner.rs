//! One assistant run: the loop that asks the model, runs the tools it calls
//! through the gate, and reports everything it does as events.
//!
//! Order of play for a request:
//!
//! 1. A request for prose is refused here, before any provider is contacted.
//! 2. The provider is chosen (the one named, or the first ready rung of the
//!    ladder) and checked: master switch, offline mode, switched on, consent
//!    for the current host, a key where one is needed, the spend caps.
//! 3. The request is built, its size and digest reported, and sent. The reply
//!    streams to the window as it arrives.
//! 4. Tool calls go through the closed table; refusals are shown and told to
//!    the model. Results go back; the loop runs at most [`MAX_STEPS`] times.
//! 5. The final text passes the output guard before it is kept.

use std::{collections::BTreeSet, sync::Arc, time::Instant};

use asten_application::Application;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use tokio_util::sync::CancellationToken;

use crate::{
    config::{AssistantConfig, ConfigStore, Kind, ProviderConfig, ToolMode},
    error::{AssistantError, Result, truncate},
    keys::KeyStore,
    ledger::{Entry, Ledger, Totals, cost_of, estimate_tokens},
    policy::{self, Access},
    provider::{
        self, AssistantMsg, Completion, Finish, ModelRequest, Msg, Piece, Quirks, ToolCall,
        ToolResultMsg, Usage, Wire, transport::Transport,
    },
    secret::Secret,
    tools::{ToolHost, definitions, parse_project},
};

/// Tool rounds a single request may use.
pub const MAX_STEPS: usize = 6;
/// Longest message accepted from the window.
const MAX_TEXT: usize = 4_000;
/// Longest passage the writer can attach.
pub const MAX_PASSAGE: usize = 8_000;
/// How much earlier conversation is sent.
const MAX_HISTORY_ITEMS: usize = 8;
const MAX_HISTORY_CHARS: usize = 4_000;

/// A skill the writer enabled for this request.
#[derive(Clone, Debug, Default, Deserialize)]
pub struct SkillRef {
    /// Its id.
    pub id: String,
    /// Its name.
    pub name: String,
    /// The tools it may use; it can only narrow, never open.
    pub tools: Vec<String>,
    /// Its instructions, as the writer edited them.
    pub body: String,
}

/// One earlier message in the thread.
#[derive(Clone, Debug, Deserialize)]
pub struct HistoryItem {
    /// `user` or `assistant`.
    pub role: String,
    /// What was said.
    pub text: String,
}

/// Something the writer attached to the question.
#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Attachment {
    /// A mark; its quoted wording may be shared.
    Mark {
        /// Mark id.
        id: String,
        /// Label shown (the code).
        label: String,
    },
    /// A scene (its record, never its text).
    Scene {
        /// Document id.
        id: String,
        /// Title.
        label: String,
    },
    /// A research or source note.
    Note {
        /// Note id.
        id: String,
        /// Title.
        label: String,
    },
    /// A passage the writer picked: the only way manuscript wording is sent.
    Selection {
        /// Document it came from.
        document_id: Option<String>,
        /// Label shown.
        label: String,
        /// The passage.
        text: String,
    },
}

/// A request from the window.
#[derive(Clone, Debug, Deserialize)]
pub struct RunRequest {
    /// Unique per run; used to cancel.
    pub run_id: String,
    /// The conversation thread (the ledger counts per thread).
    pub thread_id: String,
    /// The provider to use; `None` takes the first ready rung.
    #[serde(default)]
    pub provider: Option<String>,
    /// The open project.
    pub project_id: String,
    /// The thread's access.
    pub access: Access,
    /// Tools the writer switched off.
    #[serde(default)]
    pub disabled_tools: Vec<String>,
    /// A skill, if one was invoked.
    #[serde(default)]
    pub skill: Option<SkillRef>,
    /// The writer's message.
    pub text: String,
    /// Earlier messages in the thread.
    #[serde(default)]
    pub history: Vec<HistoryItem>,
    /// Attachments.
    #[serde(default)]
    pub attachments: Vec<Attachment>,
    /// Share the quoted wording of every mark (off by default).
    #[serde(default)]
    pub share_quotes: bool,
}

/// Everything the window is told during a run.
#[derive(Clone, Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Event {
    /// The run began with this provider.
    Started {
        /// Run id.
        run_id: String,
        /// Provider id.
        provider: String,
        /// Provider name.
        name: String,
        /// Model.
        model: String,
        /// Host the requests go to.
        host: String,
        /// Nothing leaves this machine.
        local: bool,
    },
    /// A piece of the reply.
    Text {
        /// The text.
        delta: String,
    },
    /// The model called a tool (or tried to).
    ToolCall {
        /// Call id.
        call_id: String,
        /// Tool name.
        tool: String,
        /// Its input.
        input: Value,
        /// The gate let it run.
        allowed: bool,
        /// Why not.
        reason: Option<String>,
    },
    /// A tool finished.
    ToolResult {
        /// Call id.
        call_id: String,
        /// Tool name.
        tool: String,
        /// A short line.
        summary: String,
        /// Milliseconds.
        ms: u64,
        /// It failed.
        is_error: bool,
    },
    /// A card for the writer to press.
    Card {
        /// Descriptor the window resolves.
        card: Value,
    },
    /// A request left for the provider.
    Request {
        /// Number within the run.
        n: usize,
        /// Provider id.
        provider: String,
        /// Host.
        host: String,
        /// Body bytes.
        bytes: usize,
        /// SHA-256 of the body.
        sha256: String,
        /// Rough token estimate.
        tokens_estimate: u64,
        /// The exact body, pretty-printed (long bodies are cut).
        body: String,
    },
    /// Tokens used by one call, with the running totals.
    Usage {
        /// Tokens sent.
        input: u64,
        /// Tokens received.
        output: u64,
        /// Estimated dollars.
        cost_usd: Option<f64>,
        /// The provider sent no counts; these are estimated.
        estimated: bool,
        /// Totals for today and this thread.
        totals: Totals,
    },
    /// Something the writer should know (a fallback, a limit).
    Notice {
        /// The message.
        message: String,
    },
    /// The request asked for prose and was refused before any provider was contacted.
    Refused {
        /// What matched.
        matched: String,
        /// The closed tool it would have needed.
        tool: String,
        /// The reply to show.
        reply: String,
    },
    /// The final text of the run.
    Final {
        /// The text; replaces what streamed.
        text: String,
        /// It was replaced by the refusal because it read like new prose.
        withheld: bool,
    },
    /// The run failed.
    Error {
        /// What happened.
        message: String,
    },
    /// The run ended.
    Done {
        /// Run id.
        run_id: String,
        /// Provider calls made.
        requests: usize,
        /// Tool calls made.
        tool_calls: usize,
    },
}

/// Where events go.
pub trait EventSink: Send + Sync {
    /// Deliver one event.
    fn send(&self, event: Event);
}

/// The result of a run, for the command's return value.
#[derive(Clone, Debug, Serialize)]
pub struct RunSummary {
    /// Final text.
    pub text: String,
    /// Provider calls.
    pub requests: usize,
    /// Tool calls (allowed and refused).
    pub tool_calls: usize,
    /// Refused before any provider call.
    pub refused: bool,
    /// Withheld by the output guard.
    pub withheld: bool,
}

/// The assistant: settings, keys, ledger, network, engine.
#[derive(Clone)]
pub struct Assistant {
    /// Settings.
    pub config: Arc<ConfigStore>,
    /// Keys.
    pub keys: Arc<dyn KeyStore>,
    /// Spend.
    pub ledger: Arc<Ledger>,
    /// Network.
    pub transport: Transport,
    /// The local engine.
    pub app: Application,
}

struct Chosen {
    provider: ProviderConfig,
    key: Option<Secret>,
}

impl Assistant {
    /// Put the pieces together.
    #[must_use]
    pub fn new(
        config: Arc<ConfigStore>,
        keys: Arc<dyn KeyStore>,
        ledger: Arc<Ledger>,
        transport: Transport,
        app: Application,
    ) -> Self {
        Self {
            config,
            keys,
            ledger,
            transport,
            app,
        }
    }

    async fn key_of(&self, provider: &ProviderConfig) -> Result<Option<Secret>> {
        if provider.kind == Kind::Local && provider.key_last4.is_none() {
            return Ok(None);
        }
        let keys = self.keys.clone();
        let id = provider.id.clone();
        tokio::task::spawn_blocking(move || keys.get(&id))
            .await
            .map_err(|error| AssistantError::Keys(error.to_string()))?
    }

    /// Whether a provider may be used now, and why not.
    fn check_ready(config: &AssistantConfig, p: &ProviderConfig, has_key: bool) -> Result<()> {
        if !config.enabled {
            return Err(AssistantError::Disabled);
        }
        if config.offline && !p.stays_local() {
            return Err(AssistantError::Offline);
        }
        if !p.enabled {
            return Err(AssistantError::NotReady(format!(
                "{} is switched off in Settings → AI providers",
                p.name
            )));
        }
        if !p.has_valid_consent() {
            return Err(AssistantError::NotReady(format!(
                "You have not consented to {} yet (Settings → AI providers)",
                p.name
            )));
        }
        if p.kind == Kind::Cloud && !has_key {
            return Err(AssistantError::NotReady(format!(
                "No key is stored for {}",
                p.name
            )));
        }
        Ok(())
    }

    /// The providers a request may use, in order.
    async fn candidates(&self, wanted: Option<&str>) -> Result<Vec<Chosen>> {
        let config = self.config.snapshot();
        if !config.enabled {
            return Err(AssistantError::Disabled);
        }
        if let Some(id) = wanted.filter(|id| !id.is_empty() && *id != "auto") {
            let provider = config
                .provider(id)
                .ok_or_else(|| AssistantError::NotReady(format!("No provider called {id}")))?
                .clone();
            let key = self.key_of(&provider).await?;
            Self::check_ready(&config, &provider, key.is_some())?;
            return Ok(vec![Chosen { provider, key }]);
        }
        let mut ladder: Vec<&ProviderConfig> = config.providers.iter().collect();
        if config.local_first {
            ladder.sort_by_key(|p| u8::from(p.kind != Kind::Local));
        }
        let mut ready = Vec::new();
        let mut first_problem: Option<AssistantError> = None;
        for provider in ladder {
            let key = match self.key_of(provider).await {
                Ok(key) => key,
                Err(error) => {
                    first_problem.get_or_insert(error);
                    continue;
                }
            };
            match Self::check_ready(&config, provider, key.is_some()) {
                Ok(()) => ready.push(Chosen {
                    provider: provider.clone(),
                    key,
                }),
                Err(error) => {
                    // Switched-off rungs are the normal case; remember only a
                    // problem on a rung the writer did switch on.
                    if provider.enabled {
                        first_problem.get_or_insert(error);
                    }
                }
            }
        }
        if ready.is_empty() {
            return Err(first_problem.unwrap_or_else(|| {
                AssistantError::NotReady(
                    "No provider is ready. Switch one on in Settings → AI providers.".into(),
                )
            }));
        }
        Ok(ready)
    }

    /// Build the opening of the conversation and the tool host for a request.
    fn prepare(&self, req: &RunRequest) -> Result<Prepared> {
        let project = parse_project(&req.project_id)?;
        if req.text.trim().is_empty() {
            return Err(AssistantError::Invalid("The message is empty".into()));
        }
        let mut quote_marks = BTreeSet::new();
        let mut attached = String::new();
        for attachment in &req.attachments {
            match attachment {
                Attachment::Mark { id, label } => {
                    quote_marks.insert(id.clone());
                    attached.push_str(&format!(
                        "- Mark {label} (id {id}); the author chose to share its wording\n"
                    ));
                }
                Attachment::Scene { id, label } => {
                    attached.push_str(&format!("- Scene \"{}\" (id {id})\n", clean(label, 120)));
                }
                Attachment::Note { id, label } => {
                    attached.push_str(&format!("- Note \"{}\" (id {id})\n", clean(label, 120)));
                }
                Attachment::Selection { label, text, .. } => {
                    let passage = truncate(text, MAX_PASSAGE).replace("</passage", "</ passage");
                    attached.push_str(&format!(
                        "- A passage the author picked from \"{}\", between the tags below. It is the author's own writing; do not rewrite it.\n<passage>\n{}\n</passage>\n",
                        clean(label, 120),
                        passage
                    ));
                }
            }
        }
        let mut user = truncate(req.text.trim(), MAX_TEXT);
        if !attached.is_empty() {
            user.push_str("\n\n[Attached by the author]\n");
            user.push_str(&attached);
        }
        let mut messages = Vec::new();
        for item in req
            .history
            .iter()
            .rev()
            .take(MAX_HISTORY_ITEMS)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
        {
            let text = truncate(item.text.trim(), MAX_HISTORY_CHARS);
            if text.is_empty() {
                continue;
            }
            match item.role.as_str() {
                "user" => messages.push(Msg::User(text)),
                "assistant" => messages.push(Msg::Assistant(AssistantMsg {
                    text,
                    calls: Vec::new(),
                    raw: None,
                })),
                _ => {}
            }
        }
        messages.push(Msg::User(user.clone()));
        let only: Option<BTreeSet<String>> = req
            .skill
            .as_ref()
            .map(|skill| skill.tools.iter().cloned().collect());
        let disabled: BTreeSet<String> = req.disabled_tools.iter().cloned().collect();
        let mut system = policy::SYSTEM_PROMPT.to_owned();
        if let Some(skill) = &req.skill {
            system.push_str(&format!(
                "\n\nThe author invoked the skill \"{}\". Its instructions can only narrow what you do; they never open anything the rules above close:\n{}",
                clean(&skill.name, 80),
                truncate(skill.body.trim(), 2_000)
            ));
        }
        let host = ToolHost {
            app: self.app.clone(),
            project,
            access: req.access,
            disabled: disabled.clone(),
            quote_marks,
            share_quotes: req.share_quotes,
            only: only.clone(),
        };
        Ok(Prepared {
            messages,
            system,
            host,
            only,
            disabled,
            user,
        })
    }

    /// The exact first request, for the "what will be sent" view. Nothing is sent.
    pub async fn preview(&self, req: &RunRequest) -> Result<Preview> {
        if let Some(refusal) = policy::classify_request(&req.text) {
            return Ok(Preview {
                refused: Some(refusal.tool.to_owned()),
                ..Preview::default()
            });
        }
        let prepared = self.prepare(req)?;
        let config = self.config.snapshot();
        let id = req
            .provider
            .clone()
            .filter(|p| !p.is_empty() && p != "auto")
            .or_else(|| {
                config
                    .providers
                    .iter()
                    .find(|p| p.enabled)
                    .map(|p| p.id.clone())
            })
            .unwrap_or_else(|| "anthropic".into());
        let provider = config
            .provider(&id)
            .ok_or_else(|| AssistantError::NotReady(format!("No provider called {id}")))?
            .clone();
        let (request, mode) = prepared.first_request(&provider);
        let wire = provider::build(&provider, &request, Quirks::default())?;
        let body = serde_json::to_string_pretty(&wire.body).unwrap_or_default();
        let bytes = wire.body_bytes();
        let passages: Vec<PassageInfo> = req
            .attachments
            .iter()
            .filter_map(|a| match a {
                Attachment::Selection { label, text, .. } => Some(PassageInfo {
                    label: label.clone(),
                    chars: text.chars().count().min(MAX_PASSAGE),
                }),
                _ => None,
            })
            .collect();
        Ok(Preview {
            refused: None,
            provider_id: provider.id.clone(),
            provider_name: provider.name.clone(),
            model: provider.model.clone(),
            host: provider.host(),
            local: provider.stays_local(),
            url: wire.url.to_string(),
            headers: header_view(&wire),
            sha256: hex::encode(Sha256::digest(&bytes)),
            bytes: bytes.len(),
            tokens_estimate: estimate_tokens(bytes.len()),
            contains_manuscript_text: !passages.is_empty(),
            passages,
            tools_offered: request.tools.iter().map(|t| t.name.clone()).collect(),
            json_protocol: mode == ToolMode::Json,
            body,
            notes: vec![
                "This is the first request. If the model asks for a tool, a second request carries that tool's result (marks, canon facts, or a research note you asked about). Each request is listed in the thread's log with its size and digest, and can be opened to read exactly what was sent.".into(),
                "Manuscript wording is not in this request unless you picked a passage. Quoted wording inside marks is left out unless you attach that mark.".into(),
                "Your key is added to the request header by Asten at the moment of sending. It is never shown here, in a log, or to the page.".into(),
            ],
        })
    }

    /// Run a request to the end.
    pub async fn run(
        &self,
        req: RunRequest,
        sink: &dyn EventSink,
        cancel: CancellationToken,
    ) -> Result<RunSummary> {
        let outcome = self.run_inner(&req, sink, &cancel).await;
        match outcome {
            Ok(summary) => {
                sink.send(Event::Done {
                    run_id: req.run_id.clone(),
                    requests: summary.requests,
                    tool_calls: summary.tool_calls,
                });
                Ok(summary)
            }
            Err(error) => {
                if !matches!(error, AssistantError::Cancelled) {
                    sink.send(Event::Error {
                        message: error.to_string(),
                    });
                }
                sink.send(Event::Done {
                    run_id: req.run_id.clone(),
                    requests: 0,
                    tool_calls: 0,
                });
                Err(error)
            }
        }
    }

    async fn run_inner(
        &self,
        req: &RunRequest,
        sink: &dyn EventSink,
        cancel: &CancellationToken,
    ) -> Result<RunSummary> {
        // 1. Prose is refused here, before anything else.
        if let Some(refusal) = policy::classify_request(&req.text) {
            sink.send(Event::Refused {
                matched: refusal.matched.clone(),
                tool: refusal.tool.to_owned(),
                reply: policy::REFUSAL_REPLY.to_owned(),
            });
            sink.send(Event::Final {
                text: policy::REFUSAL_REPLY.to_owned(),
                withheld: false,
            });
            return Ok(RunSummary {
                text: policy::REFUSAL_REPLY.to_owned(),
                requests: 0,
                tool_calls: 0,
                refused: true,
                withheld: false,
            });
        }
        let prepared = self.prepare(req)?;
        let candidates = self.candidates(req.provider.as_deref()).await?;
        let auto = req
            .provider
            .as_deref()
            .is_none_or(|p| p.is_empty() || p == "auto");

        let mut last_error: Option<AssistantError> = None;
        for (index, chosen) in candidates.iter().enumerate() {
            if index > 0 {
                if let Some(error) = &last_error {
                    sink.send(Event::Notice {
                        message: format!(
                            "{}; trying {} next, which you also switched on.",
                            error, chosen.provider.name
                        ),
                    });
                }
            }
            match self.run_with(req, &prepared, chosen, sink, cancel).await {
                Ok(summary) => return Ok(summary),
                Err(error) => {
                    let fall_through = auto
                        && matches!(
                            error,
                            AssistantError::Network(_)
                                | AssistantError::Provider {
                                    status: 429 | 500..=599,
                                    ..
                                }
                        );
                    if !fall_through || index + 1 == candidates.len() {
                        return Err(error);
                    }
                    last_error = Some(error);
                }
            }
        }
        Err(last_error.unwrap_or_else(|| AssistantError::NotReady("No provider is ready".into())))
    }

    async fn run_with(
        &self,
        req: &RunRequest,
        prepared: &Prepared,
        chosen: &Chosen,
        sink: &dyn EventSink,
        cancel: &CancellationToken,
    ) -> Result<RunSummary> {
        let provider = &chosen.provider;
        let config = self.config.snapshot();
        sink.send(Event::Started {
            run_id: req.run_id.clone(),
            provider: provider.id.clone(),
            name: provider.name.clone(),
            model: provider.model.clone(),
            host: provider.host(),
            local: provider.stays_local(),
        });
        let (mut request, mut mode) = prepared.first_request(provider);
        let mut quirks = Quirks::default();
        let mut grounding = prepared.user.clone();
        let mut requests = 0usize;
        let mut tool_calls = 0usize;
        let price = config.price_for(provider);
        let mut final_text: Option<String> = None;
        let mut repeats = 0usize;

        for _step in 0..=MAX_STEPS {
            if cancel.is_cancelled() {
                return Err(AssistantError::Cancelled);
            }
            let wire = provider::build(provider, &request, quirks)?;
            let bytes = wire.body_bytes();
            let planned_in = estimate_tokens(bytes.len());
            let planned_out = 512;
            let planned_cost = cost_of(price, planned_in, planned_out);
            // Re-read the caps each time: the writer may change them mid-run.
            let caps = self.config.snapshot().caps;
            self.ledger.check(
                &caps,
                &req.thread_id,
                planned_in + planned_out,
                planned_cost,
            )?;
            // Offline mode and the master switch are checked again for every
            // request, not just the first.
            let live = self.config.snapshot();
            let live_provider = live
                .provider(&provider.id)
                .ok_or_else(|| AssistantError::NotReady("That provider was removed".into()))?;
            Self::check_ready(
                &live,
                live_provider,
                chosen.key.is_some() || provider.kind == Kind::Local,
            )?;

            requests += 1;
            sink.send(Event::Request {
                n: requests,
                provider: provider.id.clone(),
                host: provider.host(),
                bytes: bytes.len(),
                sha256: hex::encode(Sha256::digest(&bytes)),
                tokens_estimate: planned_in,
                body: truncate(
                    &serde_json::to_string_pretty(&wire.body).unwrap_or_default(),
                    24_000,
                ),
            });

            let buffered = mode == ToolMode::Json;
            let mut streamed = String::new();
            let started = Instant::now();
            let sent = {
                let mut on_piece = |piece: Piece| {
                    let Piece::Text(delta) = piece;
                    streamed.push_str(&delta);
                    if !buffered {
                        sink.send(Event::Text { delta });
                    }
                };
                self.transport
                    .stream(
                        &provider.name,
                        &wire,
                        chosen.key.as_ref(),
                        cancel,
                        provider::decoder(provider),
                        &mut on_piece,
                    )
                    .await
            };
            let completion = match sent {
                Ok(completion) => completion,
                Err(AssistantError::Provider {
                    status: 400,
                    ref message,
                    ..
                }) if repeats < 2
                    && quirks.stream_usage
                    && message.to_lowercase().contains("stream_options") =>
                {
                    quirks.stream_usage = false;
                    repeats += 1;
                    requests -= 1;
                    sink.send(Event::Notice {
                        message: format!(
                            "{} does not accept stream_options; asking again without it.",
                            provider.name
                        ),
                    });
                    continue;
                }
                Err(AssistantError::Provider {
                    status: 400,
                    ref message,
                    ..
                }) if repeats < 2
                    && mode == ToolMode::Native
                    && !request.tools.is_empty()
                    && mentions_tools_unsupported(message) =>
                {
                    mode = ToolMode::Json;
                    repeats += 1;
                    requests -= 1;
                    request = prepared.json_request(&request, provider);
                    sink.send(Event::Notice {
                        message: format!(
                            "{} says this model has no tool calling; using the JSON protocol instead.",
                            provider.name
                        ),
                    });
                    continue;
                }
                Err(error) => {
                    // Whatever streamed before the break was paid for.
                    if !streamed.is_empty() {
                        self.record(req, provider, price, None, planned_in, &streamed);
                    }
                    return Err(error);
                }
            };
            let usage = self.record(
                req,
                provider,
                price,
                completion.usage,
                planned_in,
                &completion.text,
            );
            sink.send(Event::Usage {
                input: usage.input,
                output: usage.output,
                cost_usd: cost_of(price, usage.input, usage.output),
                estimated: completion.usage.is_none(),
                totals: self.ledger.totals(Some(&req.thread_id)),
            });
            tracing::debug!(
                provider = %provider.id,
                ms = started.elapsed().as_millis() as u64,
                "provider call finished"
            );

            let (text, calls) = if mode == ToolMode::Json {
                json_turn(&completion)
            } else {
                (completion.text.clone(), completion.calls.clone())
            };

            if calls.is_empty() {
                let mut text = text;
                if completion.finish == Finish::Length {
                    text.push_str("\n\n(The reply was cut off at the output limit.)");
                }
                if completion.finish == Finish::Refusal && text.trim().is_empty() {
                    text = "The provider declined to answer that.".into();
                }
                final_text = Some(text);
                break;
            }

            // Run the tools.
            let mut results = Vec::new();
            for call in &calls {
                tool_calls += 1;
                let decision = prepared.host.decide(&call.name);
                sink.send(Event::ToolCall {
                    call_id: call.id.clone(),
                    tool: call.name.clone(),
                    input: call.input.clone(),
                    allowed: decision.allowed,
                    reason: decision.reason.clone(),
                });
                let outcome = if !decision.allowed {
                    // The gate runs again inside the host; the refusal is the same.
                    prepared.host.call(&call.name, &call.input).await
                } else if let Some(error) = &call.error {
                    crate::tools::ToolOutcome {
                        allowed: true,
                        reason: None,
                        text: Some(crate::tools::wrap(&call.name, error)),
                        summary: error.clone(),
                        card: None,
                        is_error: true,
                        ms: 0,
                    }
                } else {
                    prepared.host.call(&call.name, &call.input).await
                };
                let content = match (&outcome.text, &outcome.reason) {
                    (Some(text), _) => text.clone(),
                    (None, Some(reason)) => crate::tools::wrap(
                        &call.name,
                        &format!("Refused: {reason} Say so to the author and carry on without it."),
                    ),
                    _ => String::new(),
                };
                grounding.push('\n');
                grounding.push_str(&content);
                sink.send(Event::ToolResult {
                    call_id: call.id.clone(),
                    tool: call.name.clone(),
                    summary: outcome
                        .reason
                        .clone()
                        .map_or_else(|| outcome.summary.clone(), |r| format!("refused: {r}")),
                    ms: outcome.ms,
                    is_error: outcome.is_error || !outcome.allowed,
                });
                if let Some(card) = outcome.card {
                    sink.send(Event::Card { card });
                }
                results.push(ToolResultMsg {
                    call_id: call.id.clone(),
                    name: call.name.clone(),
                    content,
                    is_error: outcome.is_error || !outcome.allowed,
                });
            }
            if mode == ToolMode::Json {
                // The JSON protocol has no tool messages: the model's JSON is
                // its turn and the results come back as the next user message.
                request.messages.push(Msg::Assistant(AssistantMsg {
                    text: completion.text.clone(),
                    calls: Vec::new(),
                    raw: None,
                }));
                let joined = results
                    .iter()
                    .map(|r| r.content.clone())
                    .collect::<Vec<_>>()
                    .join("\n");
                request.messages.push(Msg::User(format!(
                    "{joined}\nContinue: call another tool or reply with {{\"answer\": \"...\"}}."
                )));
            } else {
                request.messages.push(Msg::Assistant(AssistantMsg {
                    text: completion.text.clone(),
                    calls: calls.clone(),
                    raw: completion.raw.clone(),
                }));
                request.messages.push(Msg::ToolResults(results));
            }
        }

        let (text, withheld) = match final_text {
            Some(text) => {
                let verdict = policy::output_guard(&text, &grounding);
                if verdict.withhold {
                    tracing::info!(why = %verdict.why, "a reply was withheld by the output guard");
                    (policy::WITHHELD_REPLY.to_owned(), true)
                } else if text.trim().is_empty() {
                    ("The model sent no text.".to_owned(), false)
                } else {
                    (text, false)
                }
            }
            None => (
                format!(
                    "I stopped after {MAX_STEPS} rounds of tool calls without an answer. Ask a narrower question."
                ),
                false,
            ),
        };
        sink.send(Event::Final {
            text: text.clone(),
            withheld,
        });
        Ok(RunSummary {
            text,
            requests,
            tool_calls,
            refused: false,
            withheld,
        })
    }

    /// Record a call in the ledger. Counts are the provider's when it sent
    /// them, else estimated from sizes.
    fn record(
        &self,
        req: &RunRequest,
        provider: &ProviderConfig,
        price: Option<crate::config::Price>,
        usage: Option<Usage>,
        planned_in: u64,
        output_text: &str,
    ) -> Usage {
        let estimated = usage.is_none();
        let usage = usage.unwrap_or(Usage {
            input: planned_in,
            output: estimate_tokens(output_text.len()),
        });
        let entry = Entry {
            at: Utc::now(),
            run_id: req.run_id.clone(),
            thread_id: req.thread_id.clone(),
            provider: provider.id.clone(),
            model: provider.model.clone(),
            input_tokens: usage.input,
            output_tokens: usage.output,
            cost_usd: cost_of(price, usage.input, usage.output),
            estimated,
        };
        if let Err(error) = self.ledger.record(entry) {
            tracing::warn!(%error, "could not write the spend ledger");
        }
        usage
    }

    /// A tiny request to see that a provider answers. Counts against the caps.
    pub async fn test_provider(
        &self,
        provider_id: &str,
        sink: &dyn EventSink,
        cancel: CancellationToken,
    ) -> Result<String> {
        let candidates = self.candidates(Some(provider_id)).await?;
        let chosen = &candidates[0];
        let request = ModelRequest {
            system: "You are a connection test. Reply with the single word: ok".into(),
            messages: vec![Msg::User("Reply with the single word: ok".into())],
            tools: Vec::new(),
        };
        let wire = provider::build(&chosen.provider, &request, Quirks::default())?;
        let bytes = wire.body_bytes();
        let config = self.config.snapshot();
        let planned_in = estimate_tokens(bytes.len());
        self.ledger
            .check(&config.caps, "settings-test", planned_in + 64, None)?;
        let price = config.price_for(&chosen.provider);
        let mut collected = String::new();
        let completion: Completion = self
            .transport
            .stream(
                &chosen.provider.name,
                &wire,
                chosen.key.as_ref(),
                &cancel,
                provider::decoder(&chosen.provider),
                &mut |Piece::Text(delta)| collected.push_str(&delta),
            )
            .await?;
        let fake = RunRequest {
            run_id: format!("test-{}", Utc::now().timestamp_millis()),
            thread_id: "settings-test".into(),
            provider: Some(provider_id.to_owned()),
            project_id: String::new(),
            access: Access::Read,
            disabled_tools: Vec::new(),
            skill: None,
            text: String::new(),
            history: Vec::new(),
            attachments: Vec::new(),
            share_quotes: false,
        };
        let usage = self.record(
            &fake,
            &chosen.provider,
            price,
            completion.usage,
            planned_in,
            &completion.text,
        );
        sink.send(Event::Usage {
            input: usage.input,
            output: usage.output,
            cost_usd: cost_of(price, usage.input, usage.output),
            estimated: completion.usage.is_none(),
            totals: self.ledger.totals(Some("settings-test")),
        });
        Ok(truncate(completion.text.trim(), 120))
    }
}

fn mentions_tools_unsupported(message: &str) -> bool {
    let m = message.to_lowercase();
    (m.contains("tool") || m.contains("function"))
        && (m.contains("not support")
            || m.contains("does not support")
            || m.contains("unsupported")
            || m.contains("not available"))
}

fn clean(text: &str, max: usize) -> String {
    truncate(&text.replace(['\n', '\r', '"'], " "), max)
}

fn header_view(wire: &Wire) -> Vec<(String, String)> {
    let mut headers: Vec<(String, String)> = wire.headers.clone();
    match wire.auth {
        provider::Auth::None => {}
        provider::Auth::Bearer => headers.push((
            "authorization".into(),
            "Bearer <your key, added when sending; never shown>".into(),
        )),
        provider::Auth::Header(name) => headers.push((
            name.into(),
            "<your key, added when sending; never shown>".into(),
        )),
    }
    headers
}

/// A passage the writer picked, as the preview reports it.
#[derive(Clone, Debug, Serialize)]
pub struct PassageInfo {
    /// Label.
    pub label: String,
    /// Characters sent.
    pub chars: usize,
}

/// The "what will be sent" view.
#[derive(Clone, Debug, Default, Serialize)]
pub struct Preview {
    /// Set when the request asks for prose: nothing would be sent.
    pub refused: Option<String>,
    /// Provider id.
    pub provider_id: String,
    /// Provider name.
    pub provider_name: String,
    /// Model.
    pub model: String,
    /// Host the request goes to.
    pub host: String,
    /// Nothing leaves this machine.
    pub local: bool,
    /// The full URL.
    pub url: String,
    /// Headers, with the key shown as a placeholder.
    pub headers: Vec<(String, String)>,
    /// The exact JSON body.
    pub body: String,
    /// Body bytes.
    pub bytes: usize,
    /// SHA-256 of the body bytes.
    pub sha256: String,
    /// Rough tokens.
    pub tokens_estimate: u64,
    /// A picked passage is in the body.
    pub contains_manuscript_text: bool,
    /// The passages.
    pub passages: Vec<PassageInfo>,
    /// Tools the model is offered.
    pub tools_offered: Vec<String>,
    /// The JSON protocol replaces native tool calling.
    pub json_protocol: bool,
    /// What to know.
    pub notes: Vec<String>,
}

struct Prepared {
    messages: Vec<Msg>,
    system: String,
    host: ToolHost,
    only: Option<BTreeSet<String>>,
    disabled: BTreeSet<String>,
    user: String,
}

impl Prepared {
    fn first_request(&self, provider: &ProviderConfig) -> (ModelRequest, ToolMode) {
        let tools = definitions(self.host.access, &self.disabled, self.only.as_ref());
        let request = ModelRequest {
            system: self.system.clone(),
            messages: self.messages.clone(),
            tools,
        };
        if provider.tool_mode == ToolMode::Json {
            (self.json_request(&request, provider), ToolMode::Json)
        } else {
            (request, ToolMode::Native)
        }
    }

    /// The same conversation for a model without tool calling.
    fn json_request(&self, request: &ModelRequest, _provider: &ProviderConfig) -> ModelRequest {
        let mut system = request.system.clone();
        system.push_str("\n\n");
        system.push_str(policy::JSON_PROTOCOL_PROMPT);
        system.push_str("\nTools:\n");
        let tools = if request.tools.is_empty() {
            definitions(self.host.access, &self.disabled, self.only.as_ref())
        } else {
            request.tools.clone()
        };
        for tool in &tools {
            system.push_str(&format!(
                "- {}: {} Input: {}\n",
                tool.name, tool.description, tool.schema
            ));
        }
        ModelRequest {
            system,
            messages: request.messages.clone(),
            tools: Vec::new(),
        }
    }
}

/// Read a JSON-protocol turn: `{"tool":…,"input":…}` or `{"answer":…}`.
fn json_turn(completion: &Completion) -> (String, Vec<ToolCall>) {
    let text = completion.text.trim();
    let candidate = text
        .strip_prefix("```json")
        .or_else(|| text.strip_prefix("```"))
        .map_or(text, |rest| rest.trim_end_matches("```").trim());
    let start = candidate.find('{');
    let end = candidate.rfind('}');
    let value = match (start, end) {
        (Some(s), Some(e)) if e > s => serde_json::from_str::<Value>(&candidate[s..=e]).ok(),
        _ => None,
    };
    match value {
        Some(value) => {
            if let Some(tool) = value.get("tool").and_then(Value::as_str) {
                let input = value.get("input").cloned().unwrap_or_else(|| json!({}));
                return (
                    String::new(),
                    vec![ToolCall {
                        id: format!("json_{}", rand::random::<u32>()),
                        name: tool.to_owned(),
                        input: if input.is_object() { input } else { json!({}) },
                        error: None,
                    }],
                );
            }
            if let Some(answer) = value.get("answer").and_then(Value::as_str) {
                return (answer.to_owned(), Vec::new());
            }
            (completion.text.clone(), Vec::new())
        }
        None => (completion.text.clone(), Vec::new()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn completion(text: &str) -> Completion {
        Completion {
            text: text.into(),
            calls: Vec::new(),
            usage: None,
            finish: Finish::Stop,
            raw: None,
        }
    }

    #[test]
    fn the_json_protocol_is_read() {
        let (text, calls) = json_turn(&completion(
            "```json\n{\"tool\": \"canon.search\", \"input\": {\"query\": \"Mara\"}}\n```",
        ));
        assert!(text.is_empty());
        assert_eq!(calls[0].name, "canon.search");
        assert_eq!(calls[0].input["query"], "Mara");
        let (text, calls) = json_turn(&completion("{\"answer\": \"Mara has green eyes.\"}"));
        assert_eq!(text, "Mara has green eyes.");
        assert!(calls.is_empty());
        let (text, calls) = json_turn(&completion("Just words."));
        assert_eq!(text, "Just words.");
        assert!(calls.is_empty());
    }

    #[test]
    fn tool_unsupported_errors_are_recognised() {
        assert!(mentions_tools_unsupported(
            "registry.ollama.ai/library/gemma does not support tools"
        ));
        assert!(mentions_tools_unsupported(
            "This model does not support function calling"
        ));
        assert!(!mentions_tools_unsupported("Invalid API key"));
    }
}
