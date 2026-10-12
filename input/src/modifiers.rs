/// Held modifier keys, as in the DOM's `shiftKey`, `ctrlKey`, `altKey` and `metaKey`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Modifiers {
    /// Shift.
    pub shift_key: bool,
    /// Control.
    pub ctrl_key: bool,
    /// Alt / Option.
    pub alt_key: bool,
    /// Super / Command / Windows.
    pub meta_key: bool,
}

impl Modifiers {
    /// No modifier held.
    pub const NONE: Self = Self {
        shift_key: false,
        ctrl_key: false,
        alt_key: false,
        meta_key: false,
    };

    /// Whether no modifier is held.
    pub fn is_empty(self) -> bool {
        self == Self::NONE
    }
}
