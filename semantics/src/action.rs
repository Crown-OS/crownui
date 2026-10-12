use geometry::{Point, Vector};
use input::Keybind;

/// Actions a node can support, as advertised to assistive technology and agents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[allow(missing_docs)]
pub enum ActionKind {
    Click,
    DoubleClick,
    LongPress,
    ContextMenu,
    Focus,
    Blur,
    Scroll,
    SetValue,
    Increment,
    Decrement,
    Expand,
    Collapse,
    Dismiss,
    Drag,
}

/// An action requested by a test, an agent or assistive technology, named like
/// Testing Library / Playwright user events.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "action", rename_all = "snake_case"))]
pub enum UiAction {
    /// Press and release the primary button at the node's center.
    Click,
    /// Two clicks in quick succession.
    DoubleClick,
    /// Move the pointer over the node.
    Hover,
    /// Focus the node and type text as committed input.
    TypeText {
        /// Text to insert.
        text: String,
    },
    /// Press a key combination while the node is focused.
    PressKey {
        /// The shortcut, e.g. `"Ctrl+A"`.
        keybind: Keybind,
    },
    /// Scroll the node's scroll container by a pixel delta.
    Scroll {
        /// Content offset change.
        delta: Vector,
    },
    /// Press at the node's center, move to `to` (window coordinates) and release.
    Drag {
        /// Where to release.
        to: Point,
    },
    /// Move keyboard focus to the node.
    Focus,
}

impl UiAction {
    /// The node capability this action needs, if a specific one is required.
    pub fn required_capability(&self) -> Option<ActionKind> {
        match self {
            Self::Click => Some(ActionKind::Click),
            Self::DoubleClick => Some(ActionKind::DoubleClick),
            Self::TypeText { .. } => Some(ActionKind::SetValue),
            Self::Scroll { .. } => Some(ActionKind::Scroll),
            Self::Drag { .. } => Some(ActionKind::Drag),
            Self::Focus => Some(ActionKind::Focus),
            Self::Hover | Self::PressKey { .. } => None,
        }
    }

    /// The shared action name (`click`, `type_text` …).
    pub fn name(&self) -> &'static str {
        match self {
            Self::Click => "click",
            Self::DoubleClick => "double_click",
            Self::Hover => "hover",
            Self::TypeText { .. } => "type_text",
            Self::PressKey { .. } => "press_key",
            Self::Scroll { .. } => "scroll",
            Self::Drag { .. } => "drag",
            Self::Focus => "focus",
        }
    }
}
