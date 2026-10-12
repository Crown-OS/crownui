use view::{Element, IntoProp};

use super::kind::ToggleKind;

/// The options of a [`toggle`](super::toggle), chained before its listeners and modifiers.
pub trait ToggleOptions: Sized {
    /// Text drawn beside the track in the body color; it also names the switch.
    fn caption(self, caption: impl IntoProp<String>) -> Self;
}

impl ToggleOptions for Element<ToggleKind> {
    fn caption(self, caption: impl IntoProp<String>) -> Self {
        self.configure(|kind| kind.caption = Some(caption.into_prop()))
    }
}
