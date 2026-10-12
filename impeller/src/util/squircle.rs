use geometry::{CornerRadii, Point, Rect, Vector};
use impellers::{FillType, PathBuilder};

use super::geometry::to_point;

/// How far along each edge a corner reaches, as a multiple of its radius; the compositor's
/// `SQUIRCLE_EXTENT`, so a client's shapes stay concentric with the glass drawn under them.
const EXTENT: f32 = 1.5;
/// The cubic handle, as a fraction of the reach, that best fits a quarter of `x⁴ + y⁴ = 1`:
/// it strays from the curve by under 0.5% of the reach.
const HANDLE: f32 = 0.919_21;

/// One corner as a cubic: where it leaves the incoming edge, its two handles, and where it
/// joins the outgoing edge.
type Corner = [Point; 4];

/// `rect` with squircle corners of `radii`.
pub(crate) fn squircle_path(rect: Rect, radii: CornerRadii) -> impellers::Path {
    let corners = corners(rect, radii);
    let mut builder = PathBuilder::default();
    builder.move_to(to_point(corners[3][3]));
    for [start, first, second, end] in corners {
        builder.line_to(to_point(start));
        builder.cubic_curve_to(to_point(first), to_point(second), to_point(end));
    }
    builder.close();
    builder.take_path_new(FillType::NonZero)
}

/// The top-right, bottom-right, bottom-left and top-left corners, clockwise. A reach is capped
/// at half the shorter side, as the compositor caps it, so a pill keeps straight sides.
fn corners(rect: Rect, radii: CornerRadii) -> [Corner; 4] {
    let cap = rect.size.width.min(rect.size.height) * 0.5;
    let (left, top, right, bottom) = (rect.min_x(), rect.min_y(), rect.max_x(), rect.max_y());
    [
        (
            Point::new(right, top),
            Vector::new(1.0, 0.0),
            radii.top_right,
        ),
        (
            Point::new(right, bottom),
            Vector::new(0.0, 1.0),
            radii.bottom_right,
        ),
        (
            Point::new(left, bottom),
            Vector::new(-1.0, 0.0),
            radii.bottom_left,
        ),
        (
            Point::new(left, top),
            Vector::new(0.0, -1.0),
            radii.top_left,
        ),
    ]
    .map(|(corner, incoming, radius)| {
        let reach = (radius * EXTENT).clamp(0.0, cap);
        let slack = reach * (1.0 - HANDLE);
        let outgoing = Vector::new(-incoming.y, incoming.x);
        [
            corner - incoming * reach,
            corner - incoming * slack,
            corner + outgoing * slack,
            corner + outgoing * reach,
        ]
    })
}

#[cfg(test)]
mod tests {
    use geometry::Size;

    use super::*;

    fn cubic([start, first, second, end]: Corner, t: f32) -> Point {
        let u = 1.0 - t;
        let weights = [u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t];
        let (x, y) = [start, first, second, end]
            .iter()
            .zip(weights)
            .fold((0.0, 0.0), |(x, y), (point, weight)| {
                (x + point.x * weight, y + point.y * weight)
            });
        Point::new(x, y)
    }

    #[test]
    fn every_corner_follows_the_compositor_superellipse() {
        let rect = Rect::new(Point::new(10.0, 20.0), Size::new(200.0, 120.0));
        let radius = 20.0;
        let reach = radius * EXTENT;
        let centers = [
            Point::new(210.0 - reach, 20.0 + reach),
            Point::new(210.0 - reach, 140.0 - reach),
            Point::new(10.0 + reach, 140.0 - reach),
            Point::new(10.0 + reach, 20.0 + reach),
        ];
        for (corner, center) in corners(rect, CornerRadii::uniform(radius))
            .into_iter()
            .zip(centers)
        {
            for step in 0..=32 {
                let offset = cubic(corner, step as f32 / 32.0) - center;
                let norm = (offset.x.powi(4) + offset.y.powi(4)).sqrt().sqrt();
                assert!(
                    (norm - reach).abs() < reach * 0.005,
                    "{norm} strays from {reach}"
                );
            }
        }
    }

    #[test]
    fn a_pill_reaches_only_half_its_height() {
        let rect = Rect::new(Point::new(0.0, 0.0), Size::new(80.0, 24.0));
        let [top_right, ..] = corners(rect, CornerRadii::uniform(12.0));
        assert_eq!(top_right[0], Point::new(68.0, 0.0));
        assert_eq!(top_right[3], Point::new(80.0, 12.0));
    }

    #[test]
    fn a_square_corner_stays_on_the_corner() {
        let rect = Rect::new(Point::new(0.0, 0.0), Size::new(40.0, 40.0));
        let radii = CornerRadii {
            top_right: 0.0,
            ..CornerRadii::uniform(8.0)
        };
        let [top_right, ..] = corners(rect, radii);
        assert!(
            top_right
                .iter()
                .all(|point| *point == Point::new(40.0, 0.0))
        );
    }
}
