use geometry::{Color, Rect};
use graphics::{Stroke, StrokeCap};

use super::uniform_bytes::uniform_bytes;

const FLOATS: usize = 12;
const BYTES: usize = FLOATS * size_of::<f32>();

/// The uniforms of `impeller/shaders/flattened_path.frag`, in declaration order.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PathUniforms([f32; FLOATS]);

impl PathUniforms {
    /// A fill when `stroke` is `None`. `bounds` is the box the segment texture's coordinates are
    /// fractions of.
    pub(crate) fn new(
        bounds: Rect,
        segment_count: usize,
        stroke: Option<Stroke>,
        color: Color,
    ) -> Self {
        let half_width = stroke.map_or(0.0, |stroke| stroke.width * 0.5);
        let cap = stroke.map(|stroke| stroke.cap);
        let round_caps = f32::from(u8::from(cap == Some(StrokeCap::Round)));
        let square_extension = if cap == Some(StrokeCap::Square) {
            half_width
        } else {
            0.0
        };
        Self([
            bounds.origin.x,
            bounds.origin.y,
            bounds.size.width,
            bounds.size.height,
            segment_count as f32,
            half_width,
            round_caps,
            square_extension,
            color.r,
            color.g,
            color.b,
            color.a,
        ])
    }

    pub(crate) fn to_bytes(&self) -> [u8; BYTES] {
        uniform_bytes(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use geometry::rgba;

    use super::*;

    fn icon() -> Rect {
        Rect::from_xywh(2.0, 3.0, 20.0, 18.0)
    }

    fn stroke(cap: StrokeCap) -> Option<Stroke> {
        Some(Stroke {
            cap,
            ..Stroke::new(2.0)
        })
    }

    #[test]
    fn bounds_style_and_color_follow_the_declaration_order() {
        let color = rgba(0xff000080);
        let PathUniforms(floats) = PathUniforms::new(icon(), 7, stroke(StrokeCap::Round), color);
        assert_eq!(
            floats,
            [
                2.0, 3.0, 20.0, 18.0, 7.0, 1.0, 1.0, 0.0, color.r, color.g, color.b, color.a
            ]
        );
        let bytes = PathUniforms::new(icon(), 7, None, color).to_bytes();
        assert_eq!(bytes.len(), 48);
        assert_eq!(bytes[16..20], 7.0f32.to_ne_bytes());
    }

    #[test]
    fn caps_set_the_round_flag_or_the_square_extension() {
        let style = |stroke| {
            let PathUniforms(floats) = PathUniforms::new(icon(), 3, stroke, rgba(0xffffffff));
            [floats[4], floats[5], floats[6], floats[7]]
        };
        assert_eq!(style(None), [3.0, 0.0, 0.0, 0.0]);
        assert_eq!(style(stroke(StrokeCap::Butt)), [3.0, 1.0, 0.0, 0.0]);
        assert_eq!(style(stroke(StrokeCap::Round)), [3.0, 1.0, 1.0, 0.0]);
        assert_eq!(style(stroke(StrokeCap::Square)), [3.0, 1.0, 0.0, 1.0]);
    }
}
