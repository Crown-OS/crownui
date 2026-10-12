use rmcp::model::{JsonObject, Tool, ToolAnnotations};
use serde_json::{Map, Value, json};

use crate::ToolSpec;

pub(super) const CONFIRM_ARGUMENT: &str = "confirm";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum BuiltinTool {
    Tree,
    Query,
    Act,
    Screenshot,
    Inspect,
}

impl BuiltinTool {
    pub(super) const ALL: [Self; 5] = [
        Self::Tree,
        Self::Query,
        Self::Act,
        Self::Screenshot,
        Self::Inspect,
    ];

    pub(super) fn name(self) -> &'static str {
        match self {
            Self::Tree => "ui_tree",
            Self::Query => "ui_query",
            Self::Act => "ui_act",
            Self::Screenshot => "ui_screenshot",
            Self::Inspect => "inspect_snapshot",
        }
    }

    pub(super) fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|tool| tool.name() == name)
    }

    pub(super) fn definition(self) -> Tool {
        let (description, schema) = match self {
            Self::Tree => (
                "The app's semantic tree as indented text, with the revision ui_act needs.",
                json!({ "type": "object", "properties": {} }),
            ),
            Self::Query => (
                "Find nodes by role, exact accessible name and/or visible text; returns ids, roles, names and paths.",
                json!({
                    "type": "object",
                    "properties": {
                        "role": { "type": "string", "description": "Role in snake_case, e.g. button, text_field, checkbox, menu_item." },
                        "name": { "type": "string", "description": "Exact accessible name." },
                        "text": { "type": "string", "description": "Substring of the name or value." }
                    }
                }),
            ),
            Self::Act => (
                "Perform a user action on a node. Fails if the tree changed since `revision`.",
                json!({
                    "type": "object",
                    "properties": {
                        "id": { "type": "integer", "minimum": 0, "description": "Node id from ui_query or ui_tree.json." },
                        "revision": { "type": "integer", "minimum": 0, "description": "Revision the id was read at." },
                        "action": {
                            "type": "object",
                            "description": "Tagged by \"action\": {\"action\":\"click\"}, {\"action\":\"double_click\"}, {\"action\":\"hover\"}, {\"action\":\"focus\"}, {\"action\":\"type_text\",\"text\":\"hello\"}, {\"action\":\"press_key\",\"keybind\":\"Ctrl+A\"}, {\"action\":\"scroll\",\"delta\":{\"x\":0,\"y\":120}}, {\"action\":\"drag\",\"to\":{\"x\":40,\"y\":300}}.",
                            "properties": {
                                "action": {
                                    "type": "string",
                                    "enum": ["click", "double_click", "hover", "focus", "type_text", "press_key", "scroll", "drag"]
                                }
                            },
                            "required": ["action"]
                        }
                    },
                    "required": ["id", "revision", "action"]
                }),
            ),
            Self::Screenshot => (
                "A PNG screenshot of the app window.",
                json!({ "type": "object", "properties": {} }),
            ),
            Self::Inspect => (
                "Layout, reactive graph, render tree and frame timings of the app, as JSON.",
                json!({ "type": "object", "properties": {} }),
            ),
        };
        Tool::new(self.name(), description, object_schema(schema))
            .with_annotations(ToolAnnotations::new().read_only(self != Self::Act))
    }
}

pub(super) fn app_tool_definition(spec: &ToolSpec) -> Tool {
    let mut schema = object_schema(spec.input_schema.clone());
    if spec.destructive {
        require_confirmation(&mut schema);
    }
    Tool::new(spec.name.clone(), spec.description.clone(), schema)
        .with_annotations(ToolAnnotations::new().destructive(spec.destructive))
}

fn object_schema(schema: Value) -> JsonObject {
    match schema {
        Value::Object(schema) => schema,
        _ => Map::from_iter([("type".to_owned(), Value::from("object"))]),
    }
}

fn require_confirmation(schema: &mut JsonObject) {
    if let Value::Object(properties) = schema
        .entry("properties")
        .or_insert_with(|| Value::Object(Map::new()))
    {
        properties.insert(
            CONFIRM_ARGUMENT.into(),
            json!({
                "type": "boolean",
                "const": true,
                "description": "Destructive tool: set to true only after the user approved this call."
            }),
        );
    }
    if let Value::Array(required) = schema
        .entry("required")
        .or_insert_with(|| Value::Array(Vec::new()))
    {
        required.push(CONFIRM_ARGUMENT.into());
    }
}
