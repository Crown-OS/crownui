use std::collections::BTreeMap;

use geometry::Color;

use super::{accent::GradientStops, definition::Theme, palette::NamedSlots};

/// `#rrggbb` or `#rrggbbaa` (the `#` is optional).
pub(super) fn parse_hex(text: &str) -> Option<Color> {
    let digits = text.strip_prefix('#').unwrap_or(text);
    let value = u32::from_str_radix(digits, 16).ok()?;
    match digits.len() {
        6 => Some(geometry::rgb(value)),
        8 => Some(geometry::rgba(value)),
        _ => None,
    }
}

pub(super) fn parse_stops(start: &str, end: &str) -> Option<GradientStops> {
    Some(GradientStops::new(parse_hex(start)?, parse_hex(end)?))
}

fn find_slot<'a, const COUNT: usize>(
    slots: NamedSlots<'a, COUNT>,
    name: &str,
) -> Option<&'a mut Color> {
    slots
        .into_iter()
        .find(|(slot, _)| *slot == name)
        .map(|(_, color)| color)
}

impl Theme {
    fn slot_mut(&mut self, path: &str) -> Option<&mut Color> {
        let (group, name) = path.split_once('.')?;
        match group {
            "surface" => find_slot(self.surface.slots_mut(), name),
            "text" => find_slot(self.text.slots_mut(), name),
            "control" => find_slot(self.control.slots_mut(), name),
            "status" => find_slot(self.status.slots_mut(), name),
            "popover" => find_slot(self.popover.slots_mut(), name),
            "radio" => find_slot(self.radio.slots_mut(), name),
            "menu" => find_slot(self.menu.slots_mut(), name),
            "sidebar" => find_slot(self.sidebar.slots_mut(), name),
            "toggle_off" => match name {
                "start" => Some(&mut self.toggle_off.start),
                "end" => Some(&mut self.toggle_off.end),
                _ => None,
            },
            _ => None,
        }
    }

    /// The color behind a dotted token such as `surface.bg` or `accent.start`; `None` for a name
    /// the palette does not have.
    pub fn token(&self, path: &str) -> Option<Color> {
        match path {
            "accent.start" => Some(self.accent.start),
            "accent.end" => Some(self.accent.end),
            _ => {
                let mut palette = *self;
                palette.slot_mut(path).copied()
            }
        }
    }

    /// This palette with each `group.slot` of `overrides` replaced; unknown slots and malformed
    /// colors are skipped.
    #[must_use]
    pub fn with_overrides(mut self, overrides: &BTreeMap<String, String>) -> Self {
        for (path, hex) in overrides {
            if let (Some(slot), Some(color)) = (self.slot_mut(path), parse_hex(hex)) {
                *slot = color;
            }
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use geometry::rgb;

    use super::*;

    fn overrides(entries: &[(&str, &str)]) -> BTreeMap<String, String> {
        entries
            .iter()
            .map(|(path, hex)| ((*path).to_owned(), (*hex).to_owned()))
            .collect()
    }

    #[test]
    fn hex_accepts_six_and_eight_digits() {
        assert_eq!(parse_hex("#6d48e8"), Some(rgb(0x6d48e8)));
        assert_eq!(parse_hex("6d48e8"), Some(rgb(0x6d48e8)));
        assert_eq!(parse_hex("#11223344"), Some(geometry::rgba(0x11223344)));
        assert_eq!(parse_hex("#12345"), None);
        assert_eq!(parse_hex("#zzzzzz"), None);
    }

    #[test]
    fn overrides_replace_named_slots_only() {
        let base = Theme::dark();
        let themed = base.with_overrides(&overrides(&[
            ("surface.bg", "#101010"),
            ("menu.selected_text", "#ff0000"),
            ("toggle_off.end", "#000000"),
        ]));
        assert_eq!(themed.surface.bg, rgb(0x101010));
        assert_eq!(themed.menu.selected_text, rgb(0xff0000));
        assert_eq!(themed.toggle_off.end, rgb(0x000000));
        assert_eq!(themed.text, base.text);
    }

    #[test]
    fn every_kit_token_resolves() {
        let theme = Theme::dark();
        for token in crownconfig::COLOR_TOKENS
            .iter()
            .filter(|token| !token.starts_with("frame.") && !token.starts_with("bar."))
        {
            assert!(theme.token(token).is_some(), "{token} has no palette slot");
        }
        assert_eq!(theme.token("nowhere.bg"), None);
    }

    #[test]
    fn unknown_slots_and_bad_colors_are_ignored() {
        let base = Theme::light();
        let themed = base.with_overrides(&overrides(&[
            ("surface.nope", "#101010"),
            ("nowhere.bg", "#101010"),
            ("surface", "#101010"),
            ("surface.bg", "not a color"),
        ]));
        assert_eq!(themed, base);
    }
}
