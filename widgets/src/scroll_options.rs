use view::ScrollHandle;

/// Options shared by the scrolling components, chained before their modifiers.
pub trait ScrollOptions: Sized {
    /// Lets app code read and drive the scroll position through `handle`, e.g.
    /// `handle.ensure_visible(cx, index)` after an arrow key.
    fn scroll_handle(self, handle: ScrollHandle) -> Self;
}
