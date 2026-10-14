use std::{fmt, str::FromStr};

use crate::{Key, KeyboardEvent, Modifiers, NamedKey};

/// A keyboard shortcut such as `"Ctrl+Shift+K"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Keybind {
    /// The non-modifier key.
    pub key: Key,
    /// Modifiers that must be held, and no others.
    pub modifiers: Modifiers,
}

/// Why a shortcut string could not be parsed.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum KeybindParseError {
    /// The string held only modifiers.
    #[error("shortcut {0:?} has no key besides modifiers")]
    MissingKey(String),
    /// A part was neither a modifier nor a key name.
    #[error("unknown key {0:?} in shortcut")]
    UnknownKey(String),
}

impl Keybind {
    /// Whether `event` triggers this shortcut; letters match case-insensitively.
    pub fn matches(&self, event: &KeyboardEvent) -> bool {
        let same_key = match (self.key, event.key) {
            (Key::Character(expected), Key::Character(pressed)) => {
                expected.eq_ignore_ascii_case(&pressed)
            }
            (expected, pressed) => expected == pressed,
        };
        same_key && self.modifiers == event.modifiers
    }
}

impl FromStr for Keybind {
    type Err = KeybindParseError;

    fn from_str(shortcut: &str) -> Result<Self, Self::Err> {
        let mut modifiers = Modifiers::NONE;
        let mut key = None;
        for part in shortcut.split('+').map(str::trim) {
            match part.to_ascii_lowercase().as_str() {
                "ctrl" | "control" => modifiers.ctrl_key = true,
                "shift" => modifiers.shift_key = true,
                "alt" | "option" => modifiers.alt_key = true,
                "meta" | "super" | "cmd" | "win" => modifiers.meta_key = true,
                _ => key = Some(parse_key(part)?),
            }
        }
        let key = key.ok_or_else(|| KeybindParseError::MissingKey(shortcut.into()))?;
        Ok(Self { key, modifiers })
    }
}

fn parse_key(name: &str) -> Result<Key, KeybindParseError> {
    let mut characters = name.chars();
    match (characters.next(), characters.next()) {
        (Some(character), None) => Ok(Key::Character(character.to_ascii_lowercase())),
        _ => NamedKey::from_dom_name(name)
            .map(Key::Named)
            .ok_or_else(|| KeybindParseError::UnknownKey(name.into())),
    }
}

impl fmt::Display for Keybind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let held = [
            (self.modifiers.ctrl_key, "Ctrl+"),
            (self.modifiers.alt_key, "Alt+"),
            (self.modifiers.shift_key, "Shift+"),
            (self.modifiers.meta_key, "Meta+"),
        ];
        for (_, label) in held.iter().filter(|(is_held, _)| *is_held) {
            formatter.write_str(label)?;
        }
        match self.key {
            Key::Character(character) => write!(formatter, "{}", character.to_ascii_uppercase()),
            key => formatter.write_str(&key.dom_name()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PhysicalKey;

    #[test]
    fn shortcuts_round_trip_through_text() {
        let Ok(keybind) = "shift+ctrl+k".parse::<Keybind>() else {
            panic!("valid shortcut");
        };
        assert_eq!(keybind.to_string(), "Ctrl+Shift+K");
        assert_eq!(
            "F5".parse::<Keybind>().map(|bind| bind.key),
            Ok(Key::Named(NamedKey::F(5)))
        );
    }

    #[test]
    fn modifier_only_and_unknown_shortcuts_are_rejected() {
        assert!(matches!(
            "Ctrl+Shift".parse::<Keybind>(),
            Err(KeybindParseError::MissingKey(_))
        ));
        assert!(matches!(
            "Ctrl+Banana".parse::<Keybind>(),
            Err(KeybindParseError::UnknownKey(_))
        ));
    }

    #[test]
    fn matching_ignores_letter_case_but_not_modifiers() {
        let Ok(save) = "Ctrl+S".parse::<Keybind>() else {
            panic!("valid shortcut");
        };
        let mut event = KeyboardEvent {
            key: Key::Character('S'),
            code: PhysicalKey(31),
            text: None,
            repeat: false,
            modifiers: Modifiers {
                ctrl_key: true,
                ..Modifiers::NONE
            },
        };
        assert!(save.matches(&event));
        event.modifiers.shift_key = true;
        assert!(!save.matches(&event));
    }
}
