use geometry::{Color, CornerRadii, Insets, Rect, Vector};
use graphics::DrawList;

use crate::kit::GradientStops;

/// A blurred shadow cast `y` below the shape.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct DropShadow {
    pub color: Color,
    pub y: f32,
    pub blur: f32,
}

/// Light caught along the inside of the top edge: the shape less itself lowered by `y`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct InnerHighlight {
    pub color: Color,
    pub y: f32,
}

/// A hairline just inside the shape's edge.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct InsetRing {
    pub color: Color,
    pub width: f32,
}

/// A raised surface the view's flat background cannot draw: a top-to-bottom gradient with a drop
/// shadow under it, a highlight inside its top edge and a ring inside its rim.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct SurfacePaint {
    pub fill: GradientStops,
    pub drop: Option<DropShadow>,
    pub highlight: Option<InnerHighlight>,
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
        if let Some(InnerHighlight { color, y }) = self.highlight {
            let half = y / 2.0;
            let lowered = rect.translate(Vector::new(0.0, y));
            list.save();
            list.clip(rect, radii);
            list.stroke(
                lowered.inset(Insets::uniform(-half)),
                grow(radii, half),
                y,
                color,
            );
            list.restore();
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
            highlight: None,
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
            highlight: Some(InnerHighlight {
                color: WHITE,
                y: 1.0,
            }),
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
