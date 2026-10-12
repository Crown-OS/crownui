use input::Keybind;
use motion::Curve;
use reactive::Runtime;
use semantics::Role;

use crate::{handler::Listener, prop::Prop, style_prop::StyleProp};

/// Interaction and environment states that switch style variants on, like Tailwind's
/// `hover:`, `active:`, `focus:`, `focus-visible:`, `disabled:` and `dark:` prefixes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Variant {
    /// The pointer is over the node.
    Hover,
    /// The node is pressed.
    Active,
    /// The node has keyboard focus.
    Focus,
    /// The node has focus that arrived by keyboard.
    FocusVisible,
    /// The node is disabled.
    Disabled,
    /// The environment's color scheme is dark.
    Dark,
}

/// One style assignment inside a variant: a literal, or recomputed from signals (e.g. a theme).
pub enum VariantProp {
    /// A literal value.
    Literal(StyleProp),
    /// Recomputed when the signals it reads change.
    Bound(Box<dyn FnMut(&mut Runtime) -> StyleProp>),
}

/// Accessibility and agent-facing properties.
pub enum SemanticProp {
    /// Accessible name.
    Label(Prop<String>),
    /// Accessible value.
    Value(Prop<String>),
    /// Longer description.
    Description(String),
    /// Overrides the role the element kind implies.
    Role(Role),
    /// Whether keyboard focus can land here.
    Focusable(bool),
    /// Hides the value from assistive technology, agents and screenshots.
    Secure,
}

/// One entry in an element's ordered modifier list.
pub enum Modifier {
    /// A literal style assignment.
    Style(StyleProp),
    /// A style assignment recomputed when the signals it reads change.
    Bound(Box<dyn FnMut(&mut Runtime) -> StyleProp>),
    /// Style assignments active while a [`Variant`] applies.
    Variant(Variant, Vec<VariantProp>),
    /// Style the node starts from when mounted, animating to its real style.
    Enter(Vec<StyleProp>),
    /// Style the node animates to when unmounted, before it disappears.
    Exit(Vec<StyleProp>),
    /// An event listener.
    Listener(Listener),
    /// Animates changes in this subtree with `curve` (CSS `transition`).
    Transition(Curve),
    /// Accessibility properties.
    Semantic(SemanticProp),
    /// Disables interaction while true.
    Disabled(Prop<bool>),
    /// A keyboard shortcut that clicks this node.
    Shortcut(Keybind),
    /// A stable identity among siblings, for ids that survive reordering.
    Key(u64),
}

impl Modifier {
    pub(crate) fn from_style(prop: Prop<StyleProp>) -> Self {
        match prop {
            Prop::Static(style) => Self::Style(style),
            Prop::Dynamic(compute) => Self::Bound(compute),
        }
    }

    pub(crate) fn from_prop<T: 'static>(prop: Prop<T>, make: fn(T) -> StyleProp) -> Self {
        match prop {
            Prop::Static(value) => Self::Style(make(value)),
            Prop::Dynamic(mut compute) => {
                Self::Bound(Box::new(move |runtime| make(compute(runtime))))
            }
        }
    }
}
