use geometry::Color;

use crate::{kit::Theme, util::color::scale_alpha};

const PENDING_ALPHA: f32 = 0.55;

/// Where a step sits relative to the one the user is on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Stage {
    Done,
    Current,
    Pending,
}

impl Stage {
    pub(super) fn of(step: usize, current: usize) -> Self {
        match step.cmp(&current) {
            std::cmp::Ordering::Less => Self::Done,
            std::cmp::Ordering::Equal => Self::Current,
            std::cmp::Ordering::Greater => Self::Pending,
        }
    }

    pub(super) fn dot_fill(self, theme: &Theme) -> Color {
        match self {
            Self::Done | Self::Current => theme.accent.end,
            Self::Pending => pending_color(theme),
        }
    }

    pub(super) fn label_color(self, theme: &Theme) -> Color {
        match self {
            Self::Current => theme.text.primary,
            Self::Done => theme.text.body,
            Self::Pending => theme.text.muted,
        }
    }
}

pub(super) fn pending_color(theme: &Theme) -> Color {
    scale_alpha(theme.control.track, PENDING_ALPHA)
}

pub(super) fn connector_color(theme: &Theme, reached: bool) -> Color {
    if reached {
        theme.accent.end
    } else {
        pending_color(theme)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn steps_before_the_current_one_are_done_and_those_after_pending() {
        assert_eq!(Stage::of(0, 2), Stage::Done);
        assert_eq!(Stage::of(2, 2), Stage::Current);
        assert_eq!(Stage::of(3, 2), Stage::Pending);
    }

    #[test]
    fn done_and_current_dots_carry_the_accent_and_pending_ones_a_faint_track() {
        let theme = Theme::dark();
        assert_eq!(Stage::Done.dot_fill(&theme), theme.accent.end);
        assert_eq!(Stage::Current.dot_fill(&theme), theme.accent.end);
        assert_ne!(Stage::Pending.dot_fill(&theme), theme.accent.end);
        assert!((pending_color(&theme).a - theme.control.track.a * 0.55).abs() < 1e-6);
    }
}
