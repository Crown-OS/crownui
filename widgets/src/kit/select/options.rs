use view::Element;

use super::kind::SelectKind;

/// The options of a [`select`](super::select), chained before its listeners and modifiers.
pub trait SelectOptions: Sized {
    /// The muted text shown while nothing is selected.
    fn placeholder(self, placeholder: impl Into<String>) -> Self;

    /// Mounts the select with its panel already open: for previews and documentation.
    fn start_open(self) -> Self;
}

impl SelectOptions for Element<SelectKind> {
    fn placeholder(self, placeholder: impl Into<String>) -> Self {
        self.configure(|kind| kind.placeholder = placeholder.into())
    }

    fn start_open(self) -> Self {
        self.configure(|kind| kind.start_open = true)
    }
}
