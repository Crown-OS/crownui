use std::fmt::Display;

use base64::{Engine, prelude::BASE64_STANDARD};
use rmcp::model::{CallToolResult, ContentBlock, JsonObject};
use semantics::{Query, Role, SemanticId};
use serde::{Deserialize, de::DeserializeOwned};
use serde_json::{Value, json};

use super::{
    action_arguments::parse_action,
    app_server::AppServer,
    builtin_tools::{BuiltinTool, CONFIRM_ARGUMENT},
};
use crate::{McpRequest, util::untrusted::untrusted};

pub(crate) struct ToolFailure(String);

impl<E: Display> From<E> for ToolFailure {
    fn from(error: E) -> Self {
        Self(error.to_string())
    }
}

impl ToolFailure {
    fn untrusted(message: String) -> Self {
        Self(Value::Object(untrusted(message)).to_string())
    }

    pub(super) fn into_result(self) -> CallToolResult {
        CallToolResult::error(vec![ContentBlock::text(self.0)])
    }
}

type ToolOutcome = Result<CallToolResult, ToolFailure>;

#[derive(Deserialize)]
struct QueryArguments {
    role: Option<Role>,
    name: Option<String>,
    text: Option<String>,
}

#[derive(Deserialize)]
struct ActArguments {
    id: SemanticId,
    revision: u64,
    action: Value,
}

impl AppServer {
    pub(super) async fn call_builtin(
        &self,
        tool: BuiltinTool,
        arguments: JsonObject,
    ) -> ToolOutcome {
        match tool {
            BuiltinTool::Tree => self.ui_tree().await,
            BuiltinTool::Query => self.ui_query(parse_arguments(arguments)?).await,
            BuiltinTool::Act => self.ui_act(parse_arguments(arguments)?).await,
            BuiltinTool::Screenshot => self.ui_screenshot().await,
            BuiltinTool::Inspect => self.inspect_snapshot().await,
        }
    }

    pub(super) async fn call_app_tool(
        &self,
        name: String,
        destructive: bool,
        mut arguments: JsonObject,
    ) -> ToolOutcome {
        if destructive
            && arguments
                .remove(CONFIRM_ARGUMENT)
                .and_then(|confirm| confirm.as_bool())
                != Some(true)
        {
            return Err(ToolFailure(format!(
                "confirmation required: `{name}` is destructive; ask the user for approval, then call it again with \"{CONFIRM_ARGUMENT}\": true"
            )));
        }
        self.acquire_action_slot()?;
        let arguments = Value::Object(arguments);
        let value = self
            .bridge
            .ask(|reply| McpRequest::CallTool {
                name,
                arguments,
                reply,
            })
            .await?
            .map_err(ToolFailure::untrusted)?;
        Ok(CallToolResult::structured(Value::Object(untrusted(value))))
    }

    async fn ui_tree(&self) -> ToolOutcome {
        let snapshot = self.bridge.ask(|reply| McpRequest::Tree { reply }).await?;
        Ok(at_revision(
            snapshot.revision,
            snapshot.tree.to_string().into(),
        ))
    }

    async fn ui_query(&self, arguments: QueryArguments) -> ToolOutcome {
        let mut query = Query::default();
        if let Some(role) = arguments.role {
            query = query.role(role);
        }
        if let Some(name) = arguments.name {
            query = query.name(name);
        }
        if let Some(text) = arguments.text {
            query = query.text(text);
        }
        let snapshot = self.bridge.ask(|reply| McpRequest::Tree { reply }).await?;
        let matches: Vec<Value> = query
            .find_all(&snapshot.tree)
            .into_iter()
            .map(|node| json!({ "id": node.id, "role": node.role, "name": node.name, "path": node.path }))
            .collect();
        Ok(at_revision(snapshot.revision, matches.into()))
    }

    async fn ui_act(&self, arguments: ActArguments) -> ToolOutcome {
        let ActArguments {
            id,
            revision,
            action,
        } = arguments;
        let action = parse_action(action)?;
        self.acquire_action_slot()?;
        let outcome = self
            .bridge
            .ask(|reply| McpRequest::Act {
                id,
                revision,
                action,
                reply,
            })
            .await??;
        Ok(CallToolResult::structured(
            json!({ "ok": true, "revision": outcome.revision }),
        ))
    }

    async fn ui_screenshot(&self) -> ToolOutcome {
        let screenshot = self
            .bridge
            .ask(|reply| McpRequest::Screenshot { reply })
            .await?
            .map_err(ToolFailure::untrusted)?;
        Ok(CallToolResult::success(vec![ContentBlock::image(
            BASE64_STANDARD.encode(&screenshot.png),
            "image/png",
        )]))
    }

    async fn inspect_snapshot(&self) -> ToolOutcome {
        let snapshot = self
            .bridge
            .ask(|reply| McpRequest::Inspect { reply })
            .await?;
        Ok(CallToolResult::structured(Value::Object(untrusted(
            serde_json::to_value(snapshot)?,
        ))))
    }

    fn acquire_action_slot(&self) -> Result<(), ToolFailure> {
        match &self.limiter {
            Some(limiter) if !limiter.try_acquire() => Err(ToolFailure(format!(
                "rate limit exceeded: at most {} actions per second; wait and retry",
                limiter.rate()
            ))),
            _ => Ok(()),
        }
    }
}

fn parse_arguments<T: DeserializeOwned>(arguments: JsonObject) -> Result<T, ToolFailure> {
    serde_json::from_value(Value::Object(arguments))
        .map_err(|error| ToolFailure(format!("invalid arguments: {error}")))
}

fn at_revision(revision: u64, content: Value) -> CallToolResult {
    let mut wrapper = untrusted(content);
    wrapper.insert("revision".into(), revision.into());
    CallToolResult::structured(Value::Object(wrapper))
}
