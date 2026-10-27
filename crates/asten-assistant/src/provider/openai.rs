//! Chat Completions, as spoken by OpenAI and by Mistral, DeepSeek, Qwen
//! (DashScope), Kimi, GLM, Ollama and any compatible service.
//!
//! Request: `POST {base}/chat/completions` with `Authorization: Bearer`,
//! `messages` (system, user, assistant with `tool_calls`, `tool` results),
//! `tools` of `type: "function"`, `stream: true` and (to learn the cost)
//! `stream_options.include_usage`. Response: `data:` chunks with
//! `choices[0].delta` (`content`, `tool_calls` fragments keyed by `index`,
//! and, on reasoning models, `reasoning_content`), a final chunk with `usage`,
//! and `data: [DONE]`.
//!
//! Reasoning models (DeepSeek, Kimi, GLM) want their `reasoning_content` sent
//! back with the assistant turn that called a tool; the turn is therefore kept
//! as received (`raw`).

use std::collections::BTreeMap;

use serde_json::{Value, json};
use url::Url;

use super::{
    AssistantMsg, Auth, Completion, Decoder, Fault, Finish, ModelRequest, Msg, Piece, Quirks,
    ToolCall, ToolResultMsg, Usage, Wire, from_wire_name, join, parse_arguments, sse::SseEvent,
    wire_name,
};
use crate::{config::ProviderConfig, error::Result};

pub(super) fn build(
    provider: &ProviderConfig,
    base: &Url,
    request: &ModelRequest,
    quirks: Quirks,
) -> Result<Wire> {
    let mut messages = vec![json!({ "role": "system", "content": request.system })];
    messages.extend(convert(&request.messages, provider.family == "mistral"));
    let mut body = json!({
        "model": provider.model,
        "messages": messages,
        "stream": true,
    });
    let limit = if provider.family == "openai" {
        "max_completion_tokens"
    } else {
        "max_tokens"
    };
    body[limit] = json!(provider.max_output_tokens);
    if quirks.stream_usage {
        body["stream_options"] = json!({ "include_usage": true });
    }
    if !request.tools.is_empty() {
        body["tools"] = Value::Array(
            request
                .tools
                .iter()
                .map(|tool| {
                    json!({
                        "type": "function",
                        "function": {
                            "name": wire_name(&tool.name),
                            "description": tool.description,
                            "parameters": tool.schema,
                        }
                    })
                })
                .collect(),
        );
    }
    Ok(Wire {
        url: join(base, "chat/completions")?,
        auth: Auth::Bearer,
        headers: vec![("content-type".into(), "application/json".into())],
        body,
    })
}

fn convert(history: &[Msg], name_on_tool_messages: bool) -> Vec<Value> {
    let mut out = Vec::new();
    for message in history {
        match message {
            Msg::User(text) => out.push(json!({ "role": "user", "content": text })),
            Msg::Assistant(turn) => out.push(assistant(turn)),
            Msg::ToolResults(results) => {
                for result in results {
                    out.push(tool_message(result, name_on_tool_messages));
                }
            }
        }
    }
    out
}

fn assistant(turn: &AssistantMsg) -> Value {
    if let Some(raw) = &turn.raw {
        return raw.clone();
    }
    let mut message = json!({
        "role": "assistant",
        "content": if turn.text.is_empty() { Value::Null } else { json!(turn.text) },
    });
    if !turn.calls.is_empty() {
        message["tool_calls"] = Value::Array(
            turn.calls
                .iter()
                .map(|call| {
                    json!({
                        "id": call.id,
                        "type": "function",
                        "function": {
                            "name": wire_name(&call.name),
                            "arguments": if call.input.is_null() {
                                "{}".to_owned()
                            } else {
                                call.input.to_string()
                            },
                        }
                    })
                })
                .collect(),
        );
    }
    message
}

fn tool_message(result: &ToolResultMsg, with_name: bool) -> Value {
    let mut message = json!({
        "role": "tool",
        "tool_call_id": result.call_id,
        "content": result.content,
    });
    if with_name {
        message["name"] = json!(wire_name(&result.name));
    }
    message
}

#[derive(Default)]
struct PartialCall {
    id: String,
    name: String,
    arguments: String,
}

/// Decodes a Chat Completions stream.
#[derive(Default)]
pub struct OpenAiDecoder {
    text: String,
    reasoning: String,
    calls: BTreeMap<usize, PartialCall>,
    finish: Option<String>,
    usage: Option<Usage>,
    chunks: usize,
    done: bool,
}

impl Decoder for OpenAiDecoder {
    fn event(&mut self, event: &SseEvent, out: &mut Vec<Piece>) -> std::result::Result<(), Fault> {
        let data = event.data.trim();
        if data.is_empty() {
            return Ok(());
        }
        if data == "[DONE]" {
            self.done = true;
            return Ok(());
        }
        let chunk: Value = serde_json::from_str(data)
            .map_err(|_| Fault::Malformed("a stream chunk was not JSON".into()))?;
        if let Some(error) = chunk.get("error").filter(|e| !e.is_null()) {
            return Err(Fault::Api {
                status: error
                    .get("code")
                    .and_then(Value::as_u64)
                    .map_or(0, |n| n as u16),
                message: error
                    .get("message")
                    .and_then(Value::as_str)
                    .unwrap_or("the provider reported an error")
                    .to_owned(),
                code: error
                    .get("type")
                    .or_else(|| error.get("code"))
                    .and_then(Value::as_str)
                    .map(str::to_owned),
            });
        }
        self.chunks += 1;
        let usage = chunk
            .get("usage")
            .filter(|u| !u.is_null())
            .or_else(|| chunk.pointer("/choices/0/usage").filter(|u| !u.is_null()));
        if let Some(usage) = usage {
            let input = usage.get("prompt_tokens").and_then(Value::as_u64);
            let output = usage.get("completion_tokens").and_then(Value::as_u64);
            if input.is_some() || output.is_some() {
                self.usage = Some(Usage {
                    input: input.unwrap_or(0),
                    output: output.unwrap_or(0),
                });
            }
        }
        let Some(choice) = chunk.pointer("/choices/0") else {
            return Ok(());
        };
        if let Some(reason) = choice.get("finish_reason").and_then(Value::as_str) {
            self.finish = Some(reason.to_owned());
        }
        // Some servers send a whole message instead of a delta.
        let delta = choice.get("delta").or_else(|| choice.get("message"));
        let Some(delta) = delta else {
            return Ok(());
        };
        if let Some(content) = delta.get("content").and_then(Value::as_str) {
            if !content.is_empty() {
                self.text.push_str(content);
                out.push(Piece::Text(content.to_owned()));
            }
        }
        if let Some(reasoning) = delta
            .get("reasoning_content")
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty())
        {
            self.reasoning.push_str(reasoning);
        }
        if let Some(calls) = delta.get("tool_calls").and_then(Value::as_array) {
            for (position, call) in calls.iter().enumerate() {
                let index = call
                    .get("index")
                    .and_then(Value::as_u64)
                    .map_or(position, |n| n as usize);
                let entry = self.calls.entry(index).or_default();
                if let Some(id) = call
                    .get("id")
                    .and_then(Value::as_str)
                    .filter(|s| !s.is_empty())
                {
                    entry.id = id.to_owned();
                }
                if let Some(function) = call.get("function") {
                    if let Some(name) = function
                        .get("name")
                        .and_then(Value::as_str)
                        .filter(|s| !s.is_empty())
                    {
                        entry.name = name.to_owned();
                    }
                    match function.get("arguments") {
                        Some(Value::String(part)) => entry.arguments.push_str(part),
                        // A few services send the arguments as an object.
                        Some(object @ Value::Object(_)) => entry.arguments = object.to_string(),
                        _ => {}
                    }
                }
            }
        }
        Ok(())
    }

    fn finish(self: Box<Self>) -> std::result::Result<Completion, Fault> {
        if self.chunks == 0 && !self.done {
            return Err(Fault::Malformed(
                "the stream ended before any answer".into(),
            ));
        }
        let mut calls = Vec::new();
        let mut raw_calls = Vec::new();
        for (index, partial) in self.calls {
            if partial.name.is_empty() {
                continue;
            }
            let id = if partial.id.is_empty() {
                format!("call_{index}")
            } else {
                partial.id
            };
            let (input, error) = parse_arguments(&partial.arguments);
            raw_calls.push(json!({
                "id": id,
                "type": "function",
                "function": {
                    "name": partial.name,
                    "arguments": if partial.arguments.trim().is_empty() {
                        "{}".to_owned()
                    } else {
                        partial.arguments.clone()
                    },
                }
            }));
            calls.push(ToolCall {
                id,
                name: from_wire_name(&partial.name),
                input,
                error,
            });
        }
        let finish = match self.finish.as_deref() {
            Some("tool_calls" | "function_call") => Finish::ToolUse,
            Some("length") => Finish::Length,
            Some("content_filter") => Finish::Refusal,
            _ if !calls.is_empty() => Finish::ToolUse,
            Some("stop") | None => Finish::Stop,
            Some(_) => Finish::Other,
        };
        let mut raw = json!({
            "role": "assistant",
            "content": if self.text.is_empty() && !calls.is_empty() { Value::Null } else { json!(self.text) },
        });
        if !raw_calls.is_empty() {
            raw["tool_calls"] = Value::Array(raw_calls);
        }
        if !self.reasoning.is_empty() {
            raw["reasoning_content"] = json!(self.reasoning);
        }
        Ok(Completion {
            text: self.text,
            calls,
            usage: self.usage,
            finish,
            raw: Some(raw),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::{ToolDef, sse::SseParser};

    /// The shape OpenAI streams for a tool call, with a final usage chunk.
    pub(crate) const TOOL_STREAM: &str = "\
data: {\"id\":\"chatcmpl-1\",\"object\":\"chat.completion.chunk\",\"choices\":[{\"index\":0,\"delta\":{\"role\":\"assistant\",\"content\":\"\"},\"finish_reason\":null}],\"usage\":null}

data: {\"id\":\"chatcmpl-1\",\"object\":\"chat.completion.chunk\",\"choices\":[{\"index\":0,\"delta\":{\"content\":\"One moment.\"},\"finish_reason\":null}],\"usage\":null}

data: {\"id\":\"chatcmpl-1\",\"object\":\"chat.completion.chunk\",\"choices\":[{\"index\":0,\"delta\":{\"tool_calls\":[{\"index\":0,\"id\":\"call_abc\",\"type\":\"function\",\"function\":{\"name\":\"canon__search\",\"arguments\":\"\"}}]},\"finish_reason\":null}],\"usage\":null}

data: {\"id\":\"chatcmpl-1\",\"object\":\"chat.completion.chunk\",\"choices\":[{\"index\":0,\"delta\":{\"tool_calls\":[{\"index\":0,\"function\":{\"arguments\":\"{\\\"query\\\":\"}}]},\"finish_reason\":null}],\"usage\":null}

data: {\"id\":\"chatcmpl-1\",\"object\":\"chat.completion.chunk\",\"choices\":[{\"index\":0,\"delta\":{\"tool_calls\":[{\"index\":0,\"function\":{\"arguments\":\" \\\"Mara\\\"}\"}}]},\"finish_reason\":null}],\"usage\":null}

data: {\"id\":\"chatcmpl-1\",\"object\":\"chat.completion.chunk\",\"choices\":[{\"index\":0,\"delta\":{},\"finish_reason\":\"tool_calls\"}],\"usage\":null}

data: {\"id\":\"chatcmpl-1\",\"object\":\"chat.completion.chunk\",\"choices\":[],\"usage\":{\"prompt_tokens\":311,\"completion_tokens\":27,\"total_tokens\":338}}

data: [DONE]

";

    fn run(stream: &str) -> (String, Completion) {
        let mut parser = SseParser::new();
        let mut decoder: Box<dyn Decoder> = Box::new(OpenAiDecoder::default());
        let mut streamed = String::new();
        let mut events = parser.push(stream.as_bytes()).unwrap();
        events.extend(parser.finish());
        for event in events {
            let mut pieces = Vec::new();
            decoder.event(&event, &mut pieces).unwrap();
            for Piece::Text(t) in pieces {
                streamed.push_str(&t);
            }
        }
        (streamed, decoder.finish().unwrap())
    }

    #[test]
    fn tool_call_fragments_are_assembled() {
        let (streamed, done) = run(TOOL_STREAM);
        assert_eq!(streamed, "One moment.");
        assert_eq!(done.finish, Finish::ToolUse);
        assert_eq!(done.calls.len(), 1);
        assert_eq!(done.calls[0].name, "canon.search");
        assert_eq!(done.calls[0].id, "call_abc");
        assert_eq!(done.calls[0].input["query"], "Mara");
        assert_eq!(
            done.usage,
            Some(Usage {
                input: 311,
                output: 27
            })
        );
        let raw = done.raw.unwrap();
        assert_eq!(raw["tool_calls"][0]["function"]["name"], "canon__search");
        assert!(raw.get("reasoning_content").is_none());
    }

    #[test]
    fn reasoning_content_is_kept_for_the_next_turn() {
        let stream = "\
data: {\"choices\":[{\"index\":0,\"delta\":{\"reasoning_content\":\"Think. \"}}]}

data: {\"choices\":[{\"index\":0,\"delta\":{\"reasoning_content\":\"Then call.\"}}]}

data: {\"choices\":[{\"index\":0,\"delta\":{\"tool_calls\":[{\"index\":0,\"id\":\"c1\",\"function\":{\"name\":\"marks__list\",\"arguments\":\"{}\"}}]},\"finish_reason\":\"tool_calls\",\"usage\":{\"prompt_tokens\":10,\"completion_tokens\":4}}]}

data: [DONE]

";
        let (_, done) = run(stream);
        let raw = done.raw.unwrap();
        assert_eq!(raw["reasoning_content"], "Think. Then call.");
        assert_eq!(
            done.usage,
            Some(Usage {
                input: 10,
                output: 4
            })
        );
        assert_eq!(done.calls[0].name, "marks.list");
    }

    #[test]
    fn a_missing_call_id_is_generated_and_bad_arguments_are_reported() {
        let stream = "\
data: {\"choices\":[{\"index\":0,\"delta\":{\"tool_calls\":[{\"index\":0,\"function\":{\"name\":\"canon__search\",\"arguments\":\"{oops\"}}]},\"finish_reason\":\"tool_calls\"}]}

data: [DONE]

";
        let (_, done) = run(stream);
        assert_eq!(done.calls[0].id, "call_0");
        assert!(done.calls[0].error.is_some());
        assert!(done.calls[0].input.is_null());
    }

    #[test]
    fn an_error_chunk_is_a_fault() {
        let mut decoder = OpenAiDecoder::default();
        let event = SseEvent {
            event: None,
            data: "{\"error\":{\"message\":\"The server had an error\",\"type\":\"server_error\"}}"
                .into(),
        };
        assert!(matches!(
            decoder.event(&event, &mut Vec::new()),
            Err(Fault::Api { .. })
        ));
    }

    #[test]
    fn the_request_follows_each_family() {
        let defaults = crate::config::default_providers();
        let request = ModelRequest {
            system: "SYS".into(),
            messages: vec![
                Msg::User("hi".into()),
                Msg::Assistant(AssistantMsg {
                    text: String::new(),
                    calls: vec![ToolCall {
                        id: "call_1".into(),
                        name: "marks.list".into(),
                        input: json!({"severity": "error"}),
                        error: None,
                    }],
                    raw: None,
                }),
                Msg::ToolResults(vec![ToolResultMsg {
                    call_id: "call_1".into(),
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
        for (id, url, limit) in [
            (
                "openai",
                "https://api.openai.com/v1/chat/completions",
                "max_completion_tokens",
            ),
            (
                "mistral",
                "https://api.mistral.ai/v1/chat/completions",
                "max_tokens",
            ),
            (
                "deepseek",
                "https://api.deepseek.com/chat/completions",
                "max_tokens",
            ),
            (
                "qwen",
                "https://dashscope-intl.aliyuncs.com/compatible-mode/v1/chat/completions",
                "max_tokens",
            ),
            (
                "kimi",
                "https://api.moonshot.ai/v1/chat/completions",
                "max_tokens",
            ),
            (
                "glm",
                "https://api.z.ai/api/paas/v4/chat/completions",
                "max_tokens",
            ),
            (
                "ollama",
                "http://127.0.0.1:11434/v1/chat/completions",
                "max_tokens",
            ),
        ] {
            let provider = defaults.iter().find(|p| p.id == id).unwrap();
            let base = Url::parse(&provider.endpoint).unwrap();
            let wire = build(provider, &base, &request, Quirks::default()).unwrap();
            assert_eq!(wire.url.as_str(), url, "{id}");
            assert!(wire.body.get(limit).is_some(), "{id} uses {limit}");
            assert_eq!(wire.body["stream_options"]["include_usage"], true);
            assert_eq!(wire.body["messages"][0]["role"], "system");
            assert_eq!(wire.body["tools"][0]["function"]["name"], "marks__list");
            let call = &wire.body["messages"][2]["tool_calls"][0];
            assert_eq!(call["function"]["name"], "marks__list");
            assert_eq!(call["function"]["arguments"], "{\"severity\":\"error\"}");
            let tool = &wire.body["messages"][3];
            assert_eq!(tool["role"], "tool");
            assert_eq!(tool["tool_call_id"], "call_1");
            assert_eq!(tool.get("name").is_some(), id == "mistral", "{id}");
        }
    }

    #[test]
    fn usage_can_be_left_out() {
        let provider = crate::config::default_providers().remove(0);
        let base = Url::parse(&provider.endpoint).unwrap();
        let request = ModelRequest {
            system: "S".into(),
            messages: vec![Msg::User("x".into())],
            tools: vec![],
        };
        let wire = build(
            &provider,
            &base,
            &request,
            Quirks {
                stream_usage: false,
            },
        )
        .unwrap();
        assert!(wire.body.get("stream_options").is_none());
        assert!(wire.body.get("tools").is_none());
    }
}
