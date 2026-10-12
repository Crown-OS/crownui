use geometry::{Color, TRANSPARENT};

/// `from` blended `progress` of the way to `to`, mixed premultiplied so a fade between a faint
/// wash and an opaque fill never flashes through a half-opaque midpoint.
pub(crate) fn mix(from: Color, to: Color, progress: f32) -> Color {
    if progress <= 0.0 {
        return from;
    }
    if progress >= 1.0 {
        return to;
    }
    let alpha = from.a + (to.a - from.a) * progress;
    if alpha <= f32::EPSILON {
        return TRANSPARENT;
    }
    let channel = |from_channel: f32, to_channel: f32| {
        let (from_weighted, to_weighted) = (from_channel * from.a, to_channel * to.a);
        (from_weighted + (to_weighted - from_weighted) * progress) / alpha
    };
    Color::new(
        channel(from.r, to.r),
        channel(from.g, to.g),
        channel(from.b, to.b),
        alpha,
    )
}

/// `color` with its alpha scaled by `factor`, clamped to `0..=1`.
pub(crate) fn scale_alpha(color: Color, factor: f32) -> Color {
    color.with_alpha_factor(factor.clamp(0.0, 1.0))
}

#[cfg(test)]
mod tests {
    use geometry::{BLACK, WHITE};

    use super::*;

    #[test]
    fn mixing_with_a_transparent_end_keeps_the_other_ends_color() {
        let faded = mix(TRANSPARENT, WHITE, 0.5);
        assert_eq!((faded.r, faded.g, faded.b), (1.0, 1.0, 1.0));
        assert!((faded.a - 0.5).abs() < 1e-6);
    }

    #[test]
    fn mixing_opaque_colors_is_a_plain_component_mix() {
        let grey = mix(BLACK, WHITE, 0.5);
        assert!((grey.r - 0.5).abs() < 1e-6 && (grey.a - 1.0).abs() < 1e-6);
    }

    #[test]
    fn scaling_alpha_multiplies_it() {
        assert!((scale_alpha(WHITE.with_alpha_factor(0.5), 0.5).a - 0.25).abs() < 1e-6);
    }
}
