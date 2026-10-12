use geometry::Color;

use super::metrics::{BADGE_ALPHA_BOOST, PANEL_OPACITY, RIM_ALPHA, RIM_MIX, SELECTED_ALPHA};
use crate::{kit::Theme, util::color::mix};

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

pub(super) fn row_selected(theme: &Theme) -> Color {
    Color {
        a: SELECTED_ALPHA,
        ..theme.menu.selected_bg
    }
}

pub(super) fn badge_idle(theme: &Theme) -> Color {
    let hover = theme.surface.hover;
    Color {
        a: (hover.a * BADGE_ALPHA_BOOST).min(1.0),
        ..hover
    }
}
