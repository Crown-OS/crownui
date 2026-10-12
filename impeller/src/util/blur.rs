/// Sigma of a CSS `filter: blur(radius)` or `backdrop-filter: blur(radius)`: the radius *is* the
/// standard deviation.
pub(crate) fn filter_sigma(radius: f32) -> f32 {
    radius.max(0.0)
}

/// Sigma of a CSS `box-shadow` blur radius: half the radius.
pub(crate) fn shadow_sigma(blur_radius: f32) -> f32 {
    blur_radius.max(0.0) * 0.5
}
