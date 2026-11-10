use geometry::{BLACK, Color, WHITE};

use super::metrics::{PANEL_OPACITY, RIM_ALPHA, RIM_MIX};
use crate::{
    kit::{GradientStops, Theme},
    util::{
        color::{mix, scale_alpha},
        hsl::shift_hsl,
        surface_paint::{DropShadow, InnerShadow, InsetRing, SurfacePaint},
    },
};

/// How far a chosen badge's gradient lifts its top above the accent and sinks its bottom below
/// it, as HSL (lightness, saturation) steps.
const BADGE_LIFT: (f32, f32) = (0.08, 0.07);
const BADGE_SINK: (f32, f32) = (-0.07, -0.12);

/// The panel body at `opacity`, which the compositor's blur shows through.
pub fn panel_fill(theme: &Theme, opacity: f32) -> Color {
    Color {
        a: opacity.clamp(0.0, 1.0),
        ..theme.popover.bg
    }
}

/// The hairline that keeps a translucent panel's edge off the wallpaper, in both modes.
pub fn panel_rim(theme: &Theme, opacity: f32) -> Color {
    Color {
        a: RIM_ALPHA,
        ..mix(panel_fill(theme, opacity), theme.text.primary, RIM_MIX)
    }
}

pub(super) fn default_fill(theme: &Theme) -> Color {
    panel_fill(theme, PANEL_OPACITY)
}

fn wash(color: Color, top: f32, bottom: f32) -> GradientStops {
    GradientStops::new(scale_alpha(color, top), scale_alpha(color, bottom))
}

fn drop(color: Color, alpha: f32, y: f32, blur: f32) -> Option<DropShadow> {
    Some(DropShadow {
        color: scale_alpha(color, alpha),
        y,
        blur,
    })
}

fn highlight(alpha: f32) -> Option<InnerShadow> {
    Some(InnerShadow::highlight(scale_alpha(WHITE, alpha), 1.0))
}

/// The capsule behind a chosen row: a faint accent wash, ringed in light mode and lit along its
/// top in dark mode.
pub(super) fn row_selected(theme: &Theme) -> SurfacePaint {
    let accent = theme.accent.end;
    if theme.mode.is_dark() {
        SurfacePaint {
            fill: wash(accent, 0.32, 0.22),
            drop: drop(BLACK, 0.15, 1.0, 3.0),
            inner: highlight(0.08),
            ring: None,
        }
    } else {
        SurfacePaint {
            fill: wash(accent, 0.12, 0.07),
            drop: drop(accent, 0.08, 1.0, 3.0),
            inner: None,
            ring: Some(InsetRing {
                color: scale_alpha(accent, 0.10),
                width: 1.0,
            }),
        }
    }
}

/// The accent's own badge gradient: a lighter and a deeper shade of its deep stop.
pub(super) fn badge_stops(theme: &Theme) -> GradientStops {
    let accent = theme.accent.end;
    GradientStops::new(
        shift_hsl(accent, BADGE_LIFT.0, BADGE_LIFT.1),
        shift_hsl(accent, BADGE_SINK.0, BADGE_SINK.1),
    )
}

/// A chosen row's badge, a raised accent disc.
pub(super) fn badge_selected(theme: &Theme) -> SurfacePaint {
    let shadow = if theme.mode.is_dark() {
        BLACK
    } else {
        theme.accent.end
    };
    SurfacePaint {
        fill: badge_stops(theme),
        drop: drop(shadow, 0.30, 2.0, 6.0),
        inner: highlight(0.28),
        ring: None,
    }
}

/// A badge at rest, and the disc behind a slider's glyph: a faint ink bubble.
pub(super) fn badge_idle(theme: &Theme) -> SurfacePaint {
    let ink = theme.popover.text;
    if theme.mode.is_dark() {
        SurfacePaint {
            fill: wash(ink, 0.13, 0.07),
            drop: drop(BLACK, 0.25, 1.0, 2.0),
            inner: highlight(0.10),
            ring: None,
        }
    } else {
        SurfacePaint {
            fill: wash(ink, 0.05, 0.08),
            drop: drop(ink, 0.06, 1.0, 2.0),
            inner: highlight(0.70),
            ring: None,
        }
    }
}

/// The glyph on a chosen row's badge.
pub(super) fn badge_glyph_selected(theme: &Theme) -> Color {
    theme.text.on_accent
}

#[cfg(test)]
mod tests {
    use crownconfig::AccentColor;

    use super::*;
    use crate::kit::ThemeMode;

    fn close_to(color: Color, hex: [u8; 3]) -> bool {
        let [r, g, b, _] = color.to_rgba8();
        [r, g, b]
            .into_iter()
            .zip(hex)
            .all(|(channel, expected)| channel.abs_diff(expected) <= 2)
    }

    #[test]
    fn the_purple_badge_matches_the_design() {
        let stops = badge_stops(&Theme::for_mode(ThemeMode::Light, AccentColor::Purple));
        assert!(close_to(stops.start, [0x86, 0x66, 0xf2]), "{stops:?}");
        assert!(close_to(stops.end, [0x5b, 0x36, 0xd6]), "{stops:?}");
    }

    #[test]
    fn every_accent_badge_runs_light_to_deep_and_stays_opaque() {
        let luma = |color: Color| 0.2126 * color.r + 0.7152 * color.g + 0.0722 * color.b;
        for accent in [
            AccentColor::Purple,
            AccentColor::Blue,
            AccentColor::Green,
            AccentColor::Orange,
            AccentColor::Pink,
        ] {
            let theme = Theme::for_mode(ThemeMode::Dark, accent);
            let stops = badge_stops(&theme);
            assert!(luma(stops.start) > luma(theme.accent.end), "{accent:?}");
            assert!(luma(stops.end) < luma(theme.accent.end), "{accent:?}");
            assert!(stops.start.a == 1.0 && stops.end.a == 1.0, "{accent:?}");
        }
    }

    const PURPLE: Color = geometry::rgb(0x6d48e8);
    const INK_LIGHT: Color = geometry::rgb(0x101218);

    /// The Purple theme with the panel's ink, as a host's rows theme sets it.
    fn rows(mode: ThemeMode, ink: Color) -> Theme {
        let mut theme = Theme::for_mode(mode, AccentColor::Purple);
        theme.popover.text = ink;
        theme
    }

    fn at(color: Color, alpha: f32) -> Color {
        scale_alpha(color, alpha)
    }

    #[test]
    fn a_chosen_row_matches_the_design_in_both_modes() {
        assert_eq!(
            row_selected(&rows(ThemeMode::Light, INK_LIGHT)),
            SurfacePaint {
                fill: GradientStops::new(at(PURPLE, 0.12), at(PURPLE, 0.07)),
                drop: drop(PURPLE, 0.08, 1.0, 3.0),
                inner: None,
                ring: Some(InsetRing {
                    color: at(PURPLE, 0.10),
                    width: 1.0,
                }),
            }
        );
        assert_eq!(
            row_selected(&rows(ThemeMode::Dark, WHITE)),
            SurfacePaint {
                fill: GradientStops::new(at(PURPLE, 0.32), at(PURPLE, 0.22)),
                drop: drop(BLACK, 0.15, 1.0, 3.0),
                inner: highlight(0.08),
                ring: None,
            }
        );
    }

    #[test]
    fn a_chosen_badge_matches_the_design_in_both_modes() {
        for (mode, shadow) in [(ThemeMode::Light, PURPLE), (ThemeMode::Dark, BLACK)] {
            let theme = rows(mode, WHITE);
            let badge = badge_selected(&theme);
            assert_eq!(badge.fill, badge_stops(&theme), "{mode:?}");
            assert_eq!(badge.drop, drop(shadow, 0.30, 2.0, 6.0), "{mode:?}");
            assert_eq!(badge.inner, highlight(0.28), "{mode:?}");
            assert_eq!(badge.ring, None, "{mode:?}");
            assert_eq!(badge_glyph_selected(&theme), WHITE, "{mode:?}");
        }
    }

    #[test]
    fn an_idle_badge_matches_the_design_in_both_modes() {
        assert_eq!(
            badge_idle(&rows(ThemeMode::Light, INK_LIGHT)),
            SurfacePaint {
                fill: GradientStops::new(at(INK_LIGHT, 0.05), at(INK_LIGHT, 0.08)),
                drop: drop(INK_LIGHT, 0.06, 1.0, 2.0),
                inner: highlight(0.70),
                ring: None,
            }
        );
        assert_eq!(
            badge_idle(&rows(ThemeMode::Dark, WHITE)),
            SurfacePaint {
                fill: GradientStops::new(at(WHITE, 0.13), at(WHITE, 0.07)),
                drop: drop(BLACK, 0.25, 1.0, 2.0),
                inner: highlight(0.10),
                ring: None,
            }
        );
    }

    #[test]
    fn a_chosen_row_follows_the_accent() {
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            let [purple, blue] = [AccentColor::Purple, AccentColor::Blue]
                .map(|accent| row_selected(&Theme::for_mode(mode, accent)).fill);
            assert_ne!(purple, blue, "{mode:?}");
        }
    }
}
