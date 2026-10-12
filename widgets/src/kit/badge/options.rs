use view::Element;

use super::kind::BadgeKind;

/// The options of a [`badge`](super::badge), chained before its modifiers.
pub trait BadgeOptions: Sized {
    /// A Lucide icon from [`icons`](crate::kit::icons) drawn before the label, in the label's color.
    fn leading_icon(self, body: &'static str) -> Self;
}

impl BadgeOptions for Element<BadgeKind> {
    fn leading_icon(self, body: &'static str) -> Self {
        self.configure(|kind| kind.icon = Some(body))
    }
}
