use geometry::{CornerRadii, CornerShape, Rect};
use impellers::{ClipOperation, DisplayListBuilder, FillType, Paint, PathBuilder, RoundingRadii};

use super::{
    geometry::{to_radii, to_rect},
    squircle::squircle_path,
};

/// A rounded rectangle in the form Impeller draws fastest for its corner shape.
pub(crate) enum Outline {
    Square(impellers::Rect),
    Round(impellers::Rect, RoundingRadii),
    Path(impellers::Path),
}

impl Outline {
    pub(crate) fn new(rect: Rect, radii: CornerRadii, shape: CornerShape) -> Self {
        match shape {
            _ if radii.is_zero() => Self::Square(to_rect(rect)),
            CornerShape::Round => Self::Round(to_rect(rect), to_radii(radii)),
            CornerShape::Squircle if is_capsule(rect, radii) => Self::Path(capsule_path(rect)),
            CornerShape::Squircle => Self::Path(squircle_path(rect, radii)),
        }
    }

    pub(crate) fn draw(&self, builder: &mut DisplayListBuilder, paint: &Paint) {
        match self {
            Self::Square(rect) => builder.draw_rect(rect, paint),
            Self::Round(rect, radii) => builder.draw_rounded_rect(rect, radii, paint),
            Self::Path(path) => builder.draw_path(path, paint),
        };
    }

    pub(crate) fn clip(&self, builder: &mut DisplayListBuilder) {
        let operation = ClipOperation::Intersect;
        match self {
            Self::Square(rect) => builder.clip_rect(rect, operation),
            Self::Round(rect, radii) => builder.clip_rounded_rect(rect, radii, operation),
            Self::Path(path) => builder.clip_path(path, operation),
        };
    }
}

/// Whether every corner reaches half the shorter side — a pill or a circle, whose ends a
/// squircle capped at that reach would flatten into a rounded square.
pub(crate) fn is_capsule(rect: Rect, radii: CornerRadii) -> bool {
    let half_short_side = rect.size.width.min(rect.size.height) * 0.5;
    [
        radii.top_left,
        radii.top_right,
        radii.bottom_right,
        radii.bottom_left,
    ]
    .into_iter()
    .all(|radius| radius >= half_short_side)
}

/// `rect` with circular ends, as a path: Impeller honours path clips offscreen, where it drops a
/// rounded rect clip's radii.
fn capsule_path(rect: Rect) -> impellers::Path {
    let radius = rect.size.width.min(rect.size.height) * 0.5;
    let mut builder = PathBuilder::default();
    builder.add_rounded_rect(&to_rect(rect), &to_radii(CornerRadii::uniform(radius)));
    builder.take_path_new(FillType::NonZero)
}

#[cfg(test)]
mod tests {
    use geometry::{Point, Size};

    use super::*;

    fn rect(size: Size) -> Rect {
        Rect::new(Point::new(0.0, 0.0), size)
    }

    #[test]
    fn pills_and_circles_keep_circular_ends() {
        assert!(is_capsule(
            rect(Size::new(80.0, 24.0)),
            CornerRadii::uniform(12.0)
        ));
        assert!(is_capsule(
            rect(Size::new(20.0, 20.0)),
            CornerRadii::uniform(999.0)
        ));
    }

    #[test]
    fn cards_and_half_pills_take_the_squircle() {
        assert!(!is_capsule(
            rect(Size::new(296.0, 200.0)),
            CornerRadii::uniform(16.0)
        ));
        let half_pill = CornerRadii {
            top_right: 0.0,
            ..CornerRadii::uniform(12.0)
        };
        assert!(!is_capsule(rect(Size::new(80.0, 24.0)), half_pill));
    }
}
