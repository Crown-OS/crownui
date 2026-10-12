use std::fmt;

use geometry::Rect;

use crate::ActionKind;

/// A stable identifier for a semantic node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SemanticId(pub u64);

/// What kind of thing a node is, named like ARIA roles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[allow(missing_docs)]
pub enum Role {
    Window,
    Group,
    Text,
    Heading,
    Button,
    Link,
    TextField,
    SearchField,
    Checkbox,
    Switch,
    RadioButton,
    Slider,
    ProgressBar,
    Image,
    List,
    ListItem,
    ScrollView,
    Menu,
    MenuItem,
    Tab,
    TabList,
    Dialog,
    Tooltip,
    ComboBox,
    Separator,
}

/// Boolean and tri-state properties of a node, named like ARIA states.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct States {
    /// Has keyboard focus.
    pub focused: bool,
    /// Can receive keyboard focus.
    pub focusable: bool,
    /// Does not accept interaction.
    pub disabled: bool,
    /// Checked state of checkboxes, switches and radio buttons.
    pub checked: Option<bool>,
    /// Expanded state of disclosures and menus.
    pub expanded: Option<bool>,
    /// Selected in a list or tab set.
    pub selected: bool,
    /// Holds a secret (password); its value is never exposed.
    pub secure: bool,
    /// Rendered but scrolled or clipped out of view.
    pub offscreen: bool,
}

/// One node of the semantic tree.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SemanticNode {
    /// Stable id.
    pub id: SemanticId,
    /// Kind of element.
    pub role: Role,
    /// Accessible name (the label a screen reader announces).
    pub name: Option<String>,
    /// Current value (text field contents, slider position …).
    pub value: Option<String>,
    /// Longer description.
    pub description: Option<String>,
    /// States.
    pub states: States,
    /// Actions this node supports.
    pub actions: Vec<ActionKind>,
    /// Bounds in window coordinates, logical pixels.
    pub bounds: Rect,
    /// Children in reading order.
    pub children: Vec<SemanticId>,
    /// Component path, for diagnostics (`app > sidebar > row[42] > button`).
    pub path: String,
}

impl SemanticNode {
    /// A node with only an id and a role.
    pub fn new(id: SemanticId, role: Role) -> Self {
        Self {
            id,
            role,
            name: None,
            value: None,
            description: None,
            states: States::default(),
            actions: Vec::new(),
            bounds: Rect::ZERO,
            children: Vec::new(),
            path: String::new(),
        }
    }

    /// The value as exposed outside the app: `None` for secure nodes.
    pub fn exposed_value(&self) -> Option<&str> {
        if self.states.secure {
            None
        } else {
            self.value.as_deref()
        }
    }

    /// Whether this node supports `action`.
    pub fn supports(&self, action: ActionKind) -> bool {
        self.actions.contains(&action)
    }
}

impl fmt::Display for Role {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = format!("{self:?}");
        let mut snake = String::with_capacity(name.len() + 4);
        for (index, character) in name.chars().enumerate() {
            if character.is_uppercase() && index > 0 {
                snake.push('_');
            }
            snake.push(character.to_ascii_lowercase());
        }
        formatter.write_str(&snake)
    }
}
