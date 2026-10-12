mod kind;
mod step;
mod track;

pub use kind::StepBarKind;
pub use step::IntoStep;

use view::{Element, Styled};

use crate::util::theme_fade::theme_fade;

const TRACK_HEIGHT: f32 = 5.0;
const TRACK_GAP: f32 = 10.0;

/// How many tracks `current` should light: the step being shown counts as reached, an index past the end fills the bar.
fn reached(total: usize, current: usize) -> f32 {
    if total == 0 {
        return 0.0;
    }
    (current + 1).min(total) as f32
}

/// A step indicator with `total` equal tracks, the first `current + 1` filled with the accent; it fills its column's width.
pub fn step_bar(total: usize, current: impl IntoStep) -> Element<StepBarKind> {
    Element::new(StepBarKind {
        total,
        current: current.into_step(),
    })
    .flex_row()
    .self_stretch()
    .shrink(0.0)
    .gap(TRACK_GAP)
    .h(TRACK_HEIGHT)
    .transition(theme_fade())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_current_step_counts_as_reached_and_overshoot_is_clamped() {
        assert_eq!(reached(4, 0), 1.0);
        assert_eq!(reached(4, 3), 4.0);
        assert_eq!(reached(4, 9), 4.0);
        assert_eq!(reached(0, 0), 0.0);
    }
}
