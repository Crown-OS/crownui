use crate::{Key, Modifiers, PhysicalKey};

/// A key press or release, like the DOM's `KeyboardEvent`.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KeyboardEvent {
    /// The logical key after layout and modifiers.
    pub key: Key,
    /// The physical key position.
    pub code: PhysicalKey,
    /// Text the press produces, if any.
    pub text: Option<String>,
    /// Whether this press comes from key repeat.
    pub repeat: bool,
    /// Modifiers held at the time.
    pub modifiers: Modifiers,
}
