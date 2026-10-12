use geometry::{Color, CornerRadii, Rect};
use graphics::Brush;

/// Gradient stops `rounded_box.frag` interpolates; brushes with more take the masked path.
pub(crate) const MAX_GRADIENT_STOPS: usize = 8;
const FLOATS: usize = 56;
const BYTES: usize = FLOATS * size_of::<f32>();
const BOUNDS: usize = 0;
const RADII: usize = 4;
const STYLE: usize = 8;
const GRADIENT_LINE: usize = 12;
const COLORS: usize = 16;
const OFFSETS: usize = COLORS + MAX_GRADIENT_STOPS * 4;
const SOLID: f32 = 0.0;
const GRADIENT: f32 = 1.0;

/// The uniforms of `impeller/shaders/rounded_box.frag`, in declaration order.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ShapeUniforms([f32; FLOATS]);

impl ShapeUniforms {
    /// `None` when the shader cannot paint `brush` (images, or too many gradient stops).
    pub(crate) fn new(
        rect: Rect,
        radii: CornerRadii,
        squircle: bool,
        stroke_width: f32,
        brush: &Brush,
    ) -> Option<Self> {
        let mut uniforms = Self([0.0; FLOATS]);
        uniforms.set(
            BOUNDS,
            [
                rect.origin.x,
                rect.origin.y,
                rect.size.width,
                rect.size.height,
            ],
        );
        uniforms.set(
            RADII,
            [
                radii.bottom_right,
                radii.top_right,
                radii.bottom_left,
                radii.top_left,
            ],
        );
        let (kind, stops) = match brush {
            Brush::Solid(color) => {
                uniforms.set_color(0, *color);
                (SOLID, 1)
            }
            Brush::LinearGradient { stops, .. } if stops.len() > MAX_GRADIENT_STOPS => return None,
            Brush::LinearGradient { start, end, stops } => {
                uniforms.set(GRADIENT_LINE, [start.x, start.y, end.x, end.y]);
                for (index, stop) in stops.iter().enumerate() {
                    uniforms.set_color(index, stop.color);
                    uniforms.0[OFFSETS + index] = stop.offset;
                }
                (GRADIENT, stops.len())
            }
            Brush::Image { .. } => return None,
        };
        uniforms.set(
            STYLE,
            [
                f32::from(u8::from(squircle)),
                stroke_width,
                kind,
                stops as f32,
            ],
        );
        Some(uniforms)
    }

    pub(crate) fn to_bytes(&self) -> [u8; BYTES] {
        let mut bytes = [0; BYTES];
        for (chunk, value) in bytes.chunks_exact_mut(size_of::<f32>()).zip(&self.0) {
            chunk.copy_from_slice(&value.to_ne_bytes());
        }
        bytes
    }

    fn set(&mut self, start: usize, values: [f32; 4]) {
        self.0[start..start + 4].copy_from_slice(&values);
    }

    fn set_color(&mut self, index: usize, color: Color) {
        self.set(COLORS + index * 4, [color.r, color.g, color.b, color.a]);
    }
}

#[cfg(test)]
mod tests {
    use geometry::{Point, rgb, rgba};
    use graphics::{GradientStop, ImageId};

    use super::*;

    fn card() -> Rect {
        Rect::from_xywh(10.0, 20.0, 200.0, 120.0)
    }

    fn gradient(stop_count: usize) -> Brush {
        Brush::LinearGradient {
            start: Point::new(0.0, 0.0),
            end: Point::new(200.0, 0.0),
            stops: (0..stop_count)
                .map(|index| GradientStop {
                    offset: index as f32 / stop_count as f32,
                    color: rgb(0x3b82f6),
                })
                .collect(),
        }
    }

    #[test]
    fn corners_are_packed_in_the_shaders_quadrant_order() {
        let radii = CornerRadii {
            top_left: 1.0,
            top_right: 2.0,
            bottom_right: 3.0,
            bottom_left: 4.0,
        };
        let Some(ShapeUniforms(floats)) =
            ShapeUniforms::new(card(), radii, true, 2.0, &Brush::Solid(rgba(0xff000080)))
        else {
            panic!("solid brushes are painted by the shader");
        };
        assert_eq!(floats[BOUNDS..BOUNDS + 4], [10.0, 20.0, 200.0, 120.0]);
        assert_eq!(floats[RADII..RADII + 4], [3.0, 2.0, 4.0, 1.0]);
        assert_eq!(floats[STYLE..STYLE + 4], [1.0, 2.0, SOLID, 1.0]);
        assert_eq!(floats[COLORS + 3], rgba(0xff000080).a);
    }

    #[test]
    fn gradients_up_to_the_stop_limit_are_packed() {
        let Some(ShapeUniforms(floats)) = ShapeUniforms::new(
            card(),
            CornerRadii::uniform(8.0),
            false,
            0.0,
            &gradient(MAX_GRADIENT_STOPS),
        ) else {
            panic!("eight stops fit");
        };
        assert_eq!(floats[STYLE + 2..STYLE + 4], [GRADIENT, 8.0]);
        assert_eq!(
            floats[GRADIENT_LINE..GRADIENT_LINE + 4],
            [0.0, 0.0, 200.0, 0.0]
        );
        assert_eq!(floats[OFFSETS + 7], 7.0 / 8.0);
        assert_eq!(OFFSETS + MAX_GRADIENT_STOPS, FLOATS);
    }

    #[test]
    fn images_and_long_gradients_need_the_masked_path() {
        let radii = CornerRadii::uniform(8.0);
        let image = Brush::Image {
            image: ImageId(1),
            rect: card(),
        };
        assert!(ShapeUniforms::new(card(), radii, false, 0.0, &image).is_none());
        assert!(ShapeUniforms::new(card(), radii, false, 0.0, &gradient(9)).is_none());
    }

    #[test]
    fn bytes_cover_every_uniform() {
        let Some(uniforms) =
            ShapeUniforms::new(card(), CornerRadii::ZERO, false, 0.0, &Brush::Solid(rgb(0)))
        else {
            panic!("solid brushes are painted by the shader");
        };
        let bytes = uniforms.to_bytes();
        assert_eq!(bytes.len(), 224);
        assert_eq!(bytes[8..12], 200.0f32.to_ne_bytes());
    }
}
