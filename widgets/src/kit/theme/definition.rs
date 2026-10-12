use crownconfig::AccentColor;
use geometry::{Color, rgb};

use super::{
    accent::{DEFAULT_ACCENT, GradientStops, accent_stops},
    mode::ThemeMode,
    palette::{
        ControlColors, MenuColors, PopoverColors, RadioColors, SidebarColors, StatusColors,
        SurfaceColors, TextColors,
    },
};

/// A color from `0xRRGGBB` and an explicit 8-bit alpha.
const fn tint(hex: u32, alpha: u8) -> Color {
    Color::from_rgba8((hex >> 16) as u8, (hex >> 8) as u8, hex as u8, alpha)
}

/// The kit's complete palette: every color any widget paints, in one value.
///
/// Spread over a base to override slots:
/// `Theme { sidebar: SidebarColors { bg, ..base.sidebar }, ..base }`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Theme {
    /// Which palette this is; read only for the direction of a shadow or tint.
    pub mode: ThemeMode,
    /// The accent gradient: toggle on state, slider fill, selected radio, focus rings, menu bar.
    pub accent: GradientStops,
    /// Backgrounds and the lines between them.
    pub surface: SurfaceColors,
    /// Foregrounds.
    pub text: TextColors,
    /// Toggle knob, slider track and thumb.
    pub control: ControlColors,
    /// Off-state background of pill toggles, top to bottom; travels to `accent` as the toggle
    /// turns on.
    pub toggle_off: GradientStops,
    /// Semantic state colors; the one group that does not follow the accent.
    pub status: StatusColors,
    /// Dropdown surfaces: select trigger and panel.
    pub popover: PopoverColors,
    /// Radio groups.
    pub radio: RadioColors,
    /// Context menus.
    pub menu: MenuColors,
    /// The sidebar layout.
    pub sidebar: SidebarColors,
}

impl Theme {
    /// The palette in force before anything is loaded: dark, purple, what `Appearance::default`
    /// describes.
    pub const DEFAULT: Self = Self::dark();

    /// The palette for `mode` with a named `accent`.
    pub const fn for_mode(mode: ThemeMode, accent: AccentColor) -> Self {
        Self::with_stops(mode, accent_stops(accent))
    }

    /// The palette for `mode` with the accent given directly as gradient stops.
    pub const fn with_stops(mode: ThemeMode, accent: GradientStops) -> Self {
        match mode {
            ThemeMode::Light => Self::light_with(accent),
            ThemeMode::Dark => Self::dark_with(accent),
        }
    }

    /// The light palette with the default accent.
    pub const fn light() -> Self {
        Self::light_with(DEFAULT_ACCENT)
    }

    /// The dark palette with the default accent.
    pub const fn dark() -> Self {
        Self::dark_with(DEFAULT_ACCENT)
    }

    const fn light_with(accent: GradientStops) -> Self {
        Self {
            mode: ThemeMode::Light,
            accent,
            surface: SurfaceColors {
                bg: tint(0xf3f3f3, 0xee),
                raised: rgb(0xffffff),
                sunken: rgb(0xf2f2f6),
                border: tint(0x1b1b2a, 18),
                hairline: tint(0x1b1b2a, 12),
                hover: tint(0x2a1f45, 12),
                shadow: tint(0x14141f, 13),
            },
            text: TextColors {
                primary: rgb(0x17151f),
                body: rgb(0x403e4a),
                muted: rgb(0x8b8996),
                disabled: rgb(0xc2c0cb),
                on_accent: rgb(0xffffff),
                icon: rgb(0x3d3b47),
                icon_muted: rgb(0x78768a),
            },
            control: ControlColors {
                track: rgb(0xe7e5f1),
                knob: rgb(0xffffff),
                knob_shadow: tint(0x000000, 13),
            },
            toggle_off: GradientStops::new(rgb(0xedebf6), rgb(0xd9d6e8)),
            status: StatusColors {
                success: rgb(0x159c4e),
                warning: rgb(0xb45409),
                danger: rgb(0xc22727),
                neutral: rgb(0x8a8a8f),
                danger_bg: rgb(0xdc2a2a),
                on_danger: rgb(0xffffff),
            },
            popover: PopoverColors {
                bg: rgb(0xffffff),
                border: rgb(0xe7e5f1),
                text: rgb(0x1b1926),
                muted_text: rgb(0x78768a),
                hover_bg: rgb(0xf1f0f7),
                trigger_bg: rgb(0xffffff),
                ring: accent,
            },
            radio: RadioColors {
                accent,
                unselected_bg: rgb(0xffffff),
                unselected_border: rgb(0xd9d6e8),
                hole: rgb(0xffffff),
                text: rgb(0x1b1926),
            },
            menu: MenuColors {
                bg: rgb(0xf9f8fd),
                border: rgb(0xe7e5f1),
                text: rgb(0x1e1c29),
                shortcut_text: rgb(0x8e8c99),
                disabled_text: rgb(0xc2c0cb),
                header_text: rgb(0x8e8c99),
                separator: rgb(0xe7e5f1),
                selected_bg: accent.end,
                selected_text: rgb(0xffffff),
            },
            sidebar: SidebarColors {
                bg: tint(0xf3f3f3, 0x00),
                separator: rgb(0xeae8f2),
                text: rgb(0x646464),
                selected_text: rgb(0x545454),
                subitem_text: rgb(0x676473),
                brand_text: rgb(0x17151f),
                icon: rgb(0x646464),
                selected_icon: rgb(0x545454),
                hover_bg: tint(0x2a1f45, 8),
                selected_bg: tint(0xb0b0b0, 0x80),
                selected_border: tint(0x2a1f45, 0),
            },
        }
    }

    const fn dark_with(accent: GradientStops) -> Self {
        Self {
            mode: ThemeMode::Dark,
            accent,
            surface: SurfaceColors {
                bg: tint(0x0f0f10, 0xcc),
                raised: rgb(0x171718),
                sunken: rgb(0x121212),
                border: rgb(0x27272a),
                hairline: rgb(0x232326),
                hover: tint(0xffffff, 16),
                shadow: tint(0x000000, 89),
            },
            text: TextColors {
                primary: rgb(0xfafafa),
                body: rgb(0xd4d4d8),
                muted: rgb(0x9f9f9f),
                disabled: rgb(0x5a5a60),
                on_accent: rgb(0xffffff),
                icon: rgb(0xd4d4d8),
                icon_muted: rgb(0x8a8a93),
            },
            control: ControlColors {
                track: rgb(0x3a3a3d),
                knob: rgb(0xfafafa),
                knob_shadow: tint(0x000000, 115),
            },
            toggle_off: GradientStops::new(rgb(0x3f3f46), rgb(0x2a2a2e)),
            status: StatusColors {
                success: rgb(0x4ade80),
                warning: rgb(0xfbbf24),
                danger: rgb(0xf87171),
                neutral: rgb(0x8a8a93),
                danger_bg: rgb(0xc12b2b),
                on_danger: rgb(0xffffff),
            },
            popover: PopoverColors {
                bg: rgb(0x1d1d1d),
                border: rgb(0x282828),
                text: rgb(0xfafafa),
                muted_text: rgb(0x9f9f9f),
                hover_bg: rgb(0x2a2a2a),
                trigger_bg: rgb(0x1c1c1c),
                ring: accent,
            },
            radio: RadioColors {
                accent,
                unselected_bg: rgb(0x1c1c1c),
                unselected_border: rgb(0x3e3e3e),
                hole: rgb(0x1d1d1d),
                text: rgb(0xfafafa),
            },
            menu: MenuColors {
                bg: rgb(0x1d1d1d),
                border: rgb(0x2a2a2a),
                text: rgb(0xfafafa),
                shortcut_text: rgb(0x8e8e93),
                disabled_text: rgb(0x5a5a60),
                header_text: rgb(0x8e8e93),
                separator: rgb(0x282828),
                selected_bg: accent.end,
                selected_text: rgb(0xffffff),
            },
            sidebar: SidebarColors {
                bg: tint(0x131314, 0x00),
                separator: rgb(0x262628),
                text: rgb(0xa1a1aa),
                selected_text: rgb(0xfafafa),
                subitem_text: rgb(0x8a8a93),
                brand_text: rgb(0xfafafa),
                icon: rgb(0x8a8a93),
                selected_icon: rgb(0xfafafa),
                hover_bg: tint(0xffffff, 14),
                selected_bg: rgb(0x232326),
                selected_border: tint(0xffffff, 28),
            },
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::DEFAULT
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn luma(color: Color) -> f32 {
        color.r + color.g + color.b
    }

    #[test]
    fn palettes_are_opposites() {
        let (light, dark) = (Theme::light(), Theme::dark());
        assert!(!light.mode.is_dark() && dark.mode.is_dark());
        assert!(luma(light.surface.raised) > luma(light.text.body));
        assert!(luma(dark.surface.raised) < luma(dark.text.body));
        assert!(luma(light.surface.bg) > luma(dark.surface.bg));
        assert!(luma(dark.status.success) > luma(light.status.success));
        assert!(luma(dark.status.danger) > luma(light.status.danger));
    }

    #[test]
    fn one_accent_reaches_every_slot_that_carries_it() {
        for theme in [Theme::light(), Theme::dark()] {
            assert_eq!(theme.popover.ring, theme.accent);
            assert_eq!(theme.radio.accent, theme.accent);
            assert_eq!(theme.menu.selected_bg, theme.accent.end);
        }
    }

    #[test]
    fn status_tones_stay_distinguishable() {
        for theme in [Theme::light(), Theme::dark()] {
            let status = theme.status;
            let tones = [
                status.success,
                status.warning,
                status.danger,
                status.neutral,
            ];
            for (index, tone) in tones.iter().enumerate() {
                assert!(tones[index + 1..].iter().all(|other| other != tone));
            }
        }
    }

    #[test]
    fn the_default_is_dark_purple() {
        assert_eq!(
            Theme::default(),
            Theme::for_mode(ThemeMode::Dark, AccentColor::Purple)
        );
    }
}
