//! The Anthropic Messages API (`POST /v1/messages`, streamed).
//!
//! Request: `x-api-key`, `anthropic-version: 2023-06-01`, `system`, `tools`
//! with `input_schema`, user and assistant messages made of content blocks.
//! Response: server-sent events `message_start`, `content_block_start`,
//! `content_block_delta` (`text_delta`, `input_json_delta`, `thinking_delta`,
//! `signature_delta`), `content_block_stop`, `message_delta` (stop reason and
//! cumulative output tokens), `message_stop`, `ping`, and `error`.
//!
//! Thinking blocks must be sent back unchanged with their signature when the
//! model asked for tools, so the assistant turn is kept as the provider's own
//! content blocks (`raw`).

use std::collections::BTreeMap;

use serde_json::{Value, json};
use url::Url;

use super::{
    AssistantMsg, Auth, Completion, Decoder, Fault, Finish, ModelRequest, Msg, Piece, ToolCall,
    ToolResultMsg, Usage, Wire, from_wire_name, join, parse_arguments, sse::SseEvent, wire_name,
};
use crate::{config::ProviderConfig, error::Result};

/// The API version header value.
pub const VERSION: &str = "2023-06-01";

pub(super) fn build(provider: &ProviderConfig, base: &Url, request: &ModelRequest) -> Result<Wire> {
    let mut body = json!({
        "model": provider.model,
        "max_tokens": provider.max_output_tokens,
        "stream": true,
        "system": request.system,
        "messages": messages(&request.messages),
    });
    if !request.tools.is_empty() {
        body["tools"] = Value::Array(
            request
                .tools
                .iter()
                .map(|tool| {
                    json!({
                        "name": wire_name(&tool.name),
                        "description": tool.description,
                        "input_schema": tool.schema,
                    })
                })
                .collect(),
        );
    }
    Ok(Wire {
        url: join(base, "v1/messages")?,
        auth: Auth::Header("x-api-key"),
        headers: vec![
            ("anthropic-version".into(), VERSION.into()),
            ("content-type".into(), "application/json".into()),
        ],
        body,
    })
}

fn messages(history: &[Msg]) -> Vec<Value> {
    let mut out: Vec<Value> = Vec::new();
    for message in history {
        match message {
            Msg::User(text) => out.push(json!({ "role": "user", "content": text })),
            Msg::Assistant(turn) => out.push(json!({
                "role": "assistant",
                "content": assistant_blocks(turn),
            })),
            Msg::ToolResults(results) => out.push(json!({
                "role": "user",
                "content": results.iter().map(tool_result).collect::<Vec<_>>(),
            })),
        }
    }
    out
}

fn assistant_blocks(turn: &AssistantMsg) -> Value {
    if let Some(raw) = &turn.raw {
        return raw.clone();
    }
    let mut blocks = Vec::new();
    if !turn.text.is_empty() {
        blocks.push(json!({ "type": "text", "text": turn.text }));
    }
    for call in &turn.calls {
        blocks.push(json!({
            "type": "tool_use",
            "id": call.id,
            "name": wire_name(&call.name),
            "input": if call.input.is_null() { json!({}) } else { call.input.clone() },
        }));
    }
    if blocks.is_empty() {
        blocks.push(json!({ "type": "text", "text": "(no text)" }));
    }
    Value::Array(blocks)
}

fn tool_result(result: &ToolResultMsg) -> Value {
    let mut block = json!({
        "type": "tool_result",
        "tool_use_id": result.call_id,
        "content": result.content,
    });
    if result.is_error {
        block["is_error"] = json!(true);
    }
    block
}

enum Block {
    Text(String),
    Tool {
        id: String,
        name: String,
        json: String,
    },
    Thinking {
        thinking: String,
        signature: String,
    },
    /// Anything else, kept as received (redacted thinking, server tools).
    Other(Value),
}

/// Decodes the Messages stream.
#[derive(Default)]
pub struct AnthropicDecoder {
    blocks: BTreeMap<usize, Block>,
    input_tokens: u64,
    output_tokens: u64,
    stop: Option<String>,
    saw_usage: bool,
    stopped: bool,
}

impl Decoder for AnthropicDecoder {
    fn event(&mut self, event: &SseEvent, out: &mut Vec<Piece>) -> std::result::Result<(), Fault> {
        if event.data.trim().is_empty() {
            return Ok(());
        }
        let data: Value = serde_json::from_str(&event.data)
            .map_err(|_| Fault::Malformed("a stream event was not JSON".into()))?;
        let kind = data
            .get("type")
            .and_then(Value::as_str)
            .or(event.event.as_deref())
            .unwrap_or_default();
        match kind {
            "message_start" => {
                let usage = data.pointer("/message/usage");
                if let Some(usage) = usage {
                    let input = usage
                        .get("input_tokens")
                        .and_then(Value::as_u64)
                        .unwrap_or(0)
                        + usage
                            .get("cache_creation_input_tokens")
                            .and_then(Value::as_u64)
                            .unwrap_or(0)
                        + usage
                            .get("cache_read_input_tokens")
                            .and_then(Value::as_u64)
                            .unwrap_or(0);
                    self.input_tokens = input;
                    self.output_tokens = usage
                        .get("output_tokens")
                        .and_then(Value::as_u64)
                        .unwrap_or(0);
                    self.saw_usage = true;
                }
            }
            "content_block_start" => {
                let index = index_of(&data)?;
                let block = data.get("content_block").cloned().unwrap_or(Value::Null);
                let entry = match block.get("type").and_then(Value::as_str) {
                    Some("text") => {
                        let text = block
                            .get("text")
                            .and_then(Value::as_str)
                            .unwrap_or_default()
                            .to_owned();
                        if !text.is_empty() {
                            out.push(Piece::Text(text.clone()));
                        }
                        Block::Text(text)
                    }
                    Some("tool_use") => Block::Tool {
                        id: block
                            .get("id")
                            .and_then(Value::as_str)
                            .unwrap_or_default()
                            .to_owned(),
                        name: block
                            .get("name")
                            .and_then(Value::as_str)
                            .unwrap_or_default()
                            .to_owned(),
                        json: String::new(),
                    },
                    Some("thinking") => Block::Thinking {
                        thinking: block
                            .get("thinking")
                            .and_then(Value::as_str)
                            .unwrap_or_default()
                            .to_owned(),
                        signature: String::new(),
                    },
                    _ => Block::Other(block),
                };
                self.blocks.insert(index, entry);
            }
            "content_block_delta" => {
                let index = index_of(&data)?;
                let delta = data.get("delta").cloned().unwrap_or(Value::Null);
                let Some(block) = self.blocks.get_mut(&index) else {
                    return Ok(());
                };
                match (delta.get("type").and_then(Value::as_str), block) {
                    (Some("text_delta"), Block::Text(text)) => {
                        let piece = delta
                            .get("text")
                            .and_then(Value::as_str)
                            .unwrap_or_default();
                        text.push_str(piece);
                        if !piece.is_empty() {
                            out.push(Piece::Text(piece.to_owned()));
                        }
                    }
                    (Some("input_json_delta"), Block::Tool { json, .. }) => {
                        json.push_str(
                            delta
                                .get("partial_json")
                                .and_then(Value::as_str)
                                .unwrap_or_default(),
                        );
                    }
                    (Some("thinking_delta"), Block::Thinking { thinking, .. }) => {
                        thinking.push_str(
                            delta
                                .get("thinking")
                                .and_then(Value::as_str)
                                .unwrap_or_default(),
                        );
                    }
                    (Some("signature_delta"), Block::Thinking { signature, .. }) => {
                        signature.push_str(
                            delta
                                .get("signature")
                                .and_then(Value::as_str)
                                .unwrap_or_default(),
                        );
                    }
                    _ => {}
                }
            }
            "message_delta" => {
                if let Some(reason) = data.pointer("/delta/stop_reason").and_then(Value::as_str) {
                    self.stop = Some(reason.to_owned());
                }
                if let Some(usage) = data.get("usage") {
                    if let Some(output) = usage.get("output_tokens").and_then(Value::as_u64) {
                        self.output_tokens = output;
                        self.saw_usage = true;
                    }
                    if let Some(input) = usage.get("input_tokens").and_then(Value::as_u64) {
                        if input > 0 {
                            self.input_tokens = input;
                        }
                    }
                }
            }
            "message_stop" => self.stopped = true,
            "error" => {
                let error = data.get("error").cloned().unwrap_or(Value::Null);
                return Err(Fault::Api {
                    status: 0,
                    message: error
                        .get("message")
                        .and_then(Value::as_str)
                        .unwrap_or("the provider reported an error")
                        .to_owned(),
                    code: error.get("type").and_then(Value::as_str).map(str::to_owned),
                });
            }
            _ => {}
        }
        Ok(())
    }

    fn finish(self: Box<Self>) -> std::result::Result<Completion, Fault> {
        if self.blocks.is_empty() && !self.stopped && self.stop.is_none() {
            return Err(Fault::Malformed(
                "the stream ended before any answer".into(),
            ));
        }
        let mut text = String::new();
        let mut calls = Vec::new();
        let mut raw = Vec::new();
        for block in self.blocks.into_values() {
            match block {
                Block::Text(t) => {
                    if !t.is_empty() {
                        raw.push(json!({ "type": "text", "text": t }));
                        text.push_str(&t);
                    }
                }
                Block::Tool { id, name, json } => {
                    let (input, error) = parse_arguments(&json);
                    raw.push(json!({
                        "type": "tool_use",
                        "id": id,
                        "name": name,
                        "input": if input.is_null() { json!({}) } else { input.clone() },
                    }));
                    calls.push(ToolCall {
                        id,
                        name: from_wire_name(&name),
                        input,
                        error,
                    });
                }
                Block::Thinking {
                    thinking,
                    signature,
                } => raw.push(json!({
                    "type": "thinking",
                    "thinking": thinking,
                    "signature": signature,
                })),
                Block::Other(value) => raw.push(value),
            }
        }
        let finish = match self.stop.as_deref() {
            Some("tool_use") => Finish::ToolUse,
            Some("max_tokens") => Finish::Length,
            Some("refusal") => Finish::Refusal,
            Some("end_turn" | "stop_sequence") => Finish::Stop,
            _ if !calls.is_empty() => Finish::ToolUse,
            Some(_) => Finish::Other,
            None => Finish::Stop,
        };
        Ok(Completion {
            text,
            calls,
            usage: self.saw_usage.then_some(Usage {
                input: self.input_tokens,
                output: self.output_tokens,
            }),
            finish,
            raw: Some(Value::Array(raw)),
        })
    }
}

fn index_of(data: &Value) -> std::result::Result<usize, Fault> {
    data.get("index")
        .and_then(Value::as_u64)
        .map(|n| n as usize)
        .ok_or_else(|| Fault::Malformed("a content block had no index".into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::{ToolDef, sse::SseParser};

    /// A recorded-shape stream: text, then a tool call with split JSON.
    pub(crate) const TOOL_STREAM: &str = "\
event: message_start
data: {\"type\":\"message_start\",\"message\":{\"id\":\"msg_01\",\"type\":\"message\",\"role\":\"assistant\",\"content\":[],\"model\":\"claude-haiku-5-5\",\"stop_reason\":null,\"stop_sequence\":null,\"usage\":{\"input_tokens\":472,\"output_tokens\":2}}}

event: content_block_start
data: {\"type\":\"content_block_start\",\"index\":0,\"content_block\":{\"type\":\"thinking\",\"thinking\":\"\"}}

event: content_block_delta
data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"thinking_delta\",\"thinking\":\"The author wants the marks.\"}}

event: content_block_delta
data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"signature_delta\",\"signature\":\"EqQBCgIYAhIM\"}}

event: content_block_stop
data: {\"type\":\"content_block_stop\",\"index\":0}

event: content_block_start
data: {\"type\":\"content_block_start\",\"index\":1,\"content_block\":{\"type\":\"text\",\"text\":\"\"}}

event: ping
data: {\"type\":\"ping\"}

event: content_block_delta
data: {\"type\":\"content_block_delta\",\"index\":1,\"delta\":{\"type\":\"text_delta\",\"text\":\"Let me look.\"}}

event: content_block_stop
data: {\"type\":\"content_block_stop\",\"index\":1}

event: content_block_start
data: {\"type\":\"content_block_start\",\"index\":2,\"content_block\":{\"type\":\"tool_use\",\"id\":\"toolu_01A\",\"name\":\"marks__list\",\"input\":{}}}

event: content_block_delta
data: {\"type\":\"content_block_delta\",\"index\":2,\"delta\":{\"type\":\"input_json_delta\",\"partial_json\":\"\"}}

event: content_block_delta
data: {\"type\":\"content_block_delta\",\"index\":2,\"delta\":{\"type\":\"input_json_delta\",\"partial_json\":\"{\\\"severity\\\":\"}}

event: content_block_delta
data: {\"type\":\"content_block_delta\",\"index\":2,\"delta\":{\"type\":\"input_json_delta\",\"partial_json\":\" \\\"error\\\"}\"}}

event: content_block_stop
data: {\"type\":\"content_block_stop\",\"index\":2}

event: message_delta
data: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"tool_use\",\"stop_sequence\":null},\"usage\":{\"output_tokens\":89}}

event: message_stop
data: {\"type\":\"message_stop\"}

";

    fn run(stream: &str) -> (String, Completion) {
        let mut parser = SseParser::new();
        let mut decoder: Box<dyn Decoder> = Box::new(AnthropicDecoder::default());
        let mut streamed = String::new();
        for event in parser.push(stream.as_bytes()).unwrap() {
            let mut pieces = Vec::new();
            decoder.event(&event, &mut pieces).unwrap();
            for Piece::Text(t) in pieces {
                streamed.push_str(&t);
            }
        }
        (streamed, decoder.finish().unwrap())
    }

    #[test]
    fn a_tool_call_stream_is_decoded() {
        let (streamed, done) = run(TOOL_STREAM);
        assert_eq!(streamed, "Let me look.");
        assert_eq!(done.text, "Let me look.");
        assert_eq!(done.finish, Finish::ToolUse);
        assert_eq!(done.calls.len(), 1);
        assert_eq!(done.calls[0].name, "marks.list");
        assert_eq!(done.calls[0].id, "toolu_01A");
        assert_eq!(done.calls[0].input["severity"], "error");
        assert_eq!(
            done.usage,
            Some(Usage {
                input: 472,
                output: 89
            })
        );
        // The thinking block is replayed with its signature.
        let raw = done.raw.unwrap();
        assert_eq!(raw[0]["type"], "thinking");
        assert_eq!(raw[0]["signature"], "EqQBCgIYAhIM");
        assert_eq!(raw[2]["type"], "tool_use");
        assert_eq!(raw[2]["name"], "marks__list");
    }

    #[test]
    fn an_error_event_is_a_fault() {
        let mut decoder = AnthropicDecoder::default();
        let event = SseEvent {
            event: Some("error".into()),
            data: "{\"type\":\"error\",\"error\":{\"type\":\"overloaded_error\",\"message\":\"Overloaded\"}}".into(),
        };
        let fault = decoder.event(&event, &mut Vec::new()).unwrap_err();
        assert_eq!(
            fault,
            Fault::Api {
                status: 0,
                message: "Overloaded".into(),
                code: Some("overloaded_error".into())
            }
        );
    }

    #[test]
    fn the_request_has_the_documented_shape() {
        let provider = crate::config::default_providers()
            .into_iter()
            .find(|p| p.id == "anthropic")
            .unwrap();
        let request = ModelRequest {
            system: "SYS".into(),
            messages: vec![
                Msg::User("hello".into()),
                Msg::Assistant(AssistantMsg {
                    text: String::new(),
                    calls: vec![ToolCall {
                        id: "toolu_1".into(),
                        name: "marks.list".into(),
                        input: json!({}),
                        error: None,
                    }],
                    raw: None,
                }),
                Msg::ToolResults(vec![ToolResultMsg {
                    call_id: "toolu_1".into(),
                    name: "marks.list".into(),
                    content: "2 marks".into(),
                    is_error: false,
                }]),
            ],
            tools: vec![ToolDef {
                name: "marks.list".into(),
                description: "d".into(),
                schema: json!({"type":"object","properties":{}}),
            }],
        };
        let base = Url::parse(&provider.endpoint).unwrap();
        let wire = build(&provider, &base, &request).unwrap();
        assert_eq!(wire.url.as_str(), "https://api.anthropic.com/v1/messages");
        assert_eq!(wire.auth, Auth::Header("x-api-key"));
        assert!(
            wire.headers
                .iter()
                .any(|(k, v)| k == "anthropic-version" && v == VERSION)
        );
        assert_eq!(wire.body["stream"], true);
        assert_eq!(wire.body["system"], "SYS");
        assert_eq!(wire.body["tools"][0]["name"], "marks__list");
        assert_eq!(wire.body["tools"][0]["input_schema"]["type"], "object");
        assert_eq!(wire.body["messages"][1]["content"][0]["type"], "tool_use");
        assert_eq!(wire.body["messages"][2]["role"], "user");
        assert_eq!(
            wire.body["messages"][2]["content"][0]["tool_use_id"],
            "toolu_1"
        );
    }
}
