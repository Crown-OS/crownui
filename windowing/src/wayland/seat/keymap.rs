use input::{Key, Modifiers, NamedKey, PointerButton};
use sctk::seat::{keyboard::Keysym, pointer::CursorIcon as SctkCursorIcon};

use crate::CursorIcon;

pub(crate) fn key_from_keysym(keysym: Keysym) -> Key {
    let named = match keysym {
        Keysym::Return | Keysym::KP_Enter => NamedKey::Enter,
        Keysym::Tab | Keysym::ISO_Left_Tab => NamedKey::Tab,
        Keysym::BackSpace => NamedKey::Backspace,
        Keysym::Delete | Keysym::KP_Delete => NamedKey::Delete,
        Keysym::Escape => NamedKey::Escape,
        Keysym::Up | Keysym::KP_Up => NamedKey::ArrowUp,
        Keysym::Down | Keysym::KP_Down => NamedKey::ArrowDown,
        Keysym::Left | Keysym::KP_Left => NamedKey::ArrowLeft,
        Keysym::Right | Keysym::KP_Right => NamedKey::ArrowRight,
        Keysym::Home | Keysym::KP_Home => NamedKey::Home,
        Keysym::End | Keysym::KP_End => NamedKey::End,
        Keysym::Prior | Keysym::KP_Prior => NamedKey::PageUp,
        Keysym::Next | Keysym::KP_Next => NamedKey::PageDown,
        Keysym::Insert | Keysym::KP_Insert => NamedKey::Insert,
        Keysym::Shift_L | Keysym::Shift_R => NamedKey::Shift,
        Keysym::Control_L | Keysym::Control_R => NamedKey::Control,
        Keysym::Alt_L | Keysym::Alt_R | Keysym::ISO_Level3_Shift => NamedKey::Alt,
        Keysym::Super_L | Keysym::Super_R | Keysym::Meta_L | Keysym::Meta_R => NamedKey::Meta,
        Keysym::Caps_Lock => NamedKey::CapsLock,
        Keysym::Menu => NamedKey::ContextMenu,
        Keysym::Print => NamedKey::PrintScreen,
        Keysym::space => NamedKey::Space,
        other => {
            let raw = other.raw();
            if (Keysym::F1.raw()..=Keysym::F24.raw()).contains(&raw) {
                let number = u8::try_from(raw - Keysym::F1.raw() + 1).unwrap_or(1);
                return Key::Named(NamedKey::F(number));
            }
            return other.key_char().map_or(Key::Unidentified, Key::Character);
        }
    };
    Key::Named(named)
}

pub(crate) fn modifiers_from_sctk(modifiers: sctk::seat::keyboard::Modifiers) -> Modifiers {
    Modifiers {
        shift_key: modifiers.shift,
        ctrl_key: modifiers.ctrl,
        alt_key: modifiers.alt,
        meta_key: modifiers.logo,
    }
}

pub(crate) fn pointer_button_from_evdev(code: u32) -> PointerButton {
    const BTN_LEFT: u32 = 0x110;
    const BTN_RIGHT: u32 = 0x111;
    const BTN_MIDDLE: u32 = 0x112;
    const BTN_SIDE: u32 = 0x113;
    const BTN_EXTRA: u32 = 0x114;
    match code {
        BTN_LEFT => PointerButton::Primary,
        BTN_RIGHT => PointerButton::Secondary,
        BTN_MIDDLE => PointerButton::Auxiliary,
        BTN_SIDE => PointerButton::Back,
        BTN_EXTRA => PointerButton::Forward,
        other => PointerButton::Other(other),
    }
}

pub(crate) fn sctk_cursor(icon: CursorIcon) -> SctkCursorIcon {
    match icon {
        CursorIcon::Default => SctkCursorIcon::Default,
        CursorIcon::Pointer => SctkCursorIcon::Pointer,
        CursorIcon::Text => SctkCursorIcon::Text,
        CursorIcon::Grab => SctkCursorIcon::Grab,
        CursorIcon::Grabbing => SctkCursorIcon::Grabbing,
        CursorIcon::Move => SctkCursorIcon::Move,
        CursorIcon::NotAllowed => SctkCursorIcon::NotAllowed,
        CursorIcon::Wait => SctkCursorIcon::Wait,
        CursorIcon::Progress => SctkCursorIcon::Progress,
        CursorIcon::Crosshair => SctkCursorIcon::Crosshair,
        CursorIcon::EwResize => SctkCursorIcon::EwResize,
        CursorIcon::NsResize => SctkCursorIcon::NsResize,
        CursorIcon::NeswResize => SctkCursorIcon::NeswResize,
        CursorIcon::NwseResize => SctkCursorIcon::NwseResize,
        CursorIcon::ColResize => SctkCursorIcon::ColResize,
        CursorIcon::RowResize => SctkCursorIcon::RowResize,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keysyms_map_to_dom_keys() {
        assert_eq!(key_from_keysym(Keysym::Return), Key::Named(NamedKey::Enter));
        assert_eq!(key_from_keysym(Keysym::F5), Key::Named(NamedKey::F(5)));
        assert_eq!(key_from_keysym(Keysym::a), Key::Character('a'));
    }
}
