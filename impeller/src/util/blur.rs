/// Impeller returns no filter, and `impellers` panics, for a sigma below this.
const MIN_BLUR_SIGMA: f32 = 1.0 / 4096.0;

/// Sigma of a CSS `filter: blur(radius)` or `backdrop-filter: blur(radius)`: the radius *is* the
/// standard deviation. `None` when the blur is too small for Impeller to draw.
pub(crate) fn filter_sigma(radius: f32) -> Option<f32> {
    drawable_sigma(radius)
}

/// Sigma of a CSS `box-shadow` blur radius: half the radius. `None` when the shadow is sharp.
pub(crate) fn shadow_sigma(blur_radius: f32) -> Option<f32> {
    drawable_sigma(blur_radius * 0.5)
}

fn drawable_sigma(sigma: f32) -> Option<f32> {
    (sigma.is_finite() && sigma >= MIN_BLUR_SIGMA).then_some(sigma)
}
