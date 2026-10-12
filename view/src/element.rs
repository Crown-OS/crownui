use smallvec::SmallVec;

use crate::{
    Cx, NodeId,
    modifier::{Modifier, Variant, VariantProp},
    style_prop::StyleProp,
    styled::Styled,
    view::View,
};

/// What kind of element a node is: implemented by every widget and by custom components that
/// need their own content, children or painting. `mount` runs once, after the node exists and
/// before the element's modifiers are applied.
pub trait Kind: 'static {
    /// Name used in component paths and diagnostics (`button`, `vstack` …).
    const NAME: &'static str;

    /// Sets the node's role, content, default style and children.
    fn mount(self, cx: &mut Cx, node: NodeId);
}

/// A kind plus its ordered modifiers; the type every widget function returns.
pub struct Element<K: Kind> {
    kind: K,
    modifiers: SmallVec<[Modifier; 6]>,
}

impl<K: Kind> Element<K> {
    /// Wraps a kind with no modifiers.
    pub fn new(kind: K) -> Self {
        Self {
            kind,
            modifiers: SmallVec::new(),
        }
    }

    /// Adjusts the kind before it is mounted; component libraries build their option methods on it.
    pub fn configure(mut self, change: impl FnOnce(&mut K)) -> Self {
        change(&mut self.kind);
        self
    }

    fn variant(self, variant: Variant, style: impl FnOnce(VariantStyle) -> VariantStyle) -> Self {
        let props = style(VariantStyle::default()).props;
        self.modifier(Modifier::Variant(variant, props))
    }

    /// `hover:` styles while the pointer is over the element.
    pub fn hover(self, style: impl FnOnce(VariantStyle) -> VariantStyle) -> Self {
        self.variant(Variant::Hover, style)
    }

    /// `active:` styles while the element is pressed.
    pub fn active(self, style: impl FnOnce(VariantStyle) -> VariantStyle) -> Self {
        self.variant(Variant::Active, style)
    }

    /// `focus:` styles while the element has keyboard focus.
    pub fn focus(self, style: impl FnOnce(VariantStyle) -> VariantStyle) -> Self {
        self.variant(Variant::Focus, style)
    }

    /// `focus-visible:` styles while focus arrived by keyboard.
    pub fn focus_visible(self, style: impl FnOnce(VariantStyle) -> VariantStyle) -> Self {
        self.variant(Variant::FocusVisible, style)
    }

    /// `disabled:` styles while the element is disabled.
    pub fn when_disabled(self, style: impl FnOnce(VariantStyle) -> VariantStyle) -> Self {
        self.variant(Variant::Disabled, style)
    }

    /// `dark:` styles while the color scheme is dark.
    pub fn dark(self, style: impl FnOnce(VariantStyle) -> VariantStyle) -> Self {
        self.variant(Variant::Dark, style)
    }

    /// CSS `transition`: animates every change in this subtree with `curve`.
    pub fn transition(self, curve: impl Into<motion::Curve>) -> Self {
        self.modifier(Modifier::Transition(curve.into()))
    }

    /// The style the element starts from when it appears, animating to its real style.
    pub fn enter(self, style: impl FnOnce(VariantStyle) -> VariantStyle) -> Self {
        let props = style(VariantStyle::default()).literals();
        self.modifier(Modifier::Enter(props))
    }

    /// The style the element animates to when it is removed (it stays visible until then).
    pub fn exit(self, style: impl FnOnce(VariantStyle) -> VariantStyle) -> Self {
        let props = style(VariantStyle::default()).literals();
        self.modifier(Modifier::Exit(props))
    }

    /// A stable identity among siblings.
    pub fn key(self, key: impl std::hash::Hash) -> Self {
        use std::hash::Hasher;
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        key.hash(&mut hasher);
        self.modifier(Modifier::Key(hasher.finish()))
    }
}

impl<K: Kind> Styled for Element<K> {
    fn modifier(mut self, modifier: Modifier) -> Self {
        self.modifiers.push(modifier);
        self
    }
}

impl<K: Kind> View for Element<K> {
    fn build(self, cx: &mut Cx) -> NodeId {
        cx.mount_element(self.kind, self.modifiers)
    }
}

/// Collects the styles of a state variant. Paint, composite and text properties take effect, as
/// literals or bound to signals (`.bg(bind(..))`); layout properties are ignored (CUI0201).
#[derive(Default)]
pub struct VariantStyle {
    props: Vec<VariantProp>,
}

impl VariantStyle {
    fn literals(self) -> Vec<StyleProp> {
        self.props
            .into_iter()
            .filter_map(|prop| match prop {
                VariantProp::Literal(prop) => Some(prop),
                VariantProp::Bound(_) => {
                    tracing::warn!("CUI0202: enter/exit styles take literal values only");
                    None
                }
            })
            .collect()
    }
}

impl Styled for VariantStyle {
    fn modifier(mut self, modifier: Modifier) -> Self {
        match modifier {
            Modifier::Style(prop) if prop.is_visual() || prop.impact().text => {
                self.props.push(VariantProp::Literal(prop))
            }
            Modifier::Style(prop) => tracing::warn!(
                ?prop,
                "CUI0201: layout properties are ignored in style variants"
            ),
            Modifier::Bound(compute) => self.props.push(VariantProp::Bound(compute)),
            _ => tracing::warn!("CUI0202: only style properties are allowed in style variants"),
        }
        self
    }
}
