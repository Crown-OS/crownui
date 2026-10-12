use geometry::{Rect, Size};
use graphics::{DrawList, TextLayoutId};
use input::Keybind;
use motion::Curve;
use reactive::{ScopeId, Signal};
use semantics::{Role, SemanticId};
use slotmap::new_key_type;
use smallvec::SmallVec;

use crate::{
    dynamic::DynamicChildren,
    handler::Listener,
    modifier::Variant,
    style::{NodeStyle, TextOverrides, Visual},
    style_prop::StyleProp,
};

new_key_type! {
    /// Identifies a mounted element.
    pub struct NodeId;
}

/// What a node draws besides its box.
pub enum NodeContent {
    /// Only children.
    Container,
    /// A run of text.
    Text(TextContent),
    /// Custom drawing recorded by a painter.
    Canvas(CanvasContent),
    /// An image uploaded to the renderer.
    Image(ImageContent),
}

/// Text content and its shaping state.
pub struct TextContent {
    /// The text.
    pub text: String,
    /// The shaped layout drawn last, if any.
    pub layout: Option<TextLayoutId>,
}

/// A canvas's painter output and size feed.
pub struct CanvasContent {
    /// The node's size, which the painter effect reads.
    pub size: Signal<Size>,
    /// The latest recorded list.
    pub list: DrawList,
}

/// An image reference.
pub struct ImageContent {
    /// Renderer image id.
    pub image: graphics::ImageId,
    /// Natural size in logical pixels.
    pub natural_size: Size,
}

/// Accessibility properties set on a node.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct NodeSemantics {
    /// Accessible name.
    pub label: Option<String>,
    /// Accessible value.
    pub value: Option<String>,
    /// Description.
    pub description: Option<String>,
    /// Role override.
    pub role: Option<Role>,
    /// Focusability override.
    pub focusable: Option<bool>,
    /// Hide the value.
    pub secure: bool,
    /// Checked state for toggles.
    pub checked: Option<bool>,
}

/// Pointer and focus state that drives style variants.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct InteractionState {
    /// Pointer is over the node or a descendant.
    pub hovered: bool,
    /// Pressed.
    pub pressed: bool,
    /// Has keyboard focus.
    pub focused: bool,
    /// Focus arrived by keyboard.
    pub focus_visible: bool,
}

/// Computed geometry of a node.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct NodeGeometry {
    /// Rect relative to the parent's border box.
    pub rect: Rect,
    /// Rect in window coordinates (after scroll offsets, before transforms).
    pub window_rect: Rect,
    /// Content size, which may exceed `rect` in scroll containers.
    pub content_size: Size,
}

/// What has to be redone for a node.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Dirty {
    /// Layout style changed.
    pub style: bool,
    /// Children changed.
    pub children: bool,
    /// Text must be measured again.
    pub measure: bool,
    /// Visual (box paint or composite) changed.
    pub visual: bool,
    /// Content changed.
    pub content: bool,
    /// Frame changed.
    pub frame: bool,
}

/// What the renderer last received for this node.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SentState {
    /// Whether the renderer knows the node.
    pub created: bool,
    /// Last frame sent.
    pub frame: Option<Rect>,
    /// Last visual sent.
    pub visual: Option<Visual>,
}

/// One mounted element.
pub struct ElementNode {
    pub(crate) parent: Option<NodeId>,
    pub(crate) children: Vec<NodeId>,
    pub(crate) scope: ScopeId,
    pub(crate) name: &'static str,
    pub(crate) role: Role,
    pub(crate) content: NodeContent,
    pub(crate) style: NodeStyle,
    pub(crate) variants: SmallVec<[(Variant, Vec<StyleProp>); 2]>,
    pub(crate) enter: Option<Vec<StyleProp>>,
    pub(crate) exit: Option<Vec<StyleProp>>,
    pub(crate) visual: Visual,
    pub(crate) text: TextOverrides,
    pub(crate) resolved_text: Option<graphics::TextStyle>,
    pub(crate) listeners: Vec<Listener>,
    pub(crate) transition: Option<Curve>,
    pub(crate) semantics: NodeSemantics,
    pub(crate) disabled: bool,
    pub(crate) shortcut: Option<Keybind>,
    pub(crate) key: Option<u64>,
    pub(crate) semantic_id: SemanticId,
    pub(crate) geometry: NodeGeometry,
    pub(crate) interaction: InteractionState,
    pub(crate) dirty: Dirty,
    pub(crate) sent: SentState,
    pub(crate) dynamic: Option<DynamicChildren>,
    pub(crate) size_signal: Option<Signal<Size>>,
    pub(crate) disabled_signal: Option<Signal<bool>>,
}

impl ElementNode {
    pub(crate) fn new(parent: Option<NodeId>, scope: ScopeId, name: &'static str) -> Self {
        Self {
            parent,
            children: Vec::new(),
            scope,
            name,
            role: Role::Group,
            content: NodeContent::Container,
            style: NodeStyle::default(),
            variants: SmallVec::new(),
            enter: None,
            exit: None,
            visual: Visual::default(),
            text: TextOverrides::default(),
            resolved_text: None,
            listeners: Vec::new(),
            transition: None,
            semantics: NodeSemantics::default(),
            disabled: false,
            shortcut: None,
            key: None,
            semantic_id: SemanticId(0),
            geometry: NodeGeometry::default(),
            interaction: InteractionState::default(),
            dirty: Dirty {
                style: true,
                children: true,
                measure: true,
                visual: true,
                content: true,
                frame: true,
            },
            sent: SentState::default(),
            dynamic: None,
            size_signal: None,
            disabled_signal: None,
        }
    }

    /// The node's name in component paths (`vstack`, `button` …).
    pub fn name(&self) -> &'static str {
        self.name
    }

    /// Children in order.
    pub fn children(&self) -> &[NodeId] {
        &self.children
    }

    /// Computed geometry.
    pub fn geometry(&self) -> NodeGeometry {
        self.geometry
    }

    /// The effective visual, variants applied.
    pub fn visual(&self) -> &Visual {
        &self.visual
    }

    /// The full style before variants.
    pub fn style(&self) -> &NodeStyle {
        &self.style
    }

    /// Whether a listener of `kind` is attached.
    pub fn listens_to(&self, kind: crate::handler::ListenerKind) -> bool {
        self.listeners.iter().any(|listener| listener.kind() == kind)
    }

    /// Whether keyboard focus can land on this node.
    pub fn is_focusable(&self) -> bool {
        !self.disabled
            && self.semantics.focusable.unwrap_or_else(|| {
                matches!(
                    self.role,
                    Role::Button | Role::Link | Role::TextField | Role::SearchField | Role::Checkbox | Role::Switch | Role::Slider
                ) || self.listens_to(crate::handler::ListenerKind::Click)
            })
    }
}
