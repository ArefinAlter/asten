//! Model Context Protocol (MCP) server for Asten.
//!
//! Provides a production-grade MCP interface exposing all three core primitives:
//! 1. Tools: Action layer (continuity marks, canon search, timeline, scene summary, check runs, fact proposals).
//! 2. Resources: Data layer (canon rule invariants catalog, cast entities, timeline ledger).
//! 3. Prompts: Intent layer (audit_scene_continuity, resolve_timeline_conflict, flesh_out_cast_canon).
//!
//! Adheres strictly to agent UX best practices:
//! - Curated, high-leverage surface area
//! - Intent-driven, model-optimized descriptions
//! - Strict snake_case tool names
//! - Graceful degraded responses with actionable recovery context
//! - Asymmetric privilege guardrails (human-in-the-loop proposals)

use std::sync::Arc;
use asten_application::{Application, CreateFact};
use asten_domain::{DocumentId, FactModality, FactStatus, ProjectId};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use uuid::Uuid;

const CANON_RULES_SCHEMA: &str = r#"{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "AstenCanonRuleCatalog",
  "description": "Catalog of Asten continuity and narrative canon verification families.",
  "type": "object",
  "properties": {
    "families": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "family": { "type": "string" },
          "title": { "type": "string" },
          "description": { "type": "string" },
          "rules": {
            "type": "array",
            "items": {
              "type": "object",
              "properties": {
                "code": { "type": "string" },
                "name": { "type": "string" },
                "severity": { "type": "string" }
              }
            }
          }
        }
      }
    }
  }
}"#;

#[derive(Debug, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    #[serde(default)]
    pub id: Option<Value>,
    pub method: String,
    #[serde(default)]
    pub params: Option<Value>,
}

#[derive(Debug, Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize)]
pub struct JsonRpcError {
    pub code: i64,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

pub struct McpServer {
    app: Arc<Application>,
}

impl McpServer {
    pub fn new(app: Arc<Application>) -> Self {
        Self { app }
    }

    /// Resolve project ID from params or fall back to the first available project.
    async fn resolve_project(&self, params: &Option<Value>) -> Result<ProjectId, String> {
        if let Some(p) = params {
            if let Some(id_str) = p.get("project_id").and_then(|v| v.as_str()) {
                if let Ok(parsed) = Uuid::parse_str(id_str) {
                    return Ok(ProjectId(parsed));
                }
            }
        }
        let projects = self.app.list_projects().await.map_err(|e| e.to_string())?;
        projects
            .into_iter()
            .next()
            .map(|p| p.id)
            .ok_or_else(|| "No active project found in Asten. Create or open a project first.".to_string())
    }

    pub async fn handle_request(&self, req: JsonRpcRequest) -> JsonRpcResponse {
        let id = req.id;
        match req.method.as_str() {
            "initialize" => JsonRpcResponse {
                jsonrpc: "2.0".into(),
                id,
                result: Some(json!({
                    "protocolVersion": "2024-11-05",
                    "serverInfo": {
                        "name": "asten-mcp",
                        "version": env!("CARGO_PKG_VERSION")
                    },
                    "capabilities": {
                        "tools": { "listChanged": false },
                        "resources": { "subscribe": false, "listChanged": false },
                        "prompts": { "listChanged": false }
                    }
                })),
                error: None,
            },

            "tools/list" => JsonRpcResponse {
                jsonrpc: "2.0".into(),
                id,
                result: Some(json!({
                    "tools": [
                        {
                            "name": "asten_marks_list",
                            "description": "Lists open narrative continuity marks, plot discrepancies, and invariant warnings across the manuscript.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "project_id": {
                                        "type": "string",
                                        "description": "Optional UUID of the project (uses the primary active project if omitted)"
                                    },
                                    "severity": {
                                        "type": "string",
                                        "enum": ["error", "warning", "information"],
                                        "description": "Filter marks by severity level"
                                    }
                                },
                                "required": []
                            }
                        },
                        {
                            "name": "asten_mark_explain",
                            "description": "Deep-dive analysis of a continuity mark: conflicting statements, evidence chain, settlement locations, and deterministic repairs.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "mark_id": {
                                        "type": "string",
                                        "description": "The diagnostic/mark ID to inspect and explain"
                                    },
                                    "project_id": {
                                        "type": "string",
                                        "description": "Optional UUID of the project"
                                    }
                                },
                                "required": ["mark_id"]
                            }
                        },
                        {
                            "name": "asten_canon_search",
                            "description": "Searches characters, cast members, locations, and narrative facts across settled and proposed canon.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "query": {
                                        "type": "string",
                                        "description": "Search term for character name, alias, location, or fact predicate"
                                    },
                                    "project_id": {
                                        "type": "string",
                                        "description": "Optional UUID of the project"
                                    }
                                },
                                "required": ["query"]
                            }
                        },
                        {
                            "name": "asten_timeline_list",
                            "description": "Reads the chronological story timeline events and narrative sequence in story time and reading order.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "project_id": {
                                        "type": "string",
                                        "description": "Optional UUID of the project"
                                    }
                                },
                                "required": []
                            }
                        },
                        {
                            "name": "asten_scene_summary",
                            "description": "Reads a scene's story time, narrative context, present characters, word count, and mark count without exposing manuscript prose.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "document_id": {
                                        "type": "string",
                                        "description": "The UUID of the document or scene"
                                    },
                                    "project_id": {
                                        "type": "string",
                                        "description": "Optional UUID of the project"
                                    }
                                },
                                "required": ["document_id"]
                            }
                        },
                        {
                            "name": "asten_checks_run",
                            "description": "Re-runs the narrative continuity and canon check rules across the manuscript, returning fresh diagnostics.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "project_id": {
                                        "type": "string",
                                        "description": "Optional UUID of the project"
                                    }
                                },
                                "required": []
                            }
                        },
                        {
                            "name": "asten_fact_propose",
                            "description": "Proposes a new narrative fact for character, setting, or plot canon (prepared for author confirmation).",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "entity": {
                                        "type": "string",
                                        "description": "The name or key of the character or entity"
                                    },
                                    "predicate": {
                                        "type": "string",
                                        "description": "The attribute or fact predicate (e.g. eye_color, age, location)"
                                    },
                                    "value": {
                                        "type": "string",
                                        "description": "The proposed value of the fact"
                                    },
                                    "project_id": {
                                        "type": "string",
                                        "description": "Optional UUID of the project"
                                    }
                                },
                                "required": ["entity", "predicate", "value"]
                            }
                        }
                    ]
                })),
                error: None,
            },

            "tools/call" => {
                let params = req.params.unwrap_or(Value::Null);
                let tool_name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
                let arguments = params.get("arguments").cloned().unwrap_or(Value::Null);

                let result = match tool_name {
                    "asten_marks_list" => self.tool_marks_list(&arguments).await,
                    "asten_mark_explain" => self.tool_mark_explain(&arguments).await,
                    "asten_canon_search" => self.tool_canon_search(&arguments).await,
                    "asten_timeline_list" => self.tool_timeline_list(&arguments).await,
                    "asten_scene_summary" => self.tool_scene_summary(&arguments).await,
                    "asten_checks_run" => self.tool_checks_run(&arguments).await,
                    "asten_fact_propose" => self.tool_fact_propose(&arguments).await,
                    _ => Ok(json!({
                        "content": [{
                            "type": "text",
                            "text": format!("Unknown tool: '{tool_name}'. Available tools: asten_marks_list, asten_mark_explain, asten_canon_search, asten_timeline_list, asten_scene_summary, asten_checks_run, asten_fact_propose.")
                        }],
                        "isError": true
                    })),
                };

                match result {
                    Ok(val) => JsonRpcResponse {
                        jsonrpc: "2.0".into(),
                        id,
                        result: Some(val),
                        error: None,
                    },
                    Err(err) => JsonRpcResponse {
                        jsonrpc: "2.0".into(),
                        id,
                        result: Some(json!({
                            "content": [{ "type": "text", "text": format!("Operation failed: {err}") }],
                            "isError": true
                        })),
                        error: None,
                    },
                }
            }

            "resources/list" => JsonRpcResponse {
                jsonrpc: "2.0".into(),
                id,
                result: Some(json!({
                    "resources": [
                        {
                            "uri": "asten://schema/canon-rules",
                            "name": "Asten Canon Rule and Invariant Catalog",
                            "description": "The formal catalog of built-in canon verification families (chronology, geography, identity, perception, obligations).",
                            "mimeType": "application/json"
                        },
                        {
                            "uri": "asten://entities",
                            "name": "Asten Cast and Canon Entities",
                            "description": "Active registered cast members, characters, settings, and narrative entities in the active project.",
                            "mimeType": "application/json"
                        },
                        {
                            "uri": "asten://timeline",
                            "name": "Asten Story Timeline Events",
                            "description": "Chronological timeline event sequence and scene reading order in the active project.",
                            "mimeType": "application/json"
                        }
                    ]
                })),
                error: None,
            },

            "resources/read" => {
                let params = req.params.unwrap_or(Value::Null);
                let uri = params.get("uri").and_then(|v| v.as_str()).unwrap_or("");
                let result = match uri {
                    "asten://schema/canon-rules" => Ok(json!({
                        "contents": [{
                            "uri": "asten://schema/canon-rules",
                            "mimeType": "application/json",
                            "text": CANON_RULES_SCHEMA
                        }]
                    })),
                    "asten://entities" => match self.resolve_project(&Some(params.clone())).await {
                        Ok(project_id) => {
                            let entities = self.app.list_entities(project_id).await.unwrap_or_default();
                            Ok(json!({
                                "contents": [{
                                    "uri": "asten://entities",
                                    "mimeType": "application/json",
                                    "text": serde_json::to_string_pretty(&entities).unwrap_or_default()
                                }]
                            }))
                        }
                        Err(e) => Ok(json!({
                            "contents": [{
                                "uri": "asten://entities",
                                "mimeType": "application/json",
                                "text": json!({ "error": e, "entities": [] }).to_string()
                            }]
                        })),
                    },
                    "asten://timeline" => match self.resolve_project(&Some(params.clone())).await {
                        Ok(project_id) => {
                            let events = self.app.list_events(project_id).await.unwrap_or_default();
                            Ok(json!({
                                "contents": [{
                                    "uri": "asten://timeline",
                                    "mimeType": "application/json",
                                    "text": serde_json::to_string_pretty(&events).unwrap_or_default()
                                }]
                            }))
                        }
                        Err(e) => Ok(json!({
                            "contents": [{
                                "uri": "asten://timeline",
                                "mimeType": "application/json",
                                "text": json!({ "error": e, "events": [] }).to_string()
                            }]
                        })),
                    },
                    _ => Err(format!("Resource not found: {uri}")),
                };

                match result {
                    Ok(val) => JsonRpcResponse {
                        jsonrpc: "2.0".into(),
                        id,
                        result: Some(val),
                        error: None,
                    },
                    Err(msg) => JsonRpcResponse {
                        jsonrpc: "2.0".into(),
                        id,
                        result: None,
                        error: Some(JsonRpcError {
                            code: -32602,
                            message: msg,
                            data: None,
                        }),
                    },
                }
            }

            "prompts/list" => JsonRpcResponse {
                jsonrpc: "2.0".into(),
                id,
                result: Some(json!({
                    "prompts": [
                        {
                            "name": "audit_scene_continuity",
                            "description": "Analyze a scene's narrative continuity, character presence, and timeline constraints against established canon.",
                            "arguments": [
                                {
                                    "name": "scene_name",
                                    "description": "Title or description of the scene to audit",
                                    "required": true
                                }
                            ]
                        },
                        {
                            "name": "resolve_timeline_conflict",
                            "description": "Resolve anachronisms, multi-location paradoxes, or chronological discrepancies between scenes.",
                            "arguments": [
                                {
                                    "name": "conflict_description",
                                    "description": "Detailed description of the timeline discrepancy",
                                    "required": true
                                }
                            ]
                        },
                        {
                            "name": "flesh_out_cast_canon",
                            "description": "Expand character background, physical traits, relationships, and facts without violating existing rules.",
                            "arguments": [
                                {
                                    "name": "character_name",
                                    "description": "Name of the character to flesh out",
                                    "required": true
                                }
                            ]
                        }
                    ]
                })),
                error: None,
            },

            "prompts/get" => {
                let params = req.params.unwrap_or(Value::Null);
                let prompt_name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
                let args = params.get("arguments").cloned().unwrap_or(Value::Null);

                let result = match prompt_name {
                    "audit_scene_continuity" => {
                        let scene = args.get("scene_name").and_then(|v| v.as_str()).unwrap_or("the scene");
                        Ok(json!({
                            "description": format!("Audit continuity for {scene}"),
                            "messages": [{
                                "role": "user",
                                "content": {
                                    "type": "text",
                                    "text": format!("Please perform a thorough continuity audit for '{scene}'.\n1. Call `asten_marks_list` to see any open continuity marks associated with this scene.\n2. Call `asten_canon_search` to verify character attributes, established knowledge, and locations.\n3. Call `asten_timeline_list` to check chronological order and avoid anachronisms.\n4. Recommend precise, deterministic adjustments.")
                                }
                            }]
                        }))
                    }
                    "resolve_timeline_conflict" => {
                        let conflict = args.get("conflict_description").and_then(|v| v.as_str()).unwrap_or("timeline conflict");
                        Ok(json!({
                            "description": "Resolve timeline discrepancy",
                            "messages": [{
                                "role": "user",
                                "content": {
                                    "type": "text",
                                    "text": format!("We have a timeline conflict: '{conflict}'.\n1. Inspect open marks using `asten_marks_list`.\n2. Call `asten_mark_explain` for any timeline marks to see the exact conflicting statements and settlement evidence.\n3. Formulate a clean repair proposal that keeps the narrative coherent.")
                                }
                            }]
                        }))
                    }
                    "flesh_out_cast_canon" => {
                        let char_name = args.get("character_name").and_then(|v| v.as_str()).unwrap_or("the character");
                        Ok(json!({
                            "description": format!("Flesh out canon for {char_name}"),
                            "messages": [{
                                "role": "user",
                                "content": {
                                    "type": "text",
                                    "text": format!("Let's expand the established canon for '{char_name}'.\n1. Search existing facts with `asten_canon_search` for query '{char_name}'.\n2. Propose new consistent character facts (e.g. background, relationships, traits) using `asten_fact_propose`.\n3. Ensure none of the proposed facts violate existing rules.")
                                }
                            }]
                        }))
                    }
                    _ => Err(format!("Prompt not found: {prompt_name}")),
                };

                match result {
                    Ok(val) => JsonRpcResponse {
                        jsonrpc: "2.0".into(),
                        id,
                        result: Some(val),
                        error: None,
                    },
                    Err(msg) => JsonRpcResponse {
                        jsonrpc: "2.0".into(),
                        id,
                        result: None,
                        error: Some(JsonRpcError {
                            code: -32602,
                            message: msg,
                            data: None,
                        }),
                    },
                }
            }

            _ => JsonRpcResponse {
                jsonrpc: "2.0".into(),
                id,
                result: None,
                error: Some(JsonRpcError {
                    code: -32601,
                    message: format!("Method not found: {}", req.method),
                    data: None,
                }),
            },
        }
    }

    async fn tool_marks_list(&self, args: &Value) -> Result<Value, String> {
        let project_id = self.resolve_project(&Some(args.clone())).await?;
        let snap = self.app.analyze_project(project_id).await.map_err(|e| e.to_string())?;
        let severity_filter = args.get("severity").and_then(|v| v.as_str());

        let marks: Vec<Value> = snap
            .diagnostics
            .into_iter()
            .filter(|d| {
                if let Some(sev) = severity_filter {
                    format!("{:?}", d.severity).eq_ignore_ascii_case(sev)
                } else {
                    true
                }
            })
            .map(|d| {
                json!({
                    "id": d.id,
                    "code": d.code,
                    "severity": format!("{:?}", d.severity),
                    "title": d.title,
                    "message": d.message,
                    "scene_document_id": d.primary.as_ref().map(|p| p.document.to_string()),
                    "has_repairs": !d.repairs.is_empty(),
                    "repair_count": d.repairs.len()
                })
            })
            .collect();

        Ok(json!({
            "content": [{
                "type": "text",
                "text": serde_json::to_string_pretty(&json!({
                    "project_id": project_id.to_string(),
                    "total_marks": marks.len(),
                    "marks": marks
                })).unwrap_or_default()
            }]
        }))
    }

    async fn tool_mark_explain(&self, args: &Value) -> Result<Value, String> {
        let mark_id = args.get("mark_id").and_then(|v| v.as_str()).ok_or("mark_id is required")?;
        let project_id = self.resolve_project(&Some(args.clone())).await?;
        let snap = self.app.analyze_project(project_id).await.map_err(|e| e.to_string())?;

        let diag = snap.diagnostics.iter().find(|d| d.id == mark_id);
        match diag {
            Some(d) => {
                let repairs: Vec<Value> = d.repairs.iter().map(|r| {
                    json!({
                        "kind": format!("{:?}", r.kind),
                        "label": r.label,
                        "cost": r.cost,
                    })
                }).collect();

                Ok(json!({
                    "content": [{
                        "type": "text",
                        "text": serde_json::to_string_pretty(&json!({
                            "mark_id": d.id,
                            "code": d.code,
                            "severity": format!("{:?}", d.severity),
                            "title": d.title,
                            "message": d.message,
                            "explanation": d.explanation,
                            "primary": d.primary,
                            "repairs": repairs
                        })).unwrap_or_default()
                    }]
                }))
            }
            None => Ok(json!({
                "content": [{
                    "type": "text",
                    "text": serde_json::to_string_pretty(&json!({
                        "status": "not_found",
                        "mark_id": mark_id,
                        "message": "No diagnostic mark matches the provided ID. It may have already been resolved or dismissed.",
                        "suggestion": "Call asten_marks_list to inspect active marks."
                    })).unwrap_or_default()
                }]
            })),
        }
    }

    async fn tool_canon_search(&self, args: &Value) -> Result<Value, String> {
        let query = args.get("query").and_then(|v| v.as_str()).unwrap_or("");
        let project_id = self.resolve_project(&Some(args.clone())).await?;

        let hits = self.app.cast_search(project_id, query).await.unwrap_or_default();
        let entities = self.app.list_entities(project_id).await.unwrap_or_default();
        let facts = self.app.list_facts(project_id).await.unwrap_or_default();

        let matching_entities: Vec<Value> = entities
            .into_iter()
            .filter(|e| {
                e.name.to_lowercase().contains(&query.to_lowercase())
                    || e.aliases.iter().any(|a| a.to_lowercase().contains(&query.to_lowercase()))
            })
            .map(|e| json!({
                "id": e.id.to_string(),
                "name": e.name,
                "aliases": e.aliases,
                "kind": format!("{:?}", e.kind),
                "notes": e.notes
            }))
            .collect();

        let matching_facts: Vec<Value> = facts
            .into_iter()
            .filter(|f| {
                f.entity_key.to_lowercase().contains(&query.to_lowercase())
                    || f.predicate.to_lowercase().contains(&query.to_lowercase())
                    || f.value.to_lowercase().contains(&query.to_lowercase())
            })
            .map(|f| json!({
                "id": f.id.to_string(),
                "entity_key": f.entity_key,
                "predicate": f.predicate,
                "value": f.value,
                "status": format!("{:?}", f.status)
            }))
            .collect();

        Ok(json!({
            "content": [{
                "type": "text",
                "text": serde_json::to_string_pretty(&json!({
                    "query": query,
                    "search_hits": hits,
                    "matching_entities": matching_entities,
                    "matching_facts": matching_facts
                })).unwrap_or_default()
            }]
        }))
    }

    async fn tool_timeline_list(&self, args: &Value) -> Result<Value, String> {
        let project_id = self.resolve_project(&Some(args.clone())).await?;
        let events = self.app.list_events(project_id).await.map_err(|e| e.to_string())?;

        let items: Vec<Value> = events
            .into_iter()
            .map(|ev| {
                json!({
                    "id": ev.id.to_string(),
                    "title": ev.title,
                    "story_time": ev.story_time,
                    "reading_order": ev.reading_order,
                    "duration": ev.duration,
                    "notes": ev.notes,
                    "participants": ev.participants,
                    "kind": format!("{:?}", ev.kind)
                })
            })
            .collect();

        Ok(json!({
            "content": [{
                "type": "text",
                "text": serde_json::to_string_pretty(&json!({
                    "project_id": project_id.to_string(),
                    "total_events": items.len(),
                    "events": items
                })).unwrap_or_default()
            }]
        }))
    }

    async fn tool_scene_summary(&self, args: &Value) -> Result<Value, String> {
        let doc_id_str = args.get("document_id").and_then(|v| v.as_str()).ok_or("document_id is required")?;
        let doc_uuid = Uuid::parse_str(doc_id_str).map_err(|_| "document_id must be a valid UUID".to_string())?;
        let project_id = self.resolve_project(&Some(args.clone())).await?;

        let summary = self
            .app
            .scene_summary(project_id, DocumentId(doc_uuid))
            .await
            .map_err(|e| e.to_string())?;

        Ok(json!({
            "content": [{
                "type": "text",
                "text": serde_json::to_string_pretty(&summary).unwrap_or_default()
            }]
        }))
    }

    async fn tool_checks_run(&self, args: &Value) -> Result<Value, String> {
        let project_id = self.resolve_project(&Some(args.clone())).await?;
        let snap = self.app.analyze_project(project_id).await.map_err(|e| e.to_string())?;

        Ok(json!({
            "content": [{
                "type": "text",
                "text": serde_json::to_string_pretty(&json!({
                    "project_id": project_id.to_string(),
                    "revision": snap.revision,
                    "marks_count": snap.diagnostics.len(),
                    "timings_ms": snap.timings_ms,
                    "cached_blocks": snap.cached_blocks,
                    "total_blocks": snap.total_blocks
                })).unwrap_or_default()
            }]
        }))
    }

    async fn tool_fact_propose(&self, args: &Value) -> Result<Value, String> {
        let entity = args.get("entity").and_then(|v| v.as_str()).ok_or("entity is required")?;
        let predicate = args.get("predicate").and_then(|v| v.as_str()).ok_or("predicate is required")?;
        let value = args.get("value").and_then(|v| v.as_str()).ok_or("value is required")?;
        let project_id = self.resolve_project(&Some(args.clone())).await?;

        let fact = self
            .app
            .create_fact(CreateFact {
                project_id,
                entity_key: entity.to_string(),
                predicate: predicate.to_string(),
                value: value.to_string(),
                status: FactStatus::Proposed,
                modality: FactModality::default(),
                valid_from: None,
                valid_to: None,
                source: None,
                context: None,
                anchor: None,
                confidence: Some(1.0),
                provenance: None,
            })
            .await
            .map_err(|e| e.to_string())?;

        Ok(json!({
            "content": [{
                "type": "text",
                "text": serde_json::to_string_pretty(&json!({
                    "status": "proposed",
                    "fact_id": fact.id.to_string(),
                    "entity": entity,
                    "entity_key": fact.entity_key,
                    "predicate": fact.predicate,
                    "value": fact.value,
                    "note": "Fact successfully registered in Proposed state. Ready for author acceptance card in the desk."
                })).unwrap_or_default()
            }]
        }))
    }

    /// Run stdio JSON-RPC loop for local coding agents.
    pub async fn run_stdio(self: Arc<Self>) -> anyhow::Result<()> {
        let stdin = tokio::io::stdin();
        let mut reader = BufReader::new(stdin).lines();
        let mut stdout = tokio::io::stdout();

        while let Some(line) = reader.next_line().await? {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            let req: JsonRpcRequest = match serde_json::from_str(trimmed) {
                Ok(r) => r,
                Err(e) => {
                    let err_resp = JsonRpcResponse {
                        jsonrpc: "2.0".into(),
                        id: None,
                        result: None,
                        error: Some(JsonRpcError {
                            code: -32700,
                            message: format!("Parse error: {e}"),
                            data: None,
                        }),
                    };
                    let bytes = serde_json::to_vec(&err_resp)?;
                    stdout.write_all(&bytes).await?;
                    stdout.write_all(b"\n").await?;
                    stdout.flush().await?;
                    continue;
                }
            };

            let resp = self.handle_request(req).await;
            let bytes = serde_json::to_vec(&resp)?;
            stdout.write_all(&bytes).await?;
            stdout.write_all(b"\n").await?;
            stdout.flush().await?;
        }
        Ok(())
    }
}

pub fn router() -> axum::Router<crate::ApiState> {
    axum::Router::new()
        .route("/mcp", axum::routing::post(handle_http))
        .route("/api/v1/mcp", axum::routing::post(handle_http))
}

pub async fn handle_http(
    axum::extract::State(state): axum::extract::State<crate::ApiState>,
    axum::Json(payload): axum::Json<JsonRpcRequest>,
) -> axum::Json<JsonRpcResponse> {
    let server = McpServer::new(Arc::new(state.application.clone()));
    let resp = server.handle_request(payload).await;
    axum::Json(resp)
}
