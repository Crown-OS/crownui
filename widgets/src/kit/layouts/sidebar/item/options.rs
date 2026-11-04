use view::{Element, IntoProp, View};

use super::kind::SidebarItemKind;

/// The options of a sidebar row, chained before its listeners and modifiers.
pub trait SidebarItemOptions: Sized {
    /// A Lucide icon from [`icons`](crate::kit::icons) pinned to the row's trailing edge.
    fn trailing_icon(self, body: &'static str) -> Self;
    /// A muted caption on the row's trailing edge, e.g. how many new downloads there are.
    fn trailing_text(self, caption: impl IntoProp<String>) -> Self;
    /// Any view on the row's trailing edge, after the caption: a status dot, an eject
    /// `icon_button`, a spinner.
    fn trailing(self, view: impl View) -> Self;
}

impl SidebarItemOptions for Element<SidebarItemKind> {
    fn trailing_icon(self, body: &'static str) -> Self {
        self.configure(|kind| kind.trailing_icon = Some(body))
    }

    fn trailing_text(self, caption: impl IntoProp<String>) -> Self {
        self.configure(|kind| kind.trailing_text = Some(caption.into_prop()))
    }

    fn trailing(self, view: impl View) -> Self {
        self.configure(|kind| kind.trailing = Some(view.boxed()))
    }
}
