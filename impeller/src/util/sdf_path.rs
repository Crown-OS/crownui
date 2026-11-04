use std::f32::consts::SQRT_2;

use geometry::{Color, Insets, Rect};
use graphics::{Stroke, StrokeCap};
use impellers::{BlendMode, DisplayListBuilder, Paint};

use super::{
    flatten::{Flattening, OpenContours},
    geometry::to_rect,
    path_shader::PathShader,
    path_uniforms::PathUniforms,
    segment_texture::SegmentTexture,
};

/// The longest side, in device pixels, of paths the shader draws: it visits every segment for
/// every pixel, so larger paths stay on MSAA.
const MAX_DEVICE_EXTENT: f32 = 128.0;
/// The furthest a flattened curve strays from the path, in device pixels.
const FLATTENING_TOLERANCE: f32 = 0.1;
/// How far past the path its quad reaches, in device pixels: the shader shades the half pixel
/// outside the edge, and a quad edge never cuts through the coverage ramp.
const AA_MARGIN: f32 = 1.0;

/// A small path drawn by `flattened_path.frag` from its segment texture, with analytic edge
/// coverage. Strokes join segments round whatever their `StrokeJoin`: at icon sizes a miter or
/// bevel differs by under a pixel.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct SdfPath {
    bounds: Rect,
    stroke: Option<Stroke>,
    device_scale: f32,
}

impl SdfPath {
    /// The fill (`stroke` `None`) or stroke of a path with `bounds`, drawn at `device_scale`
    /// device pixels per unit; `None` when it is too large on screen or a hairline.
    pub(crate) fn new(bounds: Rect, stroke: Option<Stroke>, device_scale: f32) -> Option<Self> {
        let extent = bounds.size.width.max(bounds.size.height) * device_scale;
        let drawable = device_scale.is_normal()
            && extent <= MAX_DEVICE_EXTENT
            && stroke.is_none_or(|stroke| stroke.width > 0.0);
        drawable.then_some(Self {
            bounds,
            stroke,
            device_scale,
        })
    }

    /// How to cut the path into the segments the shader reads.
    pub(crate) fn flattening(&self) -> Flattening {
        Flattening {
            tolerance: FLATTENING_TOLERANCE / self.device_scale,
            open_contours: match self.stroke {
                Some(_) => OpenContours::Capped,
                None => OpenContours::Closed,
            },
        }
    }

    /// Shades the path in `color` from `segments`, its flattened texture.
    pub(crate) fn paint(
        &self,
        color: Color,
        segments: &SegmentTexture,
        shader: PathShader<'_>,
    ) -> Paint {
        let uniforms =
            PathUniforms::new(segments.bounds, segments.segment_count, self.stroke, color);
        shader.paint(&uniforms, &segments.texture)
    }

    pub(crate) fn draw(&self, builder: &mut DisplayListBuilder, shaded: &Paint) {
        builder.draw_rect(&self.quad(), shaded);
    }

    /// Paints `paint` over the path's quad in a layer, then cuts the layer to `mask`, the path
    /// shaded white.
    pub(crate) fn draw_masked(
        &self,
        builder: &mut DisplayListBuilder,
        paint: &Paint,
        mut mask: Paint,
    ) {
        let quad = self.quad();
        builder.save_layer(&quad, None, None);
        builder.draw_rect(&quad, paint);
        mask.set_blend_mode(BlendMode::DestinationIn);
        builder.draw_rect(&quad, &mask);
        builder.restore();
    }

    fn quad(&self) -> impellers::Rect {
        let stroke_reach = self.stroke.map_or(0.0, |stroke| match stroke.cap {
            StrokeCap::Square => stroke.width * 0.5 * SQRT_2,
            StrokeCap::Butt | StrokeCap::Round => stroke.width * 0.5,
        });
        let reach = stroke_reach + AA_MARGIN / self.device_scale;
        to_rect(self.bounds.inset(Insets::uniform(-reach)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn icon() -> Rect {
        Rect::from_xywh(0.0, 0.0, 24.0, 24.0)
    }

    #[test]
    fn paths_larger_than_the_device_limit_or_hairlines_stay_on_msaa() {
        assert!(SdfPath::new(icon(), None, 2.0).is_some());
        assert!(SdfPath::new(icon(), None, 6.0).is_none());
        assert!(SdfPath::new(icon(), Some(Stroke::new(0.0)), 2.0).is_none());
        assert!(SdfPath::new(icon(), None, 0.0).is_none());
    }

    #[test]
    fn the_tolerance_is_a_tenth_of_a_device_pixel() {
        let Some(path) = SdfPath::new(icon(), Some(Stroke::rounded(2.0)), 2.0) else {
            panic!("an icon fits");
        };
        let flattening = path.flattening();
        assert_eq!(flattening.tolerance, 0.05);
        assert_eq!(flattening.open_contours, OpenContours::Capped);
    }

    #[test]
    fn the_quad_covers_the_stroke_caps_and_the_antialiased_edge() {
        let quad = |stroke| match SdfPath::new(icon(), stroke, 2.0) {
            Some(path) => path.quad(),
            None => panic!("an icon fits"),
        };
        let filled = quad(None);
        assert_eq!((filled.origin.x, filled.size.width), (-0.5, 25.0));
        let round = quad(Some(Stroke::rounded(2.0)));
        assert_eq!((round.origin.x, round.size.width), (-1.5, 27.0));
        let square = quad(Some(Stroke {
            cap: StrokeCap::Square,
            ..Stroke::new(2.0)
        }));
        assert!((square.origin.x + SQRT_2 + 0.5).abs() < 1e-6);
    }
}
