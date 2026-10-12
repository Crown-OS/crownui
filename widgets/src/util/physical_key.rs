use input::PhysicalKey;

const EVDEV_TO_W3C_CODE: [(u32, &str); 76] = [
    (1, "Escape"),
    (2, "Digit1"),
    (3, "Digit2"),
    (4, "Digit3"),
    (5, "Digit4"),
    (6, "Digit5"),
    (7, "Digit6"),
    (8, "Digit7"),
    (9, "Digit8"),
    (10, "Digit9"),
    (11, "Digit0"),
    (12, "Minus"),
    (13, "Equal"),
    (14, "Backspace"),
    (15, "Tab"),
    (16, "KeyQ"),
    (17, "KeyW"),
    (18, "KeyE"),
    (19, "KeyR"),
    (20, "KeyT"),
    (21, "KeyY"),
    (22, "KeyU"),
    (23, "KeyI"),
    (24, "KeyO"),
    (25, "KeyP"),
    (26, "BracketLeft"),
    (27, "BracketRight"),
    (28, "Enter"),
    (30, "KeyA"),
    (31, "KeyS"),
    (32, "KeyD"),
    (33, "KeyF"),
    (34, "KeyG"),
    (35, "KeyH"),
    (36, "KeyJ"),
    (37, "KeyK"),
    (38, "KeyL"),
    (39, "Semicolon"),
    (40, "Quote"),
    (41, "Backquote"),
    (43, "Backslash"),
    (44, "KeyZ"),
    (45, "KeyX"),
    (46, "KeyC"),
    (47, "KeyV"),
    (48, "KeyB"),
    (49, "KeyN"),
    (50, "KeyM"),
    (51, "Comma"),
    (52, "Period"),
    (53, "Slash"),
    (57, "Space"),
    (58, "CapsLock"),
    (59, "F1"),
    (60, "F2"),
    (61, "F3"),
    (62, "F4"),
    (63, "F5"),
    (64, "F6"),
    (65, "F7"),
    (66, "F8"),
    (67, "F9"),
    (68, "F10"),
    (87, "F11"),
    (88, "F12"),
    (96, "NumpadEnter"),
    (102, "Home"),
    (103, "ArrowUp"),
    (104, "PageUp"),
    (105, "ArrowLeft"),
    (106, "ArrowRight"),
    (107, "End"),
    (108, "ArrowDown"),
    (109, "PageDown"),
    (110, "Insert"),
    (111, "Delete"),
];

pub(crate) const UNKNOWN_PHYSICAL_KEY: PhysicalKey = PhysicalKey(0);

pub(crate) fn w3c_code(key: PhysicalKey) -> Option<&'static str> {
    EVDEV_TO_W3C_CODE
        .iter()
        .find_map(|(evdev, code)| (*evdev == key.0).then_some(*code))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evdev_scancodes_name_the_w3c_keys_they_sit_under() {
        assert_eq!(w3c_code(PhysicalKey(30)), Some("KeyA"));
        assert_eq!(w3c_code(PhysicalKey(57)), Some("Space"));
        assert_eq!(w3c_code(PhysicalKey(105)), Some("ArrowLeft"));
        assert_eq!(w3c_code(PhysicalKey(26)), Some("BracketLeft"));
        assert_eq!(w3c_code(PhysicalKey(96)), Some("NumpadEnter"));
        assert_eq!(w3c_code(UNKNOWN_PHYSICAL_KEY), None);
    }

    #[test]
    fn no_scancode_is_listed_twice() {
        for (index, (evdev, _)) in EVDEV_TO_W3C_CODE.iter().enumerate() {
            assert!(
                EVDEV_TO_W3C_CODE[index + 1..]
                    .iter()
                    .all(|(other, _)| other != evdev)
            );
        }
    }
}
