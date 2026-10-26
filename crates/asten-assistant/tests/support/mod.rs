//! A mock provider on loopback that speaks each provider's real wire format
//! (SSE streams, tool-call deltas, error bodies, rate-limit headers), and a
//! fixture project with a real continuity mark.

#![allow(dead_code)]

use std::{
    collections::{BTreeMap, VecDeque},
    convert::Infallible,
    net::SocketAddr,
    sync::{Arc, Mutex},
    time::Duration,
};

use asten_application::{Application, AuthMode};
use asten_assistant::{
    config::{ConfigStore, Consent, NOTICE_VERSION},
    keys::{KeyStore, MemoryKeyStore},
    ledger::Ledger,
    provider::transport::Transport,
    runner::{Assistant, Event, EventSink, RunRequest},
    secret::Secret,
};
use asten_domain::ProjectId;
use axum::{
    Router,
    body::{Body, Bytes},
    extract::{Request, State},
    http::{HeaderValue, StatusCode},
    response::Response,
};
use chrono::Utc;
use futures::StreamExt;
use serde_json::{Value, json};

/// What the mock saw.
#[derive(Clone, Debug)]
pub struct Seen {
    pub path: String,
    pub query: String,
    pub headers: BTreeMap<String, String>,
    pub raw: Vec<u8>,
    pub body: Value,
}

/// What the mock answers with, in order.
#[derive(Clone, Debug)]
pub enum Reply {
    Sse(String),
    Status {
        code: u16,
        headers: Vec<(&'static str, String)>,
        body: String,
    },
    Hang,
}

struct MockState {
    script: Mutex<VecDeque<Reply>>,
    seen: Mutex<Vec<Seen>>,
}

pub struct Mock {
    pub base: String,
    state: Arc<MockState>,
}

impl Mock {
    pub async fn start(script: Vec<Reply>) -> Self {
        let state = Arc::new(MockState {
            script: Mutex::new(script.into()),
            seen: Mutex::new(Vec::new()),
        });
        let app = Router::new().fallback(handle).with_state(state.clone());
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr: SocketAddr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        Self {
            base: format!("http://{addr}"),
            state,
        }
    }

    pub fn seen(&self) -> Vec<Seen> {
        self.state.seen.lock().unwrap().clone()
    }

    pub fn count(&self) -> usize {
        self.state.seen.lock().unwrap().len()
    }
}

async fn handle(State(state): State<Arc<MockState>>, request: Request) -> Response {
    let (parts, body) = request.into_parts();
    let raw = axum::body::to_bytes(body, 8 * 1024 * 1024)
        .await
        .unwrap()
        .to_vec();
    let headers = parts
        .headers
        .iter()
        .map(|(k, v)| (k.as_str().to_owned(), v.to_str().unwrap_or("").to_owned()))
        .collect();
    state.seen.lock().unwrap().push(Seen {
        path: parts.uri.path().to_owned(),
        query: parts.uri.query().unwrap_or("").to_owned(),
        headers,
        body: serde_json::from_slice(&raw).unwrap_or(Value::Null),
        raw,
    });
    let reply = state.script.lock().unwrap().pop_front();
    match reply {
        Some(Reply::Sse(text)) => {
            let chunks: Vec<String> = text.split_inclusive("\n\n").map(str::to_owned).collect();
            let stream = futures::stream::iter(chunks).then(|chunk| async move {
                tokio::time::sleep(Duration::from_millis(1)).await;
                Ok::<_, Infallible>(Bytes::from(chunk))
            });
            Response::builder()
                .status(200)
                .header("content-type", "text/event-stream")
                .body(Body::from_stream(stream))
                .unwrap()
        }
        Some(Reply::Status {
            code,
            headers,
            body,
        }) => {
            let mut response = Response::builder()
                .status(StatusCode::from_u16(code).unwrap())
                .header("content-type", "application/json");
            for (name, value) in headers {
                response = response.header(name, HeaderValue::from_str(&value).unwrap());
            }
            response.body(Body::from(body)).unwrap()
        }
        Some(Reply::Hang) => {
            tokio::time::sleep(Duration::from_secs(3600)).await;
            Response::new(Body::empty())
        }
        None => Response::builder()
            .status(500)
            .body(Body::from(
                "{\"error\":{\"message\":\"the mock has no more replies\"}}",
            ))
            .unwrap(),
    }
}

/// Collects events.
#[derive(Default)]
pub struct Collector(pub Mutex<Vec<Event>>);

impl EventSink for Collector {
    fn send(&self, event: Event) {
        self.0.lock().unwrap().push(event);
    }
}

impl Collector {
    pub fn events(&self) -> Vec<Event> {
        self.0.lock().unwrap().clone()
    }

    pub fn json(&self) -> String {
        serde_json::to_string(&self.events()).unwrap()
    }

    pub fn text(&self) -> String {
        self.events()
            .into_iter()
            .filter_map(|e| match e {
                Event::Text { delta } => Some(delta),
                _ => None,
            })
            .collect()
    }

    pub fn final_text(&self) -> Option<(String, bool)> {
        self.events().into_iter().find_map(|e| match e {
            Event::Final { text, withheld } => Some((text, withheld)),
            _ => None,
        })
    }

    pub fn tool_calls(&self) -> Vec<(String, bool, Option<String>)> {
        self.events()
            .into_iter()
            .filter_map(|e| match e {
                Event::ToolCall {
                    tool,
                    allowed,
                    reason,
                    ..
                } => Some((tool, allowed, reason)),
                _ => None,
            })
            .collect()
    }

    pub fn requests(&self) -> usize {
        self.events()
            .iter()
            .filter(|e| matches!(e, Event::Request { .. }))
            .count()
    }
}

/// A project with a real mark, an assistant wired to temp files and memory keys.
pub struct Fixture {
    pub assistant: Assistant,
    pub project: ProjectId,
    pub dir: tempfile::TempDir,
    pub canary: String,
    pub keys: Arc<MemoryKeyStore>,
    pub app: Application,
    pub scene_one: asten_domain::DocumentId,
    pub scene_two: asten_domain::DocumentId,
    pub note: asten_domain::DocumentId,
}

pub const MANUSCRIPT_CANARY: &str = "MANUSCRIPT-CANARY-7731";

fn paragraph(text: &str) -> Value {
    json!({"type":"paragraph","attrs":{"id":uuid_like()},"content":[{"type":"text","text":text}]})
}

fn uuid_like() -> String {
    uuid::Uuid::now_v7().to_string()
}

impl Fixture {
    pub async fn new() -> Self {
        let repository = asten_storage::SqliteRepository::in_memory().await.unwrap();
        let app = Application::with_auth(Arc::new(repository), AuthMode::Off);
        let project = app
            .create_project(
                serde_json::from_value(json!({"name":"The Harbor","kind":"novel"})).unwrap(),
            )
            .await
            .unwrap();
        let pid = project.id;
        app.create_entity(
            serde_json::from_value(json!({
                "project_id": pid, "kind": "character", "name": "Mira",
                "aliases": ["the keeper"], "notes": "Apprentice keeper at [[Gull Point]]."
            }))
            .unwrap(),
        )
        .await
        .unwrap();
        app.create_fact(
            serde_json::from_value(json!({
                "project_id": pid, "entity_key": "mira", "predicate": "hair_color",
                "value": "black", "status": "confirmed"
            }))
            .unwrap(),
        )
        .await
        .unwrap();
        app.create_fact(
            serde_json::from_value(json!({
                "project_id": pid, "entity_key": "mira", "predicate": "eye_color",
                "value": "grey", "status": "proposed"
            }))
            .unwrap(),
        )
        .await
        .unwrap();
        let mut scenes = Vec::new();
        for (title, body) in [
            (
                "Arrival",
                format!(
                    "Mira pushed her black hair back and looked at the sea. {MANUSCRIPT_CANARY}"
                ),
            ),
            (
                "The Letter",
                "Mira shook out her red hair and read the letter.".to_owned(),
            ),
        ] {
            let doc = app
                .create_document(
                    serde_json::from_value(json!({
                        "project_id": pid, "kind": "scene", "title": title,
                        "content_json": {"type":"doc","content":[paragraph(&body)]}
                    }))
                    .unwrap(),
                )
                .await
                .unwrap();
            scenes.push(doc.id);
        }
        let note = app
            .create_document(
                serde_json::from_value(json!({
                    "project_id": pid, "kind": "source", "title": "Tide tables",
                    "content_json": {"type":"doc","content":[
                        paragraph("The harbour empties twice a day. See [[Gull Point]]."),
                        paragraph("Spring tides run highest near the new moon.")
                    ]}
                }))
                .unwrap(),
            )
            .await
            .unwrap();
        app.create_document(
            serde_json::from_value(json!({
                "project_id": pid, "kind": "wiki", "title": "Gull Point",
                "content_json": {"type":"doc","content":[paragraph("The lamp stands on the headland. See [[Tide tables]].")]}
            }))
            .unwrap(),
        )
        .await
        .unwrap();

        let dir = tempfile::tempdir().unwrap();
        let canary = format!("ASTEN-CANARY-{:08x}-KEY", rand::random::<u32>());
        let keys = Arc::new(MemoryKeyStore::default());
        let config = Arc::new(ConfigStore::open(&dir.path().join("assistant.json")));
        let ledger = Arc::new(Ledger::open(&dir.path().join("ledger.jsonl")));
        let transport =
            Transport::with_limits(Duration::from_millis(5), Duration::from_secs(5), 3).unwrap();
        let assistant = Assistant::new(
            config,
            keys.clone() as Arc<dyn KeyStore>,
            ledger,
            transport,
            app.clone(),
        );
        Self {
            assistant,
            project: pid,
            dir,
            canary,
            keys,
            app,
            scene_one: scenes[0],
            scene_two: scenes[1],
            note: note.id,
        }
    }

    /// Switch a provider on, with consent, a key (cloud) and the mock as its endpoint.
    pub fn enable(&self, id: &str, base: &str) {
        let canary = self.canary.clone();
        let base = base.to_owned();
        let id = id.to_owned();
        self.assistant
            .config
            .update(|c| {
                c.enabled = true;
                let p = c.providers.iter_mut().find(|p| p.id == id).unwrap();
                p.endpoint = match p.family.as_str() {
                    "anthropic" | "gemini" => base.clone(),
                    "deepseek" => base.clone(),
                    _ => format!("{base}/v1"),
                };
                p.enabled = true;
                p.consent = Some(Consent {
                    accepted_at: Utc::now(),
                    host: "127.0.0.1".into(),
                    notice_version: NOTICE_VERSION,
                });
                if p.kind == asten_assistant::config::Kind::Cloud {
                    p.key_last4 = Some("-KEY".into());
                }
                Ok(())
            })
            .unwrap();
        if id != "ollama" {
            self.keys.set(&id, &Secret::new(canary)).unwrap();
        }
    }

    pub fn request(&self, provider: &str, text: &str) -> RunRequest {
        serde_json::from_value(json!({
            "run_id": format!("run-{}", rand::random::<u32>()),
            "thread_id": "thread-1",
            "provider": provider,
            "project_id": self.project.to_string(),
            "access": "propose",
            "text": text,
        }))
        .unwrap()
    }
}

/// Anthropic stream: a tool call.
pub fn anthropic_tool(id: &str, wire_name: &str, input: &str) -> String {
    let escaped = input.replace('\\', "\\\\").replace('"', "\\\"");
    format!(
        "event: message_start\ndata: {{\"type\":\"message_start\",\"message\":{{\"id\":\"msg_1\",\"type\":\"message\",\"role\":\"assistant\",\"content\":[],\"model\":\"claude-haiku-5-5\",\"stop_reason\":null,\"usage\":{{\"input_tokens\":300,\"output_tokens\":2}}}}}}\n\n\
event: content_block_start\ndata: {{\"type\":\"content_block_start\",\"index\":0,\"content_block\":{{\"type\":\"tool_use\",\"id\":\"{id}\",\"name\":\"{wire_name}\",\"input\":{{}}}}}}\n\n\
event: content_block_delta\ndata: {{\"type\":\"content_block_delta\",\"index\":0,\"delta\":{{\"type\":\"input_json_delta\",\"partial_json\":\"{escaped}\"}}}}\n\n\
event: content_block_stop\ndata: {{\"type\":\"content_block_stop\",\"index\":0}}\n\n\
event: message_delta\ndata: {{\"type\":\"message_delta\",\"delta\":{{\"stop_reason\":\"tool_use\"}},\"usage\":{{\"output_tokens\":40}}}}\n\n\
event: message_stop\ndata: {{\"type\":\"message_stop\"}}\n\n"
    )
}

/// Anthropic stream: plain text in two deltas.
pub fn anthropic_text(text: &str) -> String {
    let (a, b) = text.split_at(text.len() / 2);
    let a = serde_json::to_string(a).unwrap();
    let b = serde_json::to_string(b).unwrap();
    format!(
        "event: message_start\ndata: {{\"type\":\"message_start\",\"message\":{{\"id\":\"msg_2\",\"type\":\"message\",\"role\":\"assistant\",\"content\":[],\"usage\":{{\"input_tokens\":500,\"output_tokens\":1}}}}}}\n\n\
event: content_block_start\ndata: {{\"type\":\"content_block_start\",\"index\":0,\"content_block\":{{\"type\":\"text\",\"text\":\"\"}}}}\n\n\
event: content_block_delta\ndata: {{\"type\":\"content_block_delta\",\"index\":0,\"delta\":{{\"type\":\"text_delta\",\"text\":{a}}}}}\n\n\
event: content_block_delta\ndata: {{\"type\":\"content_block_delta\",\"index\":0,\"delta\":{{\"type\":\"text_delta\",\"text\":{b}}}}}\n\n\
event: content_block_stop\ndata: {{\"type\":\"content_block_stop\",\"index\":0}}\n\n\
event: message_delta\ndata: {{\"type\":\"message_delta\",\"delta\":{{\"stop_reason\":\"end_turn\"}},\"usage\":{{\"output_tokens\":60}}}}\n\n\
event: message_stop\ndata: {{\"type\":\"message_stop\"}}\n\n"
    )
}

/// OpenAI-style stream: a tool call.
pub fn openai_tool(id: &str, wire_name: &str, arguments: &str) -> String {
    let args = serde_json::to_string(arguments).unwrap();
    format!(
        "data: {{\"id\":\"c1\",\"choices\":[{{\"index\":0,\"delta\":{{\"role\":\"assistant\",\"content\":null,\"tool_calls\":[{{\"index\":0,\"id\":\"{id}\",\"type\":\"function\",\"function\":{{\"name\":\"{wire_name}\",\"arguments\":\"\"}}}}]}}}}]}}\n\n\
data: {{\"id\":\"c1\",\"choices\":[{{\"index\":0,\"delta\":{{\"tool_calls\":[{{\"index\":0,\"function\":{{\"arguments\":{args}}}}}]}}}}]}}\n\n\
data: {{\"id\":\"c1\",\"choices\":[{{\"index\":0,\"delta\":{{}},\"finish_reason\":\"tool_calls\"}}]}}\n\n\
data: {{\"id\":\"c1\",\"choices\":[],\"usage\":{{\"prompt_tokens\":300,\"completion_tokens\":40,\"total_tokens\":340}}}}\n\n\
data: [DONE]\n\n"
    )
}

/// OpenAI-style stream: text.
pub fn openai_text(text: &str) -> String {
    let (a, b) = text.split_at(text.len() / 2);
    let a = serde_json::to_string(a).unwrap();
    let b = serde_json::to_string(b).unwrap();
    format!(
        "data: {{\"id\":\"c2\",\"choices\":[{{\"index\":0,\"delta\":{{\"role\":\"assistant\",\"content\":{a}}}}}]}}\n\n\
data: {{\"id\":\"c2\",\"choices\":[{{\"index\":0,\"delta\":{{\"content\":{b}}}}}]}}\n\n\
data: {{\"id\":\"c2\",\"choices\":[{{\"index\":0,\"delta\":{{}},\"finish_reason\":\"stop\"}}]}}\n\n\
data: {{\"id\":\"c2\",\"choices\":[],\"usage\":{{\"prompt_tokens\":500,\"completion_tokens\":60,\"total_tokens\":560}}}}\n\n\
data: [DONE]\n\n"
    )
}

/// Gemini stream: a function call.
pub fn gemini_tool(wire_name: &str, args: &Value) -> String {
    format!(
        "data: {}\n\n",
        json!({
            "candidates":[{"content":{"role":"model","parts":[{"functionCall":{"name":wire_name,"args":args},"thoughtSignature":"sig-1"}]},"finishReason":"STOP","index":0}],
            "usageMetadata":{"promptTokenCount":300,"candidatesTokenCount":40,"totalTokenCount":340}
        })
    )
}

/// Gemini stream: text.
pub fn gemini_text(text: &str) -> String {
    let (a, b) = text.split_at(text.len() / 2);
    format!(
        "data: {}\n\ndata: {}\n\n",
        json!({"candidates":[{"content":{"role":"model","parts":[{"text":a}]},"index":0}]}),
        json!({
            "candidates":[{"content":{"role":"model","parts":[{"text":b}]},"finishReason":"STOP","index":0}],
            "usageMetadata":{"promptTokenCount":500,"candidatesTokenCount":60,"totalTokenCount":560}
        })
    )
}

pub const EXPLANATION: &str = "CANON-E013 marks a clash about Mira's hair colour: the canon settles black, and the scene The Letter says red. The rule offers repairs you can press on the card, including keeping your sentence.";
