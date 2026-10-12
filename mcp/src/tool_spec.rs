use serde_json::{Value, json};

/// An app-defined tool offered to agents next to the built-in ones.
#[derive(Debug, Clone, PartialEq)]
pub struct ToolSpec {
    /// Unique tool name.
    pub name: String,
    /// What the tool does, for the agent.
    pub description: String,
    /// JSON Schema of the arguments object.
    pub input_schema: Value,
    /// Whether the tool changes or deletes user data; agents must pass `"confirm": true`.
    pub destructive: bool,
}

impl ToolSpec {
    /// A non-destructive tool taking an arbitrary object.
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            input_schema: json!({ "type": "object" }),
            destructive: false,
        }
    }

    /// Sets the arguments schema.
    pub fn input_schema(mut self, input_schema: Value) -> Self {
        self.input_schema = input_schema;
        self
    }

    /// Marks the tool destructive.
    pub fn destructive(mut self) -> Self {
        self.destructive = true;
        self
    }
}
