use geometry::{CornerRadii, CornerShape, Insets, Rect, WHITE};
use graphics::Brush;
use impellers::{BlendMode, DisplayListBuilder, Paint};

use super::{
    geometry::to_rect, outline::is_capsule, shape_shader::ShapeShader,
    shape_uniforms::ShapeUniforms,
};

/// How far past the shape its quad reaches, in local units: the shader shades the half pixel
/// outside the edge, and a quad edge never cuts through the coverage ramp.
const AA_MARGIN: f32 = 1.0;

/// A rounded rectangle drawn by `rounded_box.frag`, with analytic edge coverage.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct SdfShape {
    rect: Rect,
    radii: CornerRadii,
    squircle: bool,
}

impl SdfShape {
    /// Pills and circles stay circular, as [`super::outline::Outline`] draws them; round radii
    /// shrink like CSS so adjacent corners never overlap.
    pub(crate) fn new(rect: Rect, radii: CornerRadii, shape: CornerShape) -> Self {
        let squircle = shape == CornerShape::Squircle && !is_capsule(rect, radii);
        let radii = if squircle {
            scale_radii(radii, 1.0)
        } else {
            scale_radii(radii, css_radius_scale(rect, radii))
        };
        Self {
            rect,
            radii,
            squircle,
        }
    }

    /// Fills the shape (`stroke_width` 0) or outlines it centred on its edge; false when the
    /// shader cannot paint `brush` and [`Self::draw_masked`] is needed.
    pub(crate) fn draw(
        &self,
        builder: &mut DisplayListBuilder,
        brush: &Brush,
        stroke_width: f32,
        shader: ShapeShader<'_>,
    ) -> bool {
        let Some(uniforms) = self.uniforms(brush, stroke_width) else {
            return false;
        };
        builder.draw_rect(&self.quad(stroke_width), &shader.paint(&uniforms));
        true
    }

    /// Paints `paint` over the shape's quad in a layer, then cuts the layer to the shape.
    pub(crate) fn draw_masked(
        &self,
        builder: &mut DisplayListBuilder,
        paint: &Paint,
        stroke_width: f32,
        shader: ShapeShader<'_>,
    ) {
        let quad = self.quad(stroke_width);
        builder.save_layer(&quad, None, None);
        builder.draw_rect(&quad, paint);
        self.mask(builder, stroke_width, shader);
        builder.restore();
    }

    /// Keeps what is already drawn only inside the shape, scaled by its edge coverage.
    pub(crate) fn mask(
        &self,
        builder: &mut DisplayListBuilder,
        stroke_width: f32,
        shader: ShapeShader<'_>,
    ) {
        if let Some(uniforms) = self.uniforms(&Brush::Solid(WHITE), stroke_width) {
            let mut paint = shader.paint(&uniforms);
            paint.set_blend_mode(BlendMode::DestinationIn);
            builder.draw_rect(&self.quad(stroke_width), &paint);
        }
    }

    fn uniforms(&self, brush: &Brush, stroke_width: f32) -> Option<ShapeUniforms> {
        ShapeUniforms::new(self.rect, self.radii, self.squircle, stroke_width, brush)
    }

    fn quad(&self, stroke_width: f32) -> impellers::Rect {
        let reach = stroke_width.max(0.0) * 0.5 + AA_MARGIN;
        to_rect(self.rect.inset(Insets::uniform(-reach)))
    }
}

/// The CSS factor that shrinks every radius until no two on one side overlap.
fn css_radius_scale(rect: Rect, radii: CornerRadii) -> f32 {
    let (width, height) = (rect.size.width, rect.size.height);
    [
        (width, radii.top_left + radii.top_right),
        (width, radii.bottom_left + radii.bottom_right),
        (height, radii.top_left + radii.bottom_left),
        (height, radii.top_right + radii.bottom_right),
    ]
    .into_iter()
    .filter(|(_, sum)| *sum > 0.0)
    .map(|(side, sum)| side / sum)
    .fold(1.0, f32::min)
}

fn scale_radii(radii: CornerRadii, scale: f32) -> CornerRadii {
    let fit = |radius: f32| (radius * scale).max(0.0);
    CornerRadii {
        top_left: fit(radii.top_left),
        top_right: fit(radii.top_right),
        bottom_right: fit(radii.bottom_right),
        bottom_left: fit(radii.bottom_left),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(width: f32, height: f32) -> Rect {
        Rect::from_xywh(0.0, 0.0, width, height)
    }

    #[test]
    fn oversized_round_radii_shrink_like_css() {
        let shape = SdfShape::new(
            rect(100.0, 40.0),
            CornerRadii {
                top_left: 30.0,
                bottom_left: 30.0,
                ..CornerRadii::ZERO
            },
            CornerShape::Round,
        );
        assert_eq!(shape.radii.top_left, 20.0);
        assert_eq!(shape.radii.bottom_left, 20.0);
        assert!(!shape.squircle);
    }

    #[test]
    fn squircle_pills_are_drawn_round() {
        let pill = SdfShape::new(
            rect(80.0, 24.0),
            CornerRadii::uniform(999.0),
            CornerShape::Squircle,
        );
        assert!(!pill.squircle);
        assert_eq!(pill.radii, CornerRadii::uniform(12.0));
        let card = SdfShape::new(
            rect(200.0, 120.0),
            CornerRadii::uniform(16.0),
            CornerShape::Squircle,
        );
        assert!(card.squircle);
        assert_eq!(card.radii, CornerRadii::uniform(16.0));
    }

    #[test]
    fn the_quad_covers_the_stroke_and_the_antialiased_edge() {
        let shape = SdfShape::new(rect(40.0, 20.0), CornerRadii::ZERO, CornerShape::Round);
        let quad = shape.quad(4.0);
        assert_eq!((quad.origin.x, quad.origin.y), (-3.0, -3.0));
        assert_eq!((quad.size.width, quad.size.height), (46.0, 26.0));
    }
}
