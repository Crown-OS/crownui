use reactive::ScopeId;

use crate::Cx;

type ToolHandler = Box<dyn FnMut(&mut Cx, &str) -> Result<String, String>>;

/// An action a component exposes to AI agents over MCP. Arguments and results are JSON text so
/// the view layer stays free of serialization dependencies; `crownui` adds typed helpers.
pub struct AppTool {
    /// Unique tool name, e.g. `add_todo`.
    pub name: String,
    /// What the tool does, shown to agents.
    pub description: String,
    /// JSON Schema of the arguments object.
    pub input_schema: String,
    /// Whether agents must confirm before calling it.
    pub destructive: bool,
    handler: ToolHandler,
}

impl AppTool {
    /// A tool whose `handler` receives the arguments as JSON text and returns JSON text.
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        input_schema: impl Into<String>,
        handler: impl FnMut(&mut Cx, &str) -> Result<String, String> + 'static,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            input_schema: input_schema.into(),
            destructive: false,
            handler: Box::new(handler),
        }
    }

    /// Marks the tool as destructive (agents must confirm).
    pub fn destructive(mut self) -> Self {
        self.destructive = true;
        self
    }
}

/// Public description of a registered tool.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolInfo {
    /// Tool name.
    pub name: String,
    /// Description.
    pub description: String,
    /// JSON Schema of the arguments.
    pub input_schema: String,
    /// Whether agents must confirm.
    pub destructive: bool,
}

struct Registered {
    scope: ScopeId,
    tool: Option<AppTool>,
}

#[derive(Default)]
pub(crate) struct ToolRegistry {
    tools: Vec<Registered>,
    changed: bool,
}

impl ToolRegistry {
    fn prune(&mut self, is_alive: impl Fn(ScopeId) -> bool) {
        let before = self.tools.len();
        self.tools.retain(|registered| is_alive(registered.scope));
        self.changed |= before != self.tools.len();
    }
}

impl Cx {
    /// Exposes `tool` to agents while the current component stays mounted.
    pub fn register_tool(&mut self, tool: AppTool) {
        let scope = self.current_scope();
        self.tools.tools.retain(|registered| {
            registered
                .tool
                .as_ref()
                .is_none_or(|existing| existing.name != tool.name)
        });
        self.tools.tools.push(Registered {
            scope,
            tool: Some(tool),
        });
        self.tools.changed = true;
    }

    pub(crate) fn live_tools(&mut self) -> Vec<ToolInfo> {
        let registry = std::mem::take(&mut self.tools);
        let mut registry = registry;
        registry.prune(|scope| self.is_scope_alive(scope));
        let infos = registry
            .tools
            .iter()
            .filter_map(|registered| registered.tool.as_ref())
            .map(|tool| ToolInfo {
                name: tool.name.clone(),
                description: tool.description.clone(),
                input_schema: tool.input_schema.clone(),
                destructive: tool.destructive,
            })
            .collect();
        self.tools = registry;
        infos
    }

    pub(crate) fn take_tools_changed(&mut self) -> bool {
        let registry = std::mem::take(&mut self.tools);
        let mut registry = registry;
        registry.prune(|scope| self.is_scope_alive(scope));
        let changed = std::mem::take(&mut registry.changed);
        self.tools = registry;
        changed
    }

    pub(crate) fn call_tool(&mut self, name: &str, arguments: &str) -> Result<String, String> {
        let index = self
            .tools
            .tools
            .iter()
            .position(|registered| {
                registered
                    .tool
                    .as_ref()
                    .is_some_and(|tool| tool.name == name)
            })
            .ok_or_else(|| format!("no app tool named {name:?}"))?;
        let scope = self.tools.tools[index].scope;
        if !self.is_scope_alive(scope) {
            return Err(format!("tool {name:?} belongs to an unmounted component"));
        }
        let Some(mut tool) = self.tools.tools[index].tool.take() else {
            return Err(format!("tool {name:?} is already running"));
        };
        let previous = self.set_current_scope(scope);
        let result = (tool.handler)(self, arguments);
        self.set_current_scope(previous);
        if let Some(slot) = self
            .tools
            .tools
            .iter_mut()
            .find(|registered| registered.scope == scope && registered.tool.is_none())
        {
            slot.tool = Some(tool);
        }
        self.apply_pending();
        result
    }
}
