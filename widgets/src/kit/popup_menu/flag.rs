use reactive::{Runtime, Signal};
use view::{IntoProp, Prop, bind};

/// A yes/no property of a menu row, fixed or following a signal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuFlag {
    /// Decided when the menu is built.
    Fixed(bool),
    /// Follows the signal while the menu is open.
    Signal(Signal<bool>),
}

impl MenuFlag {
    pub(super) fn resolve(self, runtime: &mut Runtime) -> bool {
        match self {
            Self::Fixed(value) => value,
            Self::Signal(signal) => signal.get(runtime),
        }
    }
}

impl From<bool> for MenuFlag {
    fn from(value: bool) -> Self {
        Self::Fixed(value)
    }
}

impl From<Signal<bool>> for MenuFlag {
    fn from(signal: Signal<bool>) -> Self {
        Self::Signal(signal)
    }
}

impl IntoProp<bool> for MenuFlag {
    fn into_prop(self) -> Prop<bool> {
        match self {
            Self::Fixed(value) => Prop::Static(value),
            Self::Signal(signal) => bind(move |runtime| signal.get(runtime)).into_prop(),
        }
    }
}
