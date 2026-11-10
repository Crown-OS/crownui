use geometry::{Color, CornerRadii, Insets, Rect, Vector};
use graphics::{Brush, DrawList, Paint};

use crate::kit::GradientStops;

/// A blurred shadow cast `y` below the shape.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct DropShadow {
    pub color: Color,
    pub y: f32,
    pub blur: f32,
}

/// A shadow cast inside the shape's edge, as if the shape were a hole in a sheet moved by
/// `offset` and blurred by `blur` (CSS inset `box-shadow`). A white one without blur is light
/// caught along an edge.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct InnerShadow {
    pub color: Color,
    pub offset: Vector,
    pub blur: f32,
}

/// How far past its edge a blur of radius one carries a visible shadow: three sigmas.
const BLUR_REACH: f32 = 1.5;

impl InnerShadow {
    /// Light caught along the inside of the top edge, `y` deep.
    pub(crate) fn highlight(color: Color, y: f32) -> Self {
        Self {
            color,
            offset: Vector::new(0.0, y),
            blur: 0.0,
        }
    }

    /// Records the shadow inside `rect` with corners `radii`: a band around the moved shape,
    /// wide enough to cover everything it shades, clipped to the shape.
    pub(crate) fn paint(&self, list: &mut DrawList, rect: Rect, radii: CornerRadii) {
        let width = self.offset.x.abs().max(self.offset.y.abs()) + self.blur * BLUR_REACH;
        let half = width / 2.0;
        let paint = Paint {
            brush: Brush::Solid(self.color),
            blur: (self.blur > 0.0).then_some(self.blur / 2.0),
        };
        list.save();
        list.clip(rect, radii);
        list.stroke(
            rect.translate(self.offset).inset(Insets::uniform(-half)),
            grow(radii, half),
            width,
            paint,
        );
        list.restore();
    }
}

/// A hairline just inside the shape's edge.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct InsetRing {
    pub color: Color,
    pub width: f32,
}

/// A raised surface the view's flat background cannot draw: a top-to-bottom gradient with a drop
/// shadow under it, a shadow or highlight inside its edge and a ring inside its rim.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct SurfacePaint {
    pub fill: GradientStops,
    pub drop: Option<DropShadow>,
    pub inner: Option<InnerShadow>,
    pub ring: Option<InsetRing>,
}

impl SurfacePaint {
    /// Records the surface filling `rect` with corners `radii`.
    pub(crate) fn paint(&self, list: &mut DrawList, rect: Rect, radii: CornerRadii) {
        if let Some(DropShadow { color, y, blur }) = self.drop {
            list.shadow(rect, radii, color, blur, Vector::new(0.0, y));
        }
        list.fill(rect, radii, self.fill.vertical(rect.min_y(), rect.max_y()));
        if let Some(InsetRing { color, width }) = self.ring {
            let half = width / 2.0;
            list.stroke(
                rect.inset(Insets::uniform(half)),
                grow(radii, -half),
                width,
                color,
            );
        }
        if let Some(inner) = self.inner {
            inner.paint(list, rect, radii);
        }
    }
}

fn grow(radii: CornerRadii, amount: f32) -> CornerRadii {
    let grow = |radius: f32| (radius + amount).max(0.0);
    CornerRadii {
        top_left: grow(radii.top_left),
        top_right: grow(radii.top_right),
        bottom_right: grow(radii.bottom_right),
        bottom_left: grow(radii.bottom_left),
    }
}

#[cfg(test)]
mod tests {
    use geometry::{BLACK, Point, Size, WHITE};
    use graphics::DrawCommand;

    use super::*;

    fn circle() -> (Rect, CornerRadii) {
        (
            Rect::new(Point::ZERO, Size::new(28.0, 28.0)),
            CornerRadii::uniform(14.0),
        )
    }

    #[test]
    fn a_plain_surface_is_one_gradient_fill() {
        let (rect, radii) = circle();
        let mut list = DrawList::new();
        SurfacePaint {
            fill: GradientStops::new(WHITE, BLACK),
            drop: None,
            inner: None,
            ring: None,
        }
        .paint(&mut list, rect, radii);
        assert_eq!(
            list.commands(),
            [DrawCommand::FillRoundedRect(
                rect,
                radii,
                GradientStops::new(WHITE, BLACK).vertical(0.0, 28.0)
            )]
        );
    }

    #[test]
    fn the_shadow_goes_under_and_the_highlight_is_clipped_to_the_shape() {
        let (rect, radii) = circle();
        let mut list = DrawList::new();
        SurfacePaint {
            fill: GradientStops::flat(WHITE),
            drop: Some(DropShadow {
                color: BLACK,
                y: 2.0,
                blur: 6.0,
            }),
            inner: Some(InnerShadow::highlight(WHITE, 1.0)),
            ring: Some(InsetRing {
                color: BLACK,
                width: 1.0,
            }),
        }
        .paint(&mut list, rect, radii);
        let commands = list.commands();
        assert!(
            matches!(commands[0], DrawCommand::Shadow { offset, .. } if offset == Vector::new(0.0, 2.0))
        );
        assert!(matches!(commands[1], DrawCommand::FillRoundedRect(..)));
        assert!(matches!(
            commands[2],
            DrawCommand::StrokeRoundedRect(ring, ring_radii, ..)
                if ring == Rect::from_xywh(0.5, 0.5, 27.0, 27.0) && ring_radii == CornerRadii::uniform(13.5)
        ));
        assert_eq!(commands[3], DrawCommand::Save);
        assert_eq!(commands[4], DrawCommand::ClipRoundedRect(rect, radii));
        assert!(matches!(
            commands[5],
            DrawCommand::StrokeRoundedRect(band, band_radii, ..)
                if band == Rect::from_xywh(-0.5, 0.5, 29.0, 29.0) && band_radii == CornerRadii::uniform(14.5)
        ));
        assert_eq!(commands[6], DrawCommand::Restore);
    }
}
