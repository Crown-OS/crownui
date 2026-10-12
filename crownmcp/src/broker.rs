use std::{path::PathBuf, sync::Arc};

use mcp::AppEndpoint;
use rmcp::{
    ErrorData, Peer, RoleServer, ServerHandler, ServiceError,
    model::{
        CallToolRequestParams, CallToolResponse, CallToolResult, ContentBlock, Implementation,
        JsonObject, ListResourcesResult, ListToolsResult, PaginatedRequestParams,
        ReadResourceRequestParams, ReadResourceResponse, ServerCapabilities, ServerConfig,
    },
    service::RequestContext,
};
use serde_json::{Value, json};
use tokio::sync::watch;

use crate::{
    app_connection::AppConnection,
    broker_tools::{LIST_APPS, SELECT_APP, broker_tools, is_broker_tool},
    discovery::{discover_apps, find_app},
};

const INSTRUCTIONS: &str = "Bridges to running CrownUI apps. Call list_apps, then select_app (automatic \
when only one app runs); the app's ui_* tools and ui:// resources then appear here.";
const NO_APP_SELECTED: &str = "no app selected: call list_apps, then select_app";

pub(crate) struct Broker {
    socket_dir: PathBuf,
    token: Option<String>,
    selected: watch::Sender<Option<Arc<AppConnection>>>,
}

impl Broker {
    pub(crate) fn new(socket_dir: PathBuf, token: Option<String>) -> Self {
        Self {
            socket_dir,
            token,
            selected: watch::Sender::new(None),
        }
    }

    async fn connection(&self) -> Option<Arc<AppConnection>> {
        let selected = self.selected.borrow().clone();
        if let Some(connection) = selected.filter(|connection| !connection.is_closed()) {
            return Some(connection);
        }
        let mut apps = discover_apps(&self.socket_dir);
        if apps.len() != 1 {
            return None;
        }
        self.select(apps.pop()?).await.ok()
    }

    async fn select(&self, endpoint: AppEndpoint) -> anyhow::Result<Arc<AppConnection>> {
        let connection = Arc::new(AppConnection::open(endpoint, self.token.as_deref()).await?);
        self.selected.send_replace(Some(Arc::clone(&connection)));
        Ok(connection)
    }

    fn list_apps(&self) -> CallToolResult {
        let selected = self
            .selected
            .borrow()
            .as_ref()
            .map(|connection| connection.endpoint.label());
        let apps: Vec<Value> = discover_apps(&self.socket_dir)
            .into_iter()
            .map(|app| {
                let label = app.label();
                json!({
                    "selected": selected.as_ref() == Some(&label),
                    "app": label,
                    "name": app.app_name,
                    "pid": app.pid,
                    "socket": app.path,
                })
            })
            .collect();
        CallToolResult::structured(json!({ "apps": apps }))
    }

    async fn select_app(&self, arguments: &JsonObject, agent: &Peer<RoleServer>) -> CallToolResult {
        let Some(wanted) = arguments.get("app").and_then(Value::as_str) else {
            return failure("select_app needs {\"app\": \"<name or name-pid>\"}");
        };
        let endpoint = match find_app(discover_apps(&self.socket_dir), wanted) {
            Ok(endpoint) => endpoint,
            Err(message) => return failure(message),
        };
        match self.select(endpoint).await {
            Ok(connection) => {
                let _ = agent.notify_tool_list_changed().await;
                let _ = agent.notify_resource_list_changed().await;
                CallToolResult::structured(json!({ "selected": connection.endpoint.label() }))
            }
            Err(error) => failure(format!("{error:#}")),
        }
    }
}

impl ServerHandler for Broker {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(
            ServerCapabilities::builder()
                .enable_tools()
                .enable_tool_list_changed()
                .enable_resources()
                .enable_resources_list_changed()
                .build(),
        )
        .with_server_info(Implementation::new("crownmcp", env!("CARGO_PKG_VERSION")))
        .with_instructions(INSTRUCTIONS)
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, ErrorData> {
        let mut tools = broker_tools();
        if let Some(connection) = self.connection().await
            && let Ok(app_tools) = connection.peer().list_all_tools().await
        {
            tools.extend(
                app_tools
                    .into_iter()
                    .filter(|tool| !is_broker_tool(&tool.name)),
            );
        }
        Ok(ListToolsResult::with_all_items(tools))
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResponse, ErrorData> {
        if request.name == LIST_APPS {
            return Ok(self.list_apps().into());
        }
        if request.name == SELECT_APP {
            let arguments = request.arguments.unwrap_or_default();
            return Ok(self.select_app(&arguments, &context.peer).await.into());
        }
        match self.connection().await {
            Some(connection) => connection
                .peer()
                .call_tool_once(request)
                .await
                .map_err(upstream_error),
            None => Ok(failure(NO_APP_SELECTED).into()),
        }
    }

    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, ErrorData> {
        let Some(connection) = self.connection().await else {
            return Ok(ListResourcesResult::default());
        };
        let resources = connection
            .peer()
            .list_all_resources()
            .await
            .map_err(upstream_error)?;
        Ok(ListResourcesResult::with_all_items(resources))
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResponse, ErrorData> {
        let connection = self
            .connection()
            .await
            .ok_or_else(|| ErrorData::resource_not_found(NO_APP_SELECTED, None))?;
        connection
            .peer()
            .read_resource_once(request)
            .await
            .map_err(upstream_error)
    }
}

fn failure(message: impl Into<String>) -> CallToolResult {
    CallToolResult::error(vec![ContentBlock::text(message.into())])
}

fn upstream_error(error: ServiceError) -> ErrorData {
    match error {
        ServiceError::McpError(error) => error,
        other => ErrorData::internal_error(format!("app connection failed: {other}"), None),
    }
}
