use crownconfig::{KeyCode, Keybind, Mods};
use input::{Key, KeyboardEvent, Modifiers, NamedKey};

use super::physical_key::{UNKNOWN_PHYSICAL_KEY, w3c_code};

pub(crate) fn modifier_of(key: Key) -> Option<NamedKey> {
    match key {
        Key::Named(
            named @ (NamedKey::Shift | NamedKey::Control | NamedKey::Alt | NamedKey::Meta),
        ) => Some(named),
        _ => None,
    }
}

pub(crate) fn mods_of(modifier: NamedKey) -> Mods {
    Mods {
        meta: modifier == NamedKey::Meta,
        ctrl: modifier == NamedKey::Control,
        alt: modifier == NamedKey::Alt,
        shift: modifier == NamedKey::Shift,
    }
}

pub(crate) fn from_toolkit(modifiers: Modifiers) -> Mods {
    Mods {
        meta: modifiers.meta_key,
        ctrl: modifiers.ctrl_key,
        alt: modifiers.alt_key,
        shift: modifiers.shift_key,
    }
}

pub(crate) fn merge(first: Mods, second: Mods) -> Mods {
    Mods {
        meta: first.meta || second.meta,
        ctrl: first.ctrl || second.ctrl,
        alt: first.alt || second.alt,
        shift: first.shift || second.shift,
    }
}

pub(crate) fn key_code(event: &KeyboardEvent) -> Option<KeyCode> {
    if event.code != UNKNOWN_PHYSICAL_KEY {
        return w3c_code(event.code).and_then(KeyCode::from_code);
    }
    match event.key {
        Key::Named(named) => KeyCode::from_code(&named.dom_name()),
        Key::Character(character) => KeyCode::from_code(&character_code(character)?),
        Key::Unidentified => None,
    }
}

fn character_code(character: char) -> Option<String> {
    let punctuation = match character {
        '-' => "Minus",
        '=' => "Equal",
        '[' => "BracketLeft",
        ']' => "BracketRight",
        '\\' => "Backslash",
        ';' => "Semicolon",
        '\'' => "Quote",
        '`' => "Backquote",
        ',' => "Comma",
        '.' => "Period",
        '/' => "Slash",
        ' ' => "Space",
        letter if letter.is_ascii_alphabetic() => {
            return Some(format!("Key{}", letter.to_ascii_uppercase()));
        }
        digit if digit.is_ascii_digit() => return Some(format!("Digit{digit}")),
        _ => return None,
    };
    Some(punctuation.to_owned())
}

pub(crate) fn written(bind: Keybind) -> String {
    if bind.is_empty() {
        String::new()
    } else {
        bind.to_string()
    }
}

#[cfg(test)]
mod tests {
    use input::PhysicalKey;

    use super::*;

    fn event(key: Key, code: u32) -> KeyboardEvent {
        KeyboardEvent {
            key,
            code: PhysicalKey(code),
            text: None,
            repeat: false,
            modifiers: Modifiers::NONE,
        }
    }

    #[test]
    fn a_physical_key_wins_over_what_the_layout_made_of_it() {
        let dvorak_k_on_the_qwerty_t_key = event(Key::Character('k'), 20);
        assert_eq!(key_code(&dvorak_k_on_the_qwerty_t_key), Some(KeyCode::T));
    }

    #[test]
    fn a_key_outside_the_shortcut_list_is_not_a_near_miss() {
        assert_eq!(key_code(&event(Key::Named(NamedKey::Enter), 96)), None);
        assert_eq!(key_code(&event(Key::Unidentified, 240)), None);
    }

    #[test]
    fn a_synthetic_event_falls_back_on_the_logical_key() {
        assert_eq!(key_code(&event(Key::Character('K'), 0)), Some(KeyCode::K));
        assert_eq!(
            key_code(&event(Key::Character('7'), 0)),
            Some(KeyCode::Digit7)
        );
        assert_eq!(
            key_code(&event(Key::Character('['), 0)),
            Some(KeyCode::LeftBracket)
        );
        assert_eq!(
            key_code(&event(Key::Named(NamedKey::ArrowLeft), 0)),
            Some(KeyCode::ArrowLeft)
        );
        assert_eq!(
            key_code(&event(Key::Named(NamedKey::F(5)), 0)),
            Some(KeyCode::F5)
        );
        assert_eq!(key_code(&event(Key::Named(NamedKey::F(13)), 0)), None);
        assert_eq!(key_code(&event(Key::Character('é'), 0)), None);
    }

    #[test]
    fn only_the_four_modifier_keys_are_modifiers() {
        assert_eq!(
            modifier_of(Key::Named(NamedKey::Meta)),
            Some(NamedKey::Meta)
        );
        assert_eq!(modifier_of(Key::Named(NamedKey::CapsLock)), None);
        assert_eq!(modifier_of(Key::Character('a')), None);
        assert_eq!(mods_of(NamedKey::Meta), Mods::META);
    }

    #[test]
    fn modifier_sets_merge_and_translate() {
        let held = Modifiers {
            ctrl_key: true,
            ..Modifiers::NONE
        };
        let merged = merge(from_toolkit(held), Mods::META);
        assert!(merged.ctrl && merged.meta && !merged.alt && !merged.shift);
    }

    #[test]
    fn a_chord_is_written_the_way_the_config_file_spells_it() {
        assert_eq!(written(Keybind::SUPER_SPACE), "Super+Space");
        assert_eq!(written(Keybind::NONE), "");
    }
}
