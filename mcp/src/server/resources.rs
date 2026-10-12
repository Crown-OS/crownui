use rmcp::{
    ErrorData,
    model::{ListResourcesResult, ReadResourceResult, Resource, ResourceContents},
};

use super::{app_server::AppServer, tree_json::tree_json};
use crate::McpRequest;

const TREE_TEXT_URI: &str = "ui://tree";
const TREE_JSON_URI: &str = "ui://tree.json";
const INSPECT_URI: &str = "inspect://snapshot";
const JSON_MIME: &str = "application/json";

pub(super) fn resource_list() -> ListResourcesResult {
    ListResourcesResult::with_all_items(vec![
        Resource::new(TREE_TEXT_URI, "ui_tree")
            .with_description("The semantic tree as indented text.")
            .with_mime_type("text/plain"),
        Resource::new(TREE_JSON_URI, "ui_tree_json")
            .with_description(
                "The semantic tree as JSON nodes in reading order, with ids and revision.",
            )
            .with_mime_type(JSON_MIME),
        Resource::new(INSPECT_URI, "inspect_snapshot")
            .with_description("Layout, signals, render tree and frame timings.")
            .with_mime_type(JSON_MIME),
    ])
}

impl AppServer {
    pub(super) async fn read(&self, uri: &str) -> Result<ReadResourceResult, ErrorData> {
        let contents = match uri {
            TREE_TEXT_URI => ResourceContents::text(self.tree().await?.tree.to_string(), uri),
            TREE_JSON_URI => json_contents(&tree_json(&self.tree().await?), uri),
            INSPECT_URI => {
                let snapshot = self
                    .bridge
                    .ask(|reply| McpRequest::Inspect { reply })
                    .await
                    .map_err(internal_error)?;
                json_contents(
                    &serde_json::to_value(snapshot).map_err(internal_error)?,
                    uri,
                )
            }
            _ => {
                return Err(ErrorData::resource_not_found(
                    format!("no resource {uri}"),
                    None,
                ));
            }
        };
        Ok(ReadResourceResult::new(vec![contents]))
    }

    async fn tree(&self) -> Result<crate::TreeSnapshot, ErrorData> {
        self.bridge
            .ask(|reply| McpRequest::Tree { reply })
            .await
            .map_err(internal_error)
    }
}

fn json_contents(value: &serde_json::Value, uri: &str) -> ResourceContents {
    ResourceContents::text(value.to_string(), uri).with_mime_type(JSON_MIME)
}

fn internal_error(error: impl std::fmt::Display) -> ErrorData {
    ErrorData::internal_error(error.to_string(), None)
}
