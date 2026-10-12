use view::{Element, Semantic, Styled};

use super::{
    FieldShape,
    kind::{FieldWidth, TextInputKind},
};

/// Whether a field offers a trailing button that empties it while it holds text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ClearButton {
    /// No clear button.
    #[default]
    Hidden,
    /// A clear button, shown while there is something to clear.
    Visible,
}

/// The options of a [`text_input`](super::text_input) and of widgets built on one, chained before
/// listeners and modifiers.
pub trait TextInputOptions: Styled {
    /// Edits the underlying field's configuration.
    fn configure_input(self, change: impl FnOnce(&mut TextInputKind)) -> Self;

    /// Muted text shown while the field is empty; it also names the field for assistive
    /// technology unless a `.label(..)` follows.
    fn placeholder(self, placeholder: impl Into<String>) -> Self {
        let placeholder = placeholder.into();
        self.configure_input(|kind| kind.placeholder.clone_from(&placeholder))
            .label(placeholder)
    }

    /// Corner geometry; the default is [`FieldShape::Rounded`].
    fn shape(self, shape: FieldShape) -> Self {
        self.configure_input(|kind| kind.shape = shape)
    }

    /// A muted Lucide icon from [`icons`](crate::kit::icons) at the leading edge, with the text
    /// pushed clear of it.
    fn leading_icon(self, body: &'static str) -> Self {
        self.configure_input(|kind| kind.leading_icon = Some(body))
    }

    /// Whether a trailing button empties the field.
    fn clear_button(self, clear: ClearButton) -> Self {
        self.configure_input(|kind| kind.clear_button = clear)
    }

    /// A fixed width in logical pixels; the default is 252, a select trigger's.
    fn width(self, width: f32) -> Self {
        self.configure_input(|kind| kind.width = FieldWidth::Fixed(width))
    }

    /// Takes the whole width the parent offers.
    fn stretch(self) -> Self {
        self.configure_input(|kind| kind.width = FieldWidth::Fill)
    }

    /// Shows one bullet per character while the value stays real, and hides it from assistive
    /// technology, agents and the clipboard.
    fn secret(self) -> Self {
        self.configure_input(|kind| kind.secret = true).secure()
    }

    /// Keeps the value selectable and copyable but refuses every edit.
    fn read_only(self) -> Self {
        self.configure_input(|kind| kind.read_only = true)
    }
}

impl TextInputOptions for Element<TextInputKind> {
    fn configure_input(self, change: impl FnOnce(&mut TextInputKind)) -> Self {
        self.configure(change)
    }
}
