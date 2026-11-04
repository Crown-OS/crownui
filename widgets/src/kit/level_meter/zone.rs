use geometry::Color;

use super::{
    metrics::{CLIPPING_DECIBELS, WARNING_DECIBELS},
    reading::decibels_at,
};
use crate::kit::Theme;

/// The band of the scale a level falls in, colored so clipping is seen before it is heard.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum MeterZone {
    Safe,
    Hot,
    Clipping,
}

impl MeterZone {
    pub(super) fn at(position: f32) -> Self {
        match decibels_at(position) {
            decibels if decibels >= CLIPPING_DECIBELS => Self::Clipping,
            decibels if decibels >= WARNING_DECIBELS => Self::Hot,
            _ => Self::Safe,
        }
    }

    pub(super) fn color(self, theme: &Theme) -> Color {
        match self {
            Self::Safe => theme.accent.start,
            Self::Hot => theme.status.warning,
            Self::Clipping => theme.status.danger,
        }
    }
}
