use view::{Element, View};

use crate::scroll_view::{ScrollViewKind, scroll_view};

/// Makes a step's content scroll, so a step taller than the window never pushes the button bar
/// off the bottom.
pub fn wizard_scroll<Content: View>(content: Content) -> Element<ScrollViewKind<Content>> {
    scroll_view(content)
}
