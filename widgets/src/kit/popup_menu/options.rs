use view::Element;

use super::kind::PopupMenuKind;

/// The options of a [`popup_menu`](super::popup_menu), chained before its listeners and modifiers.
pub trait PopupMenuOptions: Sized {
    /// Mounts the menu with its panel already open: for previews and documentation.
    fn start_open(self) -> Self;
}

impl PopupMenuOptions for Element<PopupMenuKind> {
    fn start_open(self) -> Self {
        self.configure(|kind| kind.start_open = true)
    }
}
