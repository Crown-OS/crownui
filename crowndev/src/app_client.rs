use anyhow::{Context, Result, bail};
use mcp::AppEndpoint;
use rmcp::{
    RoleClient, ServiceExt,
    model::{
        CallToolRequestParams, CallToolResult, JsonObject, ReadResourceRequestParams,
        ResourceContents,
    },
    service::RunningService,
};
use serde_json::{Value, json};

use crate::query::NodeQuery;

const TREE_URI: &str = "ui://tree";
const INSPECT_URI: &str = "inspect://snapshot";
const UNTRUSTED_CONTENT: &str = "untrusted_app_content";

/// One node `ui_query` found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct QueryMatch {
    pub(crate) id: u64,
    pub(crate) role: String,
    pub(crate) name: Option<String>,
}

/// `ui_query` matches and the revision their ids are valid at.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct QueryResult {
    pub(crate) revision: u64,
    pub(crate) matches: Vec<QueryMatch>,
}

/// An MCP client session with one running app.
pub(crate) struct AppClient {
    service: RunningService<RoleClient, ()>,
    label: String,
}

impl AppClient {
    pub(crate) async fn connect(endpoint: &AppEndpoint, token: Option<&str>) -> Result<Self> {
        let label = endpoint.label();
        let stream = mcp::connect(&endpoint.path, token)
            .await
            .with_context(|| format!("cannot reach {label}"))?;
        let service = ()
            .serve(stream)
            .await
            .with_context(|| format!("{label} refused the connection; is CROWNMCP_TOKEN right?"))?;
        Ok(Self { service, label })
    }

    pub(crate) fn label(&self) -> &str {
        &self.label
    }

    pub(crate) async fn tree_text(&self) -> Result<String> {
        self.read_text(TREE_URI).await
    }

    pub(crate) async fn inspect(&self) -> Result<Value> {
        let text = self.read_text(INSPECT_URI).await?;
        serde_json::from_str(&text)
            .with_context(|| format!("{} sent an unreadable inspect snapshot", self.label))
    }

    pub(crate) async fn query(&self, query: &NodeQuery) -> Result<QueryResult> {
        let reply = self.call("ui_query", query.to_arguments()).await?;
        let revision = reply["revision"]
            .as_u64()
            .context("ui_query reply has no revision")?;
        let matches = reply[UNTRUSTED_CONTENT]
            .as_array()
            .context("ui_query reply has no matches")?
            .iter()
            .map(|found| {
                Ok(QueryMatch {
                    id: found["id"].as_u64().context("ui_query match has no id")?,
                    role: found["role"].as_str().unwrap_or_default().to_owned(),
                    name: found["name"].as_str().map(str::to_owned),
                })
            })
            .collect::<Result<_>>()?;
        Ok(QueryResult { revision, matches })
    }

    pub(crate) async fn act(&self, id: u64, revision: u64, action: &Value) -> Result<u64> {
        let arguments = json!({ "id": id, "revision": revision, "action": action });
        let Value::Object(arguments) = arguments else {
            bail!("ui_act arguments must be an object");
        };
        let reply = self.call("ui_act", arguments).await?;
        reply["revision"]
            .as_u64()
            .context("ui_act reply has no revision")
    }

    pub(crate) async fn close(self) -> Result<()> {
        self.service.cancel().await?;
        Ok(())
    }

    async fn read_text(&self, uri: &'static str) -> Result<String> {
        let result = self
            .service
            .read_resource(ReadResourceRequestParams::new(uri))
            .await
            .with_context(|| format!("{} could not read {uri}", self.label))?;
        match result.contents.into_iter().next() {
            Some(ResourceContents::TextResourceContents { text, .. }) => Ok(text),
            _ => bail!("{} sent no text for {uri}", self.label),
        }
    }

    async fn call(&self, tool: &'static str, arguments: JsonObject) -> Result<Value> {
        let result = self
            .service
            .call_tool(CallToolRequestParams::new(tool).with_arguments(arguments))
            .await
            .with_context(|| format!("{} could not run {tool}", self.label))?;
        if result.is_error == Some(true) {
            bail!(
                "{tool} failed: {}",
                first_text(&result).unwrap_or("no details")
            );
        }
        match result.structured_content {
            Some(value) => Ok(value),
            None => serde_json::from_str(first_text(&result).unwrap_or_default())
                .with_context(|| format!("{tool} returned no JSON")),
        }
    }
}

fn first_text(result: &CallToolResult) -> Option<&str> {
    result
        .content
        .first()
        .and_then(|content| content.as_text())
        .map(|text| text.text.as_str())
}
