use std::{sync::mpsc, thread::JoinHandle, time::Duration};

use anyhow::{Context, Result, anyhow};
use geometry::Rect;
use mcp::{ActOutcome, McpConfig, McpHandle, McpRequest, McpServer, SameUserGate, validate_action};
use semantics::{ActionKind, Role, SemanticId, SemanticNode, SemanticTree, UiAction};
use tempfile::TempDir;

use crate::{
    action::parse_action,
    cli::Command,
    commands::{Session, run},
    query::NodeQuery,
};

const PLUS: SemanticId = SemanticId(2);

enum UiSignal {
    Wake,
    Stop,
}

fn button(id: u64, name: &str, x: f32) -> SemanticNode {
    let mut node = SemanticNode::new(SemanticId(id), Role::Button);
    node.name = Some(name.into());
    node.actions = vec![ActionKind::Click];
    node.bounds = Rect::from_xywh(x, 0.0, 40.0, 30.0);
    node
}

fn publish_counter(tree: &mut SemanticTree, count: i32) {
    let mut window = SemanticNode::new(SemanticId(1), Role::Window);
    window.children = vec![PLUS, SemanticId(3), SemanticId(4)];
    let mut label = SemanticNode::new(SemanticId(4), Role::Text);
    label.name = Some(format!("Count: {count}"));
    tree.replace(
        Some(SemanticId(1)),
        [window, button(2, "+1", 0.0), button(3, "-1", 50.0), label],
    );
}

/// A stand-in UI thread: a counter whose `+1` button answers `ui_act` clicks.
struct CounterApp {
    signals: mpsc::Sender<UiSignal>,
    thread: JoinHandle<()>,
    socket_dir: TempDir,
}

impl CounterApp {
    fn start() -> Result<Self> {
        let socket_dir = tempfile::tempdir()?;
        let mut handle = McpServer::start(
            McpConfig::new("counter")
                .socket_dir(socket_dir.path())
                .gate(SameUserGate),
        )?;
        let (signals, receiver) = mpsc::channel();
        let waker = signals.clone();
        handle.set_wakeup(move || {
            let _ = waker.send(UiSignal::Wake);
        });
        let thread = std::thread::spawn(move || serve_counter(&mut handle, &receiver));
        Ok(Self {
            signals,
            thread,
            socket_dir,
        })
    }

    fn session(&self) -> Session {
        Session {
            socket_dir: self.socket_dir.path().to_owned(),
            token: None,
        }
    }

    fn stop(self) -> Result<()> {
        let _ = self.signals.send(UiSignal::Stop);
        self.thread
            .join()
            .map_err(|_| anyhow!("UI thread panicked"))
    }
}

fn serve_counter(handle: &mut McpHandle, receiver: &mpsc::Receiver<UiSignal>) {
    let mut tree = SemanticTree::new();
    let mut count = 0;
    publish_counter(&mut tree, count);
    while !matches!(
        receiver.recv_timeout(Duration::from_millis(50)),
        Ok(UiSignal::Stop) | Err(mpsc::RecvTimeoutError::Disconnected)
    ) {
        for request in handle.pending_requests() {
            match request {
                McpRequest::Tree { reply } => reply.respond(tree.clone().into()),
                McpRequest::Act {
                    id,
                    revision,
                    action,
                    reply,
                } => {
                    let valid = validate_action(&tree, id, revision, &action).map(|_| ());
                    if valid.is_ok() && id == PLUS && action == UiAction::Click {
                        count += 1;
                        publish_counter(&mut tree, count);
                    }
                    reply.respond(valid.map(|()| ActOutcome {
                        revision: tree.revision(),
                    }));
                }
                McpRequest::CallTool { reply, .. } => reply.respond(Err("no app tools".into())),
                McpRequest::Inspect { reply } => reply.respond(Default::default()),
                McpRequest::Screenshot { reply } => reply.respond(Err("not supported".into())),
            }
        }
    }
}

async fn output_of(command: Command, session: &Session) -> Result<String> {
    let mut out = Vec::new();
    run(&command, session, &mut out).await?;
    Ok(String::from_utf8(out)?)
}

fn act(query: &str, action: &str) -> Result<Command> {
    Ok(Command::Act {
        app: "counter".into(),
        query: NodeQuery::parse(query)?,
        action: parse_action(action)?,
    })
}

#[tokio::test]
async fn lists_prints_and_acts_on_a_running_app() -> Result<()> {
    let app = CounterApp::start()?;
    let session = app.session();

    let apps = output_of(Command::Apps, &session).await?;
    assert!(
        apps.starts_with(&format!("counter-{}\t", std::process::id())),
        "{apps}"
    );

    let tree = output_of(Command::Tree { app: None }, &session).await?;
    assert!(
        tree.contains("button \"+1\"") && tree.contains("Count: 0"),
        "{tree}"
    );

    let report = output_of(act("role=button,name=+1", "click")?, &session).await?;
    assert!(report.starts_with("click button \"+1\" (id 2)"), "{report}");
    let tree = output_of(
        Command::Tree {
            app: Some("counter".into()),
        },
        &session,
    )
    .await?;
    assert!(tree.contains("Count: 1"), "{tree}");

    let ambiguous = output_of(act("role=button", "click")?, &session).await;
    let message = ambiguous
        .err()
        .context("two buttons must not be clicked blindly")?
        .to_string();
    assert!(message.contains("2 nodes match"), "{message}");

    let snapshot = output_of(Command::Inspect { app: None }, &session).await?;
    assert!(
        serde_json::from_str::<serde_json::Value>(&snapshot)?.is_object(),
        "{snapshot}"
    );

    app.stop()
}

#[tokio::test]
async fn missing_apps_are_reported() -> Result<()> {
    let socket_dir = tempfile::tempdir()?;
    let session = Session {
        socket_dir: socket_dir.path().to_owned(),
        token: None,
    };
    assert!(
        output_of(Command::Apps, &session)
            .await?
            .starts_with("no CrownUI app is running")
    );
    assert!(
        output_of(Command::Tree { app: None }, &session)
            .await
            .is_err()
    );
    Ok(())
}
