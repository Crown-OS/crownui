/// The logical key, like the DOM's `KeyboardEvent.key`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Key {
    /// A key that produces a character, after the keyboard layout and modifiers.
    Character(char),
    /// A named, non-printing key.
    Named(NamedKey),
    /// A key the backend could not identify.
    Unidentified,
}

/// Non-printing keys, named as in the DOM `key` values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[allow(missing_docs)]
pub enum NamedKey {
    Enter,
    Tab,
    Space,
    Backspace,
    Delete,
    Escape,
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    Home,
    End,
    PageUp,
    PageDown,
    Insert,
    Shift,
    Control,
    Alt,
    Meta,
    CapsLock,
    ContextMenu,
    PrintScreen,
    /// Function keys `F1` to `F24`.
    F(u8),
}

/// The physical key position, like the DOM's `KeyboardEvent.code`; holds the
/// platform scancode (evdev on Linux).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PhysicalKey(pub u32);

impl Key {
    /// The DOM `key` string, e.g. `"a"`, `"Enter"`, `"F5"`.
    pub fn dom_name(self) -> String {
        match self {
            Self::Character(' ') => "Space".into(),
            Self::Character(character) => character.to_string(),
            Self::Named(named) => named.dom_name(),
            Self::Unidentified => "Unidentified".into(),
        }
    }
}

impl NamedKey {
    /// The DOM `key` string.
    pub fn dom_name(self) -> String {
        match self {
            Self::F(number) => format!("F{number}"),
            Self::Space => "Space".into(),
            other => format!("{other:?}"),
        }
    }

    /// Parses a DOM `key` name, case-insensitively.
    pub fn from_dom_name(name: &str) -> Option<Self> {
        const NAMED: [NamedKey; 22] = [
            NamedKey::Enter,
            NamedKey::Tab,
            NamedKey::Space,
            NamedKey::Backspace,
            NamedKey::Delete,
            NamedKey::Escape,
            NamedKey::ArrowUp,
            NamedKey::ArrowDown,
            NamedKey::ArrowLeft,
            NamedKey::ArrowRight,
            NamedKey::Home,
            NamedKey::End,
            NamedKey::PageUp,
            NamedKey::PageDown,
            NamedKey::Insert,
            NamedKey::Shift,
            NamedKey::Control,
            NamedKey::Alt,
            NamedKey::Meta,
            NamedKey::CapsLock,
            NamedKey::ContextMenu,
            NamedKey::PrintScreen,
        ];
        if let Some(number) = name
            .strip_prefix(['F', 'f'])
            .and_then(|digits| digits.parse::<u8>().ok())
            .filter(|number| (1..=24).contains(number))
        {
            return Some(Self::F(number));
        }
        NAMED
            .into_iter()
            .find(|named| named.dom_name().eq_ignore_ascii_case(name))
    }
}
