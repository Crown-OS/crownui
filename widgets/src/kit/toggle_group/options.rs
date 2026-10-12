use view::Element;

use super::kind::ToggleGroupKind;

/// The options of a [`toggle_group`](super::toggle_group), chained before its listeners.
pub trait ToggleGroupOptions: Sized {
    /// Shares the whole width the parent offers between the segments.
    fn stretch(self) -> Self;
}

impl ToggleGroupOptions for Element<ToggleGroupKind> {
    fn stretch(self) -> Self {
        self.configure(|kind| kind.stretch = true)
    }
}
