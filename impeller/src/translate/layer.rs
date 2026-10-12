use geometry::Rect;
use impellers::{DisplayListBuilder, ImageFilter, TileMode};

use super::paint::opacity_paint;
use crate::util::{blur::filter_sigma, geometry::to_rect};

const UNBOUNDED_LAYER_EXTENT: f32 = 1.0e7;

/// Starts an offscreen layer when `opacity` or `backdrop_blur` needs one; returns whether it did.
pub(crate) fn save_layer(
    builder: &mut DisplayListBuilder,
    bounds: Option<Rect>,
    opacity: f32,
    backdrop_blur: Option<f32>,
) -> bool {
    let backdrop = backdrop_blur.filter(|radius| *radius > 0.0).map(|radius| {
        let sigma = filter_sigma(radius);
        ImageFilter::new_blur(sigma, sigma, TileMode::Clamp)
    });
    let paint = (opacity < 1.0).then(|| opacity_paint(opacity));
    if backdrop.is_none() && paint.is_none() {
        return false;
    }
    let bounds = bounds.map_or_else(unbounded_rect, to_rect);
    builder.save_layer(&bounds, paint.as_ref(), backdrop.as_ref());
    true
}

fn unbounded_rect() -> impellers::Rect {
    impellers::Rect::new(
        impellers::Point::new(-UNBOUNDED_LAYER_EXTENT, -UNBOUNDED_LAYER_EXTENT),
        impellers::Size::new(UNBOUNDED_LAYER_EXTENT * 2.0, UNBOUNDED_LAYER_EXTENT * 2.0),
    )
}
