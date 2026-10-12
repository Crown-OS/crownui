use view::{Element, IntoProp};

use super::kind::CalloutKind;

/// The options of a [`callout`](super::callout), chained before its modifiers.
pub trait CalloutOptions: Sized {
    /// The explanatory paragraph under the title; it wraps.
    fn body(self, body: impl IntoProp<String>) -> Self;

    /// A Lucide icon from [`icons`](crate::kit::icons) replacing the glyph the tone would choose.
    fn icon(self, body: &'static str) -> Self;
}

impl CalloutOptions for Element<CalloutKind> {
    fn body(self, body: impl IntoProp<String>) -> Self {
        let body = body.into_prop();
        self.configure(|kind| kind.body = Some(body))
    }

    fn icon(self, body: &'static str) -> Self {
        self.configure(|kind| kind.icon = Some(body))
    }
}
