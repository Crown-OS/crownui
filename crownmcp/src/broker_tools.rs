use rmcp::model::{JsonObject, Tool, ToolAnnotations};
use serde_json::{Value, json};

pub(crate) const LIST_APPS: &str = "list_apps";
pub(crate) const SELECT_APP: &str = "select_app";

pub(crate) fn is_broker_tool(name: &str) -> bool {
    name == LIST_APPS || name == SELECT_APP
}

pub(crate) fn broker_tools() -> Vec<Tool> {
    vec![
        Tool::new(
            LIST_APPS,
            "Running CrownUI apps (name, pid, socket) and which one is selected.",
            object_schema(json!({ "type": "object", "properties": {} })),
        )
        .with_annotations(ToolAnnotations::new().read_only(true)),
        Tool::new(
            SELECT_APP,
            "Connect to an app by name or name-pid; its tools and resources then appear here.",
            object_schema(json!({
                "type": "object",
                "properties": { "app": { "type": "string", "description": "App name or name-pid from list_apps." } },
                "required": ["app"]
            })),
        )
        .with_annotations(ToolAnnotations::new().read_only(true)),
    ]
}

fn object_schema(schema: Value) -> JsonObject {
    match schema {
        Value::Object(schema) => schema,
        _ => JsonObject::new(),
    }
}
