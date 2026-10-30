use asten_api::mcp::{JsonRpcRequest, McpServer};
use asten_application::CreateProject;
use asten_domain::ProjectKind;
use serde_json::json;
use std::path::PathBuf;
use std::sync::Arc;
use uuid::Uuid;

struct TestContext {
    dir: PathBuf,
}

impl Drop for TestContext {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

async fn setup_app() -> (TestContext, asten_application::Application) {
    let id = Uuid::now_v7();
    let dir = std::env::temp_dir().join(format!("asten-mcp-test-{id}"));
    std::fs::create_dir_all(&dir).unwrap();
    let db_path = dir.join("asten.db").to_string_lossy().to_string();
    let app = asten_api::application_for_database(&db_path, asten_application::AuthMode::Off)
        .await
        .unwrap();
    (TestContext { dir }, app)
}

#[tokio::test]
async fn mcp_initialize_and_capabilities() {
    let (_ctx, app) = setup_app().await;
    let server = McpServer::new(Arc::new(app));

    let init_req = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: Some(json!(1)),
        method: "initialize".into(),
        params: None,
    };
    let resp = server.handle_request(init_req).await;
    assert_eq!(resp.jsonrpc, "2.0");
    assert_eq!(resp.id, Some(json!(1)));

    let result = resp.result.expect("result expected");
    assert_eq!(result["serverInfo"]["name"], "asten-mcp");
    assert!(result["capabilities"]["tools"].is_object());
    assert!(result["capabilities"]["resources"].is_object());
    assert!(result["capabilities"]["prompts"].is_object());
}

#[tokio::test]
async fn mcp_tools_list_and_call() {
    let (_ctx, app) = setup_app().await;

    let project = app
        .create_project(CreateProject {
            name: "The Starlight Chronicles".into(),
            kind: ProjectKind::Novel,
            description: Some("An interstellar mystery".into()),
        })
        .await
        .unwrap();

    let server = McpServer::new(Arc::new(app.clone()));

    // List tools
    let list_req = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: Some(json!(2)),
        method: "tools/list".into(),
        params: None,
    };
    let resp = server.handle_request(list_req).await;
    let tools = resp.result.unwrap()["tools"].as_array().unwrap().clone();
    let tool_names: Vec<&str> = tools.iter().filter_map(|t| t["name"].as_str()).collect();

    assert!(tool_names.contains(&"asten_marks_list"));
    assert!(tool_names.contains(&"asten_mark_explain"));
    assert!(tool_names.contains(&"asten_canon_search"));
    assert!(tool_names.contains(&"asten_timeline_list"));
    assert!(tool_names.contains(&"asten_scene_summary"));
    assert!(tool_names.contains(&"asten_checks_run"));
    assert!(!tool_names.contains(&"asten_fact_propose"));

    // Call asten_canon_search
    let search_req = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: Some(json!(3)),
        method: "tools/call".into(),
        params: Some(json!({
            "name": "asten_canon_search",
            "arguments": {
                "project_id": project.id.to_string(),
                "query": "Starlight"
            }
        })),
    };
    let search_resp = server.handle_request(search_req).await;
    let content = &search_resp.result.unwrap()["content"][0]["text"];
    assert!(content.as_str().unwrap().contains("search_hits"));

    // An agent cannot persist facts, even if it calls the retired tool directly.
    let propose_req = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: Some(json!(4)),
        method: "tools/call".into(),
        params: Some(json!({"name":"asten_fact_propose", "arguments":{
            "project_id":project.id.to_string(), "entity":"Commander Hayes", "predicate":"rank", "value":"Captain"
        }})),
    };
    let response = server.handle_request(propose_req).await;
    assert_eq!(response.error.unwrap().code, -32602);
    assert!(app.list_facts(project.id).await.unwrap().is_empty());

    // Call asten_marks_list
    let marks_req = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: Some(json!(5)),
        method: "tools/call".into(),
        params: Some(json!({
            "name": "asten_marks_list",
            "arguments": {
                "project_id": project.id.to_string()
            }
        })),
    };
    let marks_resp = server.handle_request(marks_req).await;
    let marks_text = marks_resp.result.unwrap()["content"][0]["text"]
        .as_str()
        .unwrap()
        .to_string();
    assert!(marks_text.contains("total_marks"));
}

#[tokio::test]
async fn mcp_resources_and_prompts() {
    let (_ctx, app) = setup_app().await;
    let server = McpServer::new(Arc::new(app));

    // List resources
    let res_list_req = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: Some(json!(10)),
        method: "resources/list".into(),
        params: None,
    };
    let res_list_resp = server.handle_request(res_list_req).await;
    let res_items = res_list_resp.result.unwrap()["resources"]
        .as_array()
        .unwrap()
        .clone();
    let res_uris: Vec<&str> = res_items.iter().filter_map(|r| r["uri"].as_str()).collect();

    assert!(res_uris.contains(&"asten://schema/canon-rules"));
    assert!(res_uris.contains(&"asten://entities"));
    assert!(res_uris.contains(&"asten://timeline"));

    // Read canon-rules resource
    let read_req = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: Some(json!(11)),
        method: "resources/read".into(),
        params: Some(json!({ "uri": "asten://schema/canon-rules" })),
    };
    let read_resp = server.handle_request(read_req).await;
    let schema_text = read_resp.result.unwrap()["contents"][0]["text"]
        .as_str()
        .unwrap()
        .to_string();
    assert!(schema_text.contains("AstenCanonRuleCatalog"));

    // List prompts
    let p_list_req = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: Some(json!(12)),
        method: "prompts/list".into(),
        params: None,
    };
    let p_list_resp = server.handle_request(p_list_req).await;
    let prompts = p_list_resp.result.unwrap()["prompts"]
        .as_array()
        .unwrap()
        .clone();
    let p_names: Vec<&str> = prompts.iter().filter_map(|p| p["name"].as_str()).collect();

    assert!(p_names.contains(&"audit_scene_continuity"));
    assert!(p_names.contains(&"resolve_timeline_conflict"));
    assert!(p_names.contains(&"flesh_out_cast_canon"));

    // Get prompt
    let p_get_req = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: Some(json!(13)),
        method: "prompts/get".into(),
        params: Some(json!({
            "name": "audit_scene_continuity",
            "arguments": { "scene_name": "Chapter 1 Opening" }
        })),
    };
    let p_get_resp = server.handle_request(p_get_req).await;
    let msg_content = p_get_resp.result.unwrap()["messages"][0]["content"]["text"]
        .as_str()
        .unwrap()
        .to_string();
    assert!(msg_content.contains("Chapter 1 Opening"));
    assert!(msg_content.contains("asten_marks_list"));
}

#[tokio::test]
async fn rejects_wrong_project_types_unknown_arguments_and_ambiguous_project() {
    let (_ctx, app) = setup_app().await;
    for name in ["One", "Two"] {
        app.create_project(CreateProject {
            name: name.into(),
            kind: ProjectKind::Novel,
            description: None,
        })
        .await
        .unwrap();
    }
    let server = McpServer::new(Arc::new(app));
    for arguments in [
        json!({"query":"x", "project_id": 1}),
        json!({"query":"x", "project_id":"../other"}),
        json!({"query":"x"}),
    ] {
        let response = server
            .handle_request(JsonRpcRequest {
                jsonrpc: "2.0".into(),
                id: Some(json!(1)),
                method: "tools/call".into(),
                params: Some(json!({"name":"asten_canon_search", "arguments":arguments})),
            })
            .await;
        assert_eq!(response.result.unwrap()["isError"], true);
    }
    let response = server
        .handle_request(JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: Some(json!(2)),
            method: "tools/call".into(),
            params: Some(
                json!({"name":"asten_canon_search", "arguments":{"query":"x", "extra":true}}),
            ),
        })
        .await;
    assert_eq!(response.error.unwrap().code, -32602);
}
