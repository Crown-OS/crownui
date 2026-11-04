use view::Element;

use super::pill::ToastKind;

/// The options of a [`toast`](super::toast), chained before its modifiers.
pub trait ToastOptions: Sized {
    /// A Lucide icon from [`icons`](crate::kit::icons) drawn before the message.
    fn leading_icon(self, body: &'static str) -> Self;
}

impl ToastOptions for Element<ToastKind> {
    fn leading_icon(self, body: &'static str) -> Self {
        self.configure(|kind| kind.icon = Some(body))
    }
}
