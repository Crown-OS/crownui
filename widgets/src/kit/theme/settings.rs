use crownconfig::ThemeSettings;

use super::{definition::Theme, mode::ThemeMode, overrides::parse_stops};

impl Theme {
    /// The palette `appearance.ron` and `theme.ron` describe: the appearance picks mode and
    /// accent, then the overrides edit the accent and individual slots.
    pub fn from_settings(settings: &ThemeSettings) -> Self {
        let appearance = &settings.appearance;
        let mode = ThemeMode::from_dark_mode(appearance.dark_mode);
        let theme = Self::for_mode(mode, appearance.accent);
        let theme = match settings
            .overrides
            .accent
            .as_ref()
            .and_then(|accent| parse_stops(&accent.start, &accent.end))
        {
            Some(stops) => Self::with_stops(mode, stops),
            None => theme,
        };
        let slots = if mode.is_dark() {
            &settings.overrides.dark
        } else {
            &settings.overrides.light
        };
        theme.with_overrides(slots)
    }
}

#[cfg(test)]
mod tests {
    use crownconfig::{AccentColor, AccentOverride, Appearance, ThemeOverrides};
    use geometry::rgb;

    use super::*;
    use crate::kit::theme::accent::accent_stops;

    fn settings(dark_mode: bool, accent: AccentColor, overrides: ThemeOverrides) -> ThemeSettings {
        ThemeSettings {
            appearance: Appearance {
                dark_mode,
                accent,
                ..Appearance::default()
            },
            overrides,
        }
    }

    #[test]
    fn appearance_selects_mode_and_accent() {
        let theme = Theme::from_settings(&settings(
            false,
            AccentColor::Orange,
            ThemeOverrides::default(),
        ));
        assert_eq!(theme.mode, ThemeMode::Light);
        assert_eq!(theme.accent, accent_stops(AccentColor::Orange));
    }

    #[test]
    fn a_custom_accent_reaches_every_slot_that_carries_it() {
        let overrides = ThemeOverrides {
            accent: Some(AccentOverride {
                start: "#112233".into(),
                end: "#445566".into(),
            }),
            ..ThemeOverrides::default()
        };
        let theme = Theme::from_settings(&settings(true, AccentColor::Purple, overrides));
        assert_eq!(theme.radio.accent, theme.accent);
        assert_eq!(theme.menu.selected_bg, rgb(0x445566));
    }

    #[test]
    fn slot_overrides_apply_to_the_active_mode_only() {
        let mut overrides = ThemeOverrides::default();
        overrides.dark.insert("surface.bg".into(), "#010203".into());
        overrides
            .light
            .insert("surface.bg".into(), "#fefdfc".into());
        let dark = Theme::from_settings(&settings(true, AccentColor::Purple, overrides.clone()));
        let light = Theme::from_settings(&settings(false, AccentColor::Purple, overrides));
        assert_eq!(dark.surface.bg, rgb(0x010203));
        assert_eq!(light.surface.bg, rgb(0xfefdfc));
    }

    #[test]
    fn a_malformed_accent_falls_back_to_the_appearance() {
        let overrides = ThemeOverrides {
            accent: Some(AccentOverride {
                start: "oops".into(),
                end: "#445566".into(),
            }),
            ..ThemeOverrides::default()
        };
        let theme = Theme::from_settings(&settings(true, AccentColor::Blue, overrides));
        assert_eq!(theme.accent, accent_stops(AccentColor::Blue));
    }
}
