//! Provider adapters.
//!
//! Three wire formats cover every provider Asten offers:
//!
//! - [`anthropic`]: the Messages API;
//! - [`openai`]: Chat Completions, spoken by OpenAI and by Mistral, DeepSeek,
//!   Qwen (DashScope), Kimi, GLM, Ollama and any compatible service;
//! - [`gemini`][]: `streamGenerateContent`.
//!
//! An adapter turns a provider-neutral [`ModelRequest`] into a [`Wire`] (URL,
//! headers without the key, and the exact JSON body) and a [`Decoder`] that
//! turns the streamed events back into text, tool calls and token counts. The
//! [`transport`] sends the wire with the key and feeds the decoder.

use serde::Serialize;
use serde_json::Value;
use url::Url;

use crate::{
    config::{Protocol, ProviderConfig},
    error::{AssistantError, Result},
};

#[path = "../../../../shared/local_agent.rs"]
pub mod local_agent;

pub mod anthropic;
pub mod gemini;
pub mod openai;
pub mod sse;
pub mod transport;

use sse::SseEvent;

/// A tool as offered to a model.
#[derive(Clone, Debug, Serialize)]
pub struct ToolDef {
    /// The closed-table name (`marks.list`).
    pub name: String,
    /// What it does.
    pub description: String,
    /// JSON Schema of its input.
    pub schema: Value,
}

/// Providers allow `[a-zA-Z0-9_-]` in tool names; the table's dots become
/// double underscores on the wire.
#[must_use]
pub fn wire_name(name: &str) -> String {
    name.replace('.', "__")
}

/// The inverse of [`wire_name`].
#[must_use]
pub fn from_wire_name(name: &str) -> String {
    name.replace("__", ".")
}

/// A tool call the model made.
#[derive(Clone, Debug, PartialEq)]
pub struct ToolCall {
    /// The provider's id for the call (generated when it sent none).
    pub id: String,
    /// The table name (`marks.list`), or whatever the model invented.
    pub name: String,
    /// The arguments; `Null` when they could not be parsed.
    pub input: Value,
    /// Set when the arguments could not be parsed.
    pub error: Option<String>,
}

/// One message in the conversation the adapter serialises.
#[derive(Clone, Debug)]
pub enum Msg {
    /// The writer's words (and anything they attached).
    User(String),
    /// What the model said, with the calls it made.
    Assistant(AssistantMsg),
    /// The results of the calls in the preceding assistant message.
    ToolResults(Vec<ToolResultMsg>),
}

/// The model's turn.
#[derive(Clone, Debug)]
pub struct AssistantMsg {
    /// Visible text.
    pub text: String,
    /// Calls made.
    pub calls: Vec<ToolCall>,
    /// The provider's own representation, replayed verbatim on the next call
    /// (thinking blocks and signatures, reasoning content, thought signatures).
    pub raw: Option<Value>,
}

/// One tool result.
#[derive(Clone, Debug)]
pub struct ToolResultMsg {
    /// The call it answers.
    pub call_id: String,
    /// The tool's table name.
    pub name: String,
    /// The result, as text. Untrusted: the system prompt says so.
    pub content: String,
    /// The call was refused or failed.
    pub is_error: bool,
}

/// A request in provider-neutral form.
#[derive(Clone, Debug)]
pub struct ModelRequest {
    /// The system prompt.
    pub system: String,
    /// The conversation so far.
    pub messages: Vec<Msg>,
    /// Tools offered (empty in the JSON protocol).
    pub tools: Vec<ToolDef>,
}

/// Behaviour learned from a provider's answers during a run.
#[derive(Clone, Copy, Debug)]
pub struct Quirks {
    /// Ask for token usage in the stream (`stream_options`).
    pub stream_usage: bool,
}

impl Default for Quirks {
    fn default() -> Self {
        Self { stream_usage: true }
    }
}

/// How the key travels.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Auth {
    /// No key (a model on this machine).
    None,
    /// `Authorization: Bearer <key>`.
    Bearer,
    /// A named header (`x-api-key`, `x-goog-api-key`).
    Header(&'static str),
}

/// A request ready to send, minus the key.
#[derive(Clone, Debug)]
pub struct Wire {
    /// Where it goes.
    pub url: Url,
    /// How the key is attached.
    pub auth: Auth,
    /// Other headers (never the key).
    pub headers: Vec<(String, String)>,
    /// The JSON body, exactly as it will be sent.
    pub body: Value,
}

impl Wire {
    /// The body bytes that go on the wire.
    #[must_use]
    pub fn body_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(&self.body).unwrap_or_default()
    }
}

/// Tokens a call used.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Usage {
    /// Tokens in the request.
    pub input: u64,
    /// Tokens in the answer (including any hidden reasoning).
    pub output: u64,
}

/// Why the model stopped.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Finish {
    /// A natural end.
    Stop,
    /// It wants tool results.
    ToolUse,
    /// The output limit was reached.
    Length,
    /// The provider declined to answer.
    Refusal,
    /// Anything else.
    Other,
}

/// A finished model turn.
#[derive(Clone, Debug)]
pub struct Completion {
    /// The visible text.
    pub text: String,
    /// The tool calls.
    pub calls: Vec<ToolCall>,
    /// Token usage, if the provider reported it.
    pub usage: Option<Usage>,
    /// Why it stopped.
    pub finish: Finish,
    /// What to replay on the next request.
    pub raw: Option<Value>,
}

/// A piece of the answer as it streams.
#[derive(Clone, Debug)]
pub enum Piece {
    /// Visible text.
    Text(String),
}

/// A problem found while decoding.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Fault {
    /// The provider sent an error (HTTP status when known, else 0).
    Api {
        /// Status, or 0.
        status: u16,
        /// The provider's message.
        message: String,
        /// The provider's error type or code.
        code: Option<String>,
    },
    /// The stream did not make sense.
    Malformed(String),
}

/// Turns streamed events into a [`Completion`].
pub trait Decoder: Send {
    /// Handle one event; text pieces go to `out`.
    fn event(&mut self, event: &SseEvent, out: &mut Vec<Piece>) -> std::result::Result<(), Fault>;
    /// The stream ended.
    fn finish(self: Box<Self>) -> std::result::Result<Completion, Fault>;
}

/// Build the wire request for a provider.
pub fn build(provider: &ProviderConfig, request: &ModelRequest, quirks: Quirks) -> Result<Wire> {
    let url = crate::config::validate_endpoint(&provider.endpoint)?;
    match provider.protocol {
        Protocol::Anthropic => anthropic::build(provider, &url, request),
        Protocol::Openai => openai::build(provider, &url, request, quirks),
        Protocol::Gemini => gemini::build(provider, &url, request),
    }
}

/// A fresh decoder for a provider's stream.
#[must_use]
pub fn decoder(provider: &ProviderConfig) -> Box<dyn Decoder> {
    match provider.protocol {
        Protocol::Anthropic => Box::new(anthropic::AnthropicDecoder::default()),
        Protocol::Openai => Box::new(openai::OpenAiDecoder::default()),
        Protocol::Gemini => Box::new(gemini::GeminiDecoder::default()),
    }
}

/// Join `path` onto a base URL, keeping the base's own path.
pub(crate) fn join(base: &Url, path: &str) -> Result<Url> {
    let mut text = base.as_str().trim_end_matches('/').to_owned();
    text.push('/');
    text.push_str(path.trim_start_matches('/'));
    Url::parse(&text).map_err(|_| AssistantError::Invalid("The endpoint cannot be extended".into()))
}

/// Parse tool-call arguments.
pub(crate) fn parse_arguments(arguments: &str) -> (Value, Option<String>) {
    let trimmed = arguments.trim();
    if trimmed.is_empty() {
        return (Value::Object(serde_json::Map::new()), None);
    }
    match serde_json::from_str::<Value>(trimmed) {
        Ok(Value::Object(map)) => (Value::Object(map), None),
        Ok(_) => (
            Value::Null,
            Some("The tool arguments were not a JSON object".into()),
        ),
        Err(error) => (
            Value::Null,
            Some(format!("The tool arguments were not valid JSON: {error}")),
        ),
    }
}
