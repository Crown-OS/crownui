use view::Element;

use super::kind::KeybindInputKind;

/// The options of a [`keybind_input`](super::keybind_input), chained before its listeners.
pub trait KeybindInputOptions: Sized {
    /// The field's width in logical pixels; the default is 252, a select trigger's.
    fn width(self, width: f32) -> Self;
}

impl KeybindInputOptions for Element<KeybindInputKind> {
    fn width(self, width: f32) -> Self {
        self.configure(|kind| kind.width = width)
    }
}
