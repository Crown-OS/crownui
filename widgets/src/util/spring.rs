use std::f32::consts::TAU;

use motion::Spring;

/// The critically damped spring with the given `stiffness`: crownuikit's springs are stated that
/// way (`damping = 2 * sqrt(stiffness)`), while `motion` describes them by response.
fn critically_damped(stiffness: f32) -> Spring {
    Spring::new(TAU / stiffness.sqrt(), 1.0)
}

/// Fast settle (~200 ms) for anything that tracks a tap or the pointer: toggles, sliders, hovers.
pub(crate) fn snappy() -> Spring {
    critically_damped(320.0)
}

/// Softer, wider arc (~350 ms) for values that slide across a distance: panels, highlights.
pub(crate) fn smooth() -> Spring {
    critically_damped(180.0)
}

/// Very fast settle (~120 ms) for short decisive motion: press washes, fades.
pub(crate) fn crisp() -> Spring {
    critically_damped(900.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_three_springs_keep_crownuikits_stiffness_and_critical_damping() {
        for (spring, stiffness) in [(snappy(), 320.0), (smooth(), 180.0), (crisp(), 900.0)] {
            assert!((spring.stiffness() - stiffness).abs() / stiffness < 1e-3);
            assert!((spring.damping_ratio() - 1.0).abs() < 1e-6);
        }
    }
}
