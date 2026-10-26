//! Google Gemini (`models/{model}:streamGenerateContent?alt=sse`).
//!
//! Request: `x-goog-api-key`, `systemInstruction`, `contents` (roles `user`
//! and `model`, made of parts), `tools[].functionDeclarations`,
//! `generationConfig.maxOutputTokens`. Response: one `GenerateContentResponse`
//! per event with `candidates[0].content.parts` (`text`, `functionCall`,
//! thought parts and `thoughtSignature`), `finishReason` and `usageMetadata`.
//!
//! The model's parts are kept as received and sent back unchanged on the next
//! request, so thought signatures survive a tool round.

use serde_json::{Value, json};
use url::Url;

use super::{
    AssistantMsg, Auth, Completion, Decoder, Fault, Finish, ModelRequest, Msg, Piece, ToolCall,
    ToolResultMsg, Usage, Wire, from_wire_name, join, sse::SseEvent, wire_name,
};
use crate::{config::ProviderConfig, error::Result};

/// Ids this adapter makes up for calls that came without one.
const GENERATED: &str = "gcall_";

pub(super) fn build(provider: &ProviderConfig, base: &Url, request: &ModelRequest) -> Result<Wire> {
    let path = format!("v1beta/models/{}:streamGenerateContent", provider.model);
    let mut url = join(base, &path)?;
    url.query_pairs_mut().append_pair("alt", "sse");
    let mut body = json!({
        "systemInstruction": { "parts": [{ "text": request.system }] },
        "contents": contents(&request.messages),
        "generationConfig": { "maxOutputTokens": provider.max_output_tokens },
    });
    if !request.tools.is_empty() {
        body["tools"] = json!([{
            "functionDeclarations": request.tools.iter().map(|tool| json!({
                "name": wire_name(&tool.name),
                "description": tool.description,
                "parameters": tool.schema,
            })).collect::<Vec<_>>()
        }]);
    }
    Ok(Wire {
        url,
        auth: Auth::Header("x-goog-api-key"),
        headers: vec![("content-type".into(), "application/json".into())],
        body,
    })
}

fn contents(history: &[Msg]) -> Vec<Value> {
    let mut out = Vec::new();
    for message in history {
        match message {
            Msg::User(text) => out.push(json!({ "role": "user", "parts": [{ "text": text }] })),
            Msg::Assistant(turn) => {
                out.push(json!({ "role": "model", "parts": model_parts(turn) }))
            }
            Msg::ToolResults(results) => out.push(json!({
                "role": "user",
                "parts": results.iter().map(function_response).collect::<Vec<_>>(),
            })),
        }
    }
    out
}

fn model_parts(turn: &AssistantMsg) -> Value {
    if let Some(raw) = &turn.raw {
        return raw.clone();
    }
    let mut parts = Vec::new();
    if !turn.text.is_empty() {
        parts.push(json!({ "text": turn.text }));
    }
    for call in &turn.calls {
        parts.push(json!({
            "functionCall": {
                "name": wire_name(&call.name),
                "args": if call.input.is_null() { json!({}) } else { call.input.clone() },
            }
        }));
    }
    if parts.is_empty() {
        parts.push(json!({ "text": "(no text)" }));
    }
    Value::Array(parts)
}

fn function_response(result: &ToolResultMsg) -> Value {
    let mut response = json!({
        "name": wire_name(&result.name),
        "response": if result.is_error {
            json!({ "error": result.content })
        } else {
            json!({ "result": result.content })
        },
    });
    if !result.call_id.starts_with(GENERATED) {
        response["id"] = json!(result.call_id);
    }
    json!({ "functionResponse": response })
}

/// Decodes the Gemini stream.
#[derive(Default)]
pub struct GeminiDecoder {
    parts: Vec<Value>,
    text: String,
    calls: Vec<ToolCall>,
    finish: Option<String>,
    blocked: bool,
    usage: Option<Usage>,
    chunks: usize,
}

impl Decoder for GeminiDecoder {
    fn event(&mut self, event: &SseEvent, out: &mut Vec<Piece>) -> std::result::Result<(), Fault> {
        let data = event.data.trim();
        if data.is_empty() {
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
                    .get("status")
                    .and_then(Value::as_str)
                    .map(str::to_owned),
            });
        }
        self.chunks += 1;
        if chunk
            .pointer("/promptFeedback/blockReason")
            .and_then(Value::as_str)
            .is_some()
        {
            self.blocked = true;
        }
        if let Some(meta) = chunk.get("usageMetadata") {
            let input = meta
                .get("promptTokenCount")
                .and_then(Value::as_u64)
                .unwrap_or(0)
                + meta
                    .get("toolUsePromptTokenCount")
                    .and_then(Value::as_u64)
                    .unwrap_or(0);
            let output = meta
                .get("candidatesTokenCount")
                .and_then(Value::as_u64)
                .unwrap_or(0)
                + meta
                    .get("thoughtsTokenCount")
                    .and_then(Value::as_u64)
                    .unwrap_or(0);
            self.usage = Some(Usage { input, output });
        }
        let Some(candidate) = chunk.pointer("/candidates/0") else {
            return Ok(());
        };
        if let Some(reason) = candidate.get("finishReason").and_then(Value::as_str) {
            self.finish = Some(reason.to_owned());
        }
        let Some(parts) = candidate
            .pointer("/content/parts")
            .and_then(Value::as_array)
        else {
            return Ok(());
        };
        for part in parts {
            let thought = part
                .get("thought")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            if let Some(call) = part.get("functionCall") {
                let name = call
                    .get("name")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_owned();
                let id = call
                    .get("id")
                    .and_then(Value::as_str)
                    .map(str::to_owned)
                    .unwrap_or_else(|| format!("{GENERATED}{}", self.calls.len()));
                let args = call.get("args").cloned().unwrap_or_else(|| json!({}));
                let (input, error) = if args.is_object() {
                    (args, None)
                } else {
                    (
                        Value::Null,
                        Some("The tool arguments were not a JSON object".into()),
                    )
                };
                self.calls.push(ToolCall {
                    id,
                    name: from_wire_name(&name),
                    input,
                    error,
                });
                self.parts.push(part.clone());
            } else if let Some(text) = part.get("text").and_then(Value::as_str) {
                if thought {
                    self.parts.push(part.clone());
                    continue;
                }
                if !text.is_empty() {
                    self.text.push_str(text);
                    out.push(Piece::Text(text.to_owned()));
                }
                let has_signature = part.get("thoughtSignature").is_some();
                match self.parts.last_mut() {
                    Some(last)
                        if !has_signature
                            && last.get("functionCall").is_none()
                            && last.get("thought").is_none()
                            && last.get("thoughtSignature").is_none()
                            && last.get("text").is_some() =>
                    {
                        let joined = format!(
                            "{}{}",
                            last.get("text").and_then(Value::as_str).unwrap_or_default(),
                            text
                        );
                        last["text"] = json!(joined);
                    }
                    _ => self.parts.push(part.clone()),
                }
            } else {
                self.parts.push(part.clone());
            }
        }
        Ok(())
    }

    fn finish(self: Box<Self>) -> std::result::Result<Completion, Fault> {
        if self.chunks == 0 {
            return Err(Fault::Malformed(
                "the stream ended before any answer".into(),
            ));
        }
        let finish = if !self.calls.is_empty() {
            Finish::ToolUse
        } else if self.blocked {
            Finish::Refusal
        } else {
            match self.finish.as_deref() {
                Some("MAX_TOKENS") => Finish::Length,
                Some("SAFETY" | "RECITATION" | "BLOCKLIST" | "PROHIBITED_CONTENT" | "SPII") => {
                    Finish::Refusal
                }
                Some("STOP") | None => Finish::Stop,
                Some(_) => Finish::Other,
            }
        };
        let raw = (!self.parts.is_empty()).then(|| Value::Array(self.parts.clone()));
        Ok(Completion {
            text: self.text,
            calls: self.calls,
            usage: self.usage,
            finish,
            raw,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::{ToolDef, sse::SseParser};

    pub(crate) const TOOL_STREAM: &str = "\
data: {\"candidates\":[{\"content\":{\"role\":\"model\",\"parts\":[{\"text\":\"Checking. \"}]},\"index\":0}],\"usageMetadata\":{\"promptTokenCount\":120,\"totalTokenCount\":120},\"modelVersion\":\"gemini-3.8-flash\"}

data: {\"candidates\":[{\"content\":{\"role\":\"model\",\"parts\":[{\"functionCall\":{\"name\":\"checks__run\",\"args\":{}},\"thoughtSignature\":\"sig-abc\"}]},\"finishReason\":\"STOP\",\"index\":0}],\"usageMetadata\":{\"promptTokenCount\":120,\"candidatesTokenCount\":12,\"thoughtsTokenCount\":30,\"totalTokenCount\":162}}

";

    fn run(stream: &str) -> (String, Completion) {
        let mut parser = SseParser::new();
        let mut decoder: Box<dyn Decoder> = Box::new(GeminiDecoder::default());
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
    fn a_function_call_with_a_thought_signature_is_kept() {
        let (streamed, done) = run(TOOL_STREAM);
        assert_eq!(streamed, "Checking. ");
        assert_eq!(done.finish, Finish::ToolUse);
        assert_eq!(done.calls[0].name, "checks.run");
        assert!(done.calls[0].id.starts_with(GENERATED));
        assert_eq!(
            done.usage,
            Some(Usage {
                input: 120,
                output: 42
            })
        );
        let raw = done.raw.unwrap();
        assert_eq!(raw[0]["text"], "Checking. ");
        assert_eq!(raw[1]["thoughtSignature"], "sig-abc");
    }

    #[test]
    fn an_error_object_is_a_fault() {
        let mut decoder = GeminiDecoder::default();
        let event = SseEvent {
            event: None,
            data: "{\"error\":{\"code\":429,\"message\":\"Quota exceeded\",\"status\":\"RESOURCE_EXHAUSTED\"}}".into(),
        };
        assert_eq!(
            decoder.event(&event, &mut Vec::new()).unwrap_err(),
            Fault::Api {
                status: 429,
                message: "Quota exceeded".into(),
                code: Some("RESOURCE_EXHAUSTED".into())
            }
        );
    }

    #[test]
    fn the_request_has_the_documented_shape() {
        let provider = crate::config::default_providers()
            .into_iter()
            .find(|p| p.id == "google")
            .unwrap();
        let base = Url::parse(&provider.endpoint).unwrap();
        let request = ModelRequest {
            system: "SYS".into(),
            messages: vec![
                Msg::User("hi".into()),
                Msg::Assistant(AssistantMsg {
                    text: String::new(),
                    calls: vec![ToolCall {
                        id: "gcall_0".into(),
                        name: "checks.run".into(),
                        input: json!({}),
                        error: None,
                    }],
                    raw: None,
                }),
                Msg::ToolResults(vec![ToolResultMsg {
                    call_id: "gcall_0".into(),
                    name: "checks.run".into(),
                    content: "3 marks".into(),
                    is_error: false,
                }]),
            ],
            tools: vec![ToolDef {
                name: "checks.run".into(),
                description: "d".into(),
                schema: json!({"type":"object","properties":{}}),
            }],
        };
        let wire = build(&provider, &base, &request).unwrap();
        assert_eq!(
            wire.url.as_str(),
            "https://generativelanguage.googleapis.com/v1beta/models/gemini-3.8-flash:streamGenerateContent?alt=sse"
        );
        assert_eq!(wire.auth, Auth::Header("x-goog-api-key"));
        assert!(!wire.url.as_str().contains("key="));
        assert_eq!(wire.body["systemInstruction"]["parts"][0]["text"], "SYS");
        assert_eq!(
            wire.body["tools"][0]["functionDeclarations"][0]["name"],
            "checks__run"
        );
        assert_eq!(wire.body["contents"][1]["role"], "model");
        assert_eq!(
            wire.body["contents"][1]["parts"][0]["functionCall"]["name"],
            "checks__run"
        );
        let response = &wire.body["contents"][2]["parts"][0]["functionResponse"];
        assert_eq!(response["name"], "checks__run");
        assert_eq!(response["response"]["result"], "3 marks");
        assert!(response.get("id").is_none(), "made-up ids are not sent");
    }
}
