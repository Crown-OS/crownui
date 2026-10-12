use semantics::Role;

use crate::{
    modifier::{Modifier, SemanticProp},
    prop::IntoProp,
    styled::Styled,
};

/// Accessibility properties, read by screen readers, MCP agents and `crowntest` queries.
pub trait Semantic: Styled {
    /// `aria-label`: the accessible name.
    fn label(self, label: impl IntoProp<String>) -> Self {
        self.modifier(Modifier::Semantic(SemanticProp::Label(label.into_prop())))
    }

    /// The accessible value.
    fn value(self, value: impl IntoProp<String>) -> Self {
        self.modifier(Modifier::Semantic(SemanticProp::Value(value.into_prop())))
    }

    /// `aria-description`.
    fn description(self, description: impl Into<String>) -> Self {
        self.modifier(Modifier::Semantic(SemanticProp::Description(description.into())))
    }

    /// Overrides the ARIA role implied by the element.
    fn role(self, role: Role) -> Self {
        self.modifier(Modifier::Semantic(SemanticProp::Role(role)))
    }

    /// `tabindex="0"` / `"-1"`: whether keyboard focus can land here.
    fn focusable(self, focusable: bool) -> Self {
        self.modifier(Modifier::Semantic(SemanticProp::Focusable(focusable)))
    }

    /// Hides the value from assistive technology, agents and screenshots.
    fn secure(self) -> Self {
        self.modifier(Modifier::Semantic(SemanticProp::Secure))
    }

    /// `disabled`: blocks interaction and activates the `disabled` variant.
    fn disabled(self, disabled: impl IntoProp<bool>) -> Self {
        self.modifier(Modifier::Disabled(disabled.into_prop()))
    }
}

impl<T: Styled> Semantic for T {}
