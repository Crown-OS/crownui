use view::Element;

use super::kind::SidebarItemKind;

/// The options of a sidebar row, chained before its listeners and modifiers.
pub trait SidebarItemOptions: Sized {
    /// A Lucide icon from [`icons`](crate::kit::icons) pinned to the row's trailing edge.
    fn trailing_icon(self, body: &'static str) -> Self;
}

impl SidebarItemOptions for Element<SidebarItemKind> {
    fn trailing_icon(self, body: &'static str) -> Self {
        self.configure(|kind| kind.trailing_icon = Some(body))
    }
}
