mod bubble;
mod hover;
mod kind;
mod metrics;

pub use bubble::TooltipBubbleKind;
pub use kind::TooltipKind;

use view::{Element, IntoProp, View};

/// Wraps `content` so a small label appears under it (above it near the window's bottom edge)
/// once the pointer has rested on it for half a second; leaving or pressing hides it.
pub fn tooltip<V: View>(label: impl IntoProp<String>, content: V) -> Element<TooltipKind<V>> {
    Element::new(TooltipKind {
        label: label.into_prop(),
        content,
        start_visible: false,
    })
}

/// A tooltip that starts out showing, for previews.
pub(crate) fn shown_tooltip<V: View>(
    label: impl IntoProp<String>,
    content: V,
) -> Element<TooltipKind<V>> {
    tooltip(label, content).configure(|kind| kind.start_visible = true)
}
