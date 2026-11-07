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
    let backdrop_sigma = backdrop_blur.and_then(filter_sigma);
    if opacity >= 1.0 && backdrop_sigma.is_none() {
        return false;
    }
    let backdrop = backdrop_sigma.map(|sigma| ImageFilter::new_blur(sigma, sigma, TileMode::Clamp));
    let paint = (opacity < 1.0).then(|| opacity_paint(opacity));
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_blur_animating_up_from_zero_does_not_abort_the_layer() {
        let mut builder = DisplayListBuilder::new(None);
        for radius in [1.0e-6, 1.0e-4, f32::INFINITY, f32::NAN, -1.0] {
            assert!(!save_layer(&mut builder, None, 1.0, Some(radius)));
        }
        assert!(save_layer(&mut builder, None, 1.0, Some(8.0)));
    }
}
