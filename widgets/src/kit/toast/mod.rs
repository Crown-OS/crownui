mod host;
mod metrics;
mod options;
mod pill;
mod toasts;

pub use host::ToastHostKind;
pub use options::ToastOptions;
pub use pill::{ToastKind, toast};
pub use toasts::{Toast, Toasts, provide_toasts, use_toasts};

use view::{Element, Styled, View};

/// Wraps the window's `content` and shows the [`Toasts`] raised under it bottom-center, above
/// everything, each leaving on its own after its duration. Make it the root, or a view that
/// fills the window.
pub fn toast_host<V: View>(content: V) -> Element<ToastHostKind<V>> {
    Element::new(ToastHostKind { content }).flex_col()
}
