use mcp::{ActError, ActOutcome, McpConfig, McpHandle, McpRequest, McpServer, TreeSnapshot};
use semantics::UiAction;
use view::{ActionError, Ui};
use windowing::Waker;

/// Answers MCP agents from the UI thread.
pub(crate) struct McpBridge {
    handle: McpHandle,
    awaiting_frame: Vec<mcp::Responder<Result<ActOutcome, ActError>>>,
    frames: Vec<inspect::FrameStats>,
}

fn app_socket_name(title: &str) -> String {
    let name: String = title
        .chars()
        .map(|character| if character.is_ascii_alphanumeric() { character.to_ascii_lowercase() } else { '-' })
        .collect();
    if name.is_empty() { "crownui".into() } else { name }
}

impl McpBridge {
    pub(crate) fn start(title: &str, wake: Waker) -> Option<Self> {
        match McpServer::start(McpConfig::new(app_socket_name(title))) {
            Ok(handle) => {
                handle.set_wakeup(wake);
                tracing::info!(socket = %handle.socket_path().display(), "MCP server listening");
                Some(Self {
                    handle,
                    awaiting_frame: Vec::new(),
                    frames: Vec::new(),
                })
            }
            Err(error) => {
                tracing::error!("CUI0408: MCP server unavailable: {error}");
                None
            }
        }
    }

    pub(crate) fn poll(&mut self, ui: &mut Ui) {
        if ui.tools_changed() {
            let specs = ui
                .tools()
                .into_iter()
                .map(|tool| {
                    let schema = serde_json::from_str(&tool.input_schema)
                        .unwrap_or_else(|_| serde_json::json!({ "type": "object" }));
                    let spec = mcp::ToolSpec::new(tool.name, tool.description).input_schema(schema);
                    if tool.destructive { spec.destructive() } else { spec }
                })
                .collect();
            self.handle.set_tools(specs);
        }
        let requests: Vec<McpRequest> = self.handle.pending_requests().collect();
        for request in requests {
            match request {
                McpRequest::Tree { reply } => reply.respond(TreeSnapshot::from(ui.semantic_tree().clone())),
                McpRequest::Act {
                    id,
                    revision,
                    action,
                    reply,
                } => match act(ui, id, revision, &action) {
                    Ok(()) => self.awaiting_frame.push(reply),
                    Err(error) => reply.respond(Err(error)),
                },
                McpRequest::CallTool { name, arguments, reply } => {
                    let result = ui.call_tool(&name, &arguments.to_string()).and_then(|output| {
                        serde_json::from_str(&output).or(Ok(serde_json::Value::String(output)))
                    });
                    reply.respond(result);
                }
                McpRequest::Inspect { reply } => reply.respond(inspect::InspectSnapshot {
                    version: inspect::SNAPSHOT_VERSION,
                    semantics: ui.semantic_tree().to_string(),
                    layout: ui.inspect_layout(),
                    signals: ui.inspect_signals(),
                    frames: self.frames.clone(),
                    ..inspect::InspectSnapshot::default()
                }),
                McpRequest::Screenshot { reply } => {
                    reply.respond(Err("screenshots need the crownos-agent-access protocol (not yet available)".into()));
                }
            }
        }
    }
}

impl McpBridge {
    pub(crate) fn set_frames(&mut self, frames: Vec<inspect::FrameStats>) {
        self.frames = frames;
    }

    /// Answers actions once the frame they caused is laid out, so the reported revision is the
    /// one agents will read next.
    pub(crate) fn after_frame(&mut self, ui: &mut Ui) {
        if self.awaiting_frame.is_empty() {
            return;
        }
        let revision = ui.semantic_tree().revision();
        for reply in self.awaiting_frame.drain(..) {
            reply.respond(Ok(ActOutcome { revision }));
        }
    }
}

fn act(ui: &mut Ui, id: semantics::SemanticId, revision: u64, action: &UiAction) -> Result<(), ActError> {
    mcp::validate_action(ui.semantic_tree(), id, revision, action)?;
    ui.perform(id, action).map_err(|error| match error {
        ActionError::UnknownNode(_) => ActError::UnknownNode { id },
        ActionError::Secure(_) => ActError::Secure,
        ActionError::Disabled(_) => ActError::Disabled,
        ActionError::Unsupported { action, .. } => ActError::Unsupported {
            action,
            role: semantics::Role::Group,
        },
    })
}
