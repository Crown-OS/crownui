use std::{fs, path::Path, sync::mpsc, thread::JoinHandle, time::Duration};

use anyhow::{Context, Result};
use geometry::Rect;
use mcp::{ActOutcome, McpConfig, McpHandle, McpRequest, McpServer, SameUserGate, validate_action};
use rmcp::{
    RoleClient, ServiceExt,
    model::{
        CallToolRequestParams, CallToolResult, JsonObject, ReadResourceRequestParams,
        ResourceContents,
    },
    service::RunningService,
};
use semantics::{ActionKind, Role, SemanticId, SemanticNode, SemanticTree};
use serde_json::{Value, json};

use crate::broker::Broker;

type Agent = RunningService<RoleClient, ()>;

fn sample_tree() -> SemanticTree {
    let mut window = SemanticNode::new(SemanticId(1), Role::Window);
    window.children = vec![SemanticId(2)];
    let mut save = SemanticNode::new(SemanticId(2), Role::Button);
    save.name = Some("Save".into());
    save.actions = vec![ActionKind::Click];
    save.bounds = Rect::from_xywh(0.0, 0.0, 80.0, 32.0);
    let mut tree = SemanticTree::new();
    tree.replace(Some(SemanticId(1)), [window, save]);
    tree
}

fn spawn_fake_ui(mut handle: McpHandle) -> (mpsc::Sender<bool>, JoinHandle<()>) {
    let (signals, receiver) = mpsc::channel();
    let waker = signals.clone();
    handle.set_wakeup(move || {
        let _ = waker.send(false);
    });
    let thread = std::thread::spawn(move || {
        let tree = sample_tree();
        while !matches!(
            receiver.recv_timeout(Duration::from_millis(50)),
            Ok(true) | Err(mpsc::RecvTimeoutError::Disconnected)
        ) {
            for request in handle.pending_requests() {
                match request {
                    McpRequest::Tree { reply } => reply.respond(tree.clone().into()),
                    McpRequest::Act {
                        id,
                        revision,
                        action,
                        reply,
                    } => reply.respond(
                        validate_action(&tree, id, revision, &action)
                            .map(|_| ActOutcome { revision }),
                    ),
                    McpRequest::CallTool { reply, .. } => reply.respond(Err("no app tools".into())),
                    McpRequest::Inspect { reply } => reply.respond(Default::default()),
                    McpRequest::Screenshot { reply } => reply.respond(Err("not supported".into())),
                }
            }
        }
    });
    (signals, thread)
}

async fn agent_for(socket_dir: &Path) -> Result<Agent> {
    let (broker_side, agent_side) = tokio::io::duplex(1 << 16);
    let broker = Broker::new(socket_dir.to_owned(), None);
    tokio::spawn(async move {
        if let Ok(session) = broker.serve(broker_side).await {
            let _ = session.waiting().await;
        }
    });
    Ok(().serve(agent_side).await?)
}

async fn call(agent: &Agent, tool: &'static str, arguments: Value) -> Result<CallToolResult> {
    let arguments = match arguments {
        Value::Object(arguments) => arguments,
        _ => JsonObject::new(),
    };
    Ok(agent
        .call_tool(CallToolRequestParams::new(tool).with_arguments(arguments))
        .await?)
}

fn json_of(result: &CallToolResult) -> Result<Value> {
    let text = result
        .content
        .first()
        .and_then(|content| content.as_text())
        .context("text content")?;
    Ok(serde_json::from_str(&text.text)?)
}

#[tokio::test]
async fn lists_selects_and_forwards_to_an_app() -> Result<()> {
    let socket_dir = tempfile::tempdir()?;
    let handle = McpServer::start(
        McpConfig::new("notes")
            .socket_dir(socket_dir.path())
            .gate(SameUserGate),
    )?;
    let (ui_signals, ui_thread) = spawn_fake_ui(handle);
    let ghost = socket_dir.path().join("ghost-999999999.sock");
    fs::write(&ghost, b"")?;
    let agent = agent_for(socket_dir.path()).await?;

    let apps = json_of(&call(&agent, "list_apps", json!({})).await?)?;
    let label = format!("notes-{}", std::process::id());
    assert_eq!(apps["apps"].as_array().map(Vec::len), Some(1));
    assert_eq!(
        (
            apps["apps"][0]["app"].as_str(),
            apps["apps"][0]["selected"].as_bool()
        ),
        (Some(label.as_str()), Some(false))
    );
    assert!(!ghost.exists());

    let missing = call(&agent, "select_app", json!({ "app": "calculator" })).await?;
    assert_eq!(missing.is_error, Some(true));
    let selected = json_of(&call(&agent, "select_app", json!({ "app": "notes" })).await?)?;
    assert_eq!(selected["selected"], label);

    let tools = agent.list_all_tools().await?;
    assert!(
        ["list_apps", "select_app", "ui_tree", "ui_act"]
            .iter()
            .all(|name| tools.iter().any(|tool| tool.name == *name))
    );

    let found = json_of(&call(&agent, "ui_query", json!({ "role": "button" })).await?)?;
    assert_eq!(found["untrusted_app_content"][0]["name"], "Save");
    let clicked = call(
        &agent,
        "ui_act",
        json!({ "id": 2, "revision": found["revision"], "action": { "action": "click" } }),
    )
    .await?;
    assert_eq!(clicked.is_error, Some(false));

    let tree = agent
        .read_resource(ReadResourceRequestParams::new("ui://tree"))
        .await?;
    assert!(matches!(
        tree.contents.first(),
        Some(ResourceContents::TextResourceContents { text, .. }) if *text == sample_tree().to_string()
    ));

    agent.cancel().await?;
    let _ = ui_signals.send(true);
    ui_thread
        .join()
        .map_err(|_| anyhow::anyhow!("UI thread panicked"))
}

#[tokio::test]
async fn a_single_running_app_is_selected_automatically() -> Result<()> {
    let socket_dir = tempfile::tempdir()?;
    let agent = agent_for(socket_dir.path()).await?;
    let unselected = call(&agent, "ui_tree", json!({})).await?;
    assert_eq!(unselected.is_error, Some(true));

    let handle = McpServer::start(McpConfig::new("solo").socket_dir(socket_dir.path()))?;
    let (ui_signals, ui_thread) = spawn_fake_ui(handle);
    let tree = json_of(&call(&agent, "ui_tree", json!({})).await?)?;
    assert_eq!(tree["untrusted_app_content"], sample_tree().to_string());

    agent.cancel().await?;
    let _ = ui_signals.send(true);
    ui_thread
        .join()
        .map_err(|_| anyhow::anyhow!("UI thread panicked"))
}
