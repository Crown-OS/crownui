use geometry::Point;
use graphics::{Path, PathCommand};

const MAX_SEGMENT_SWEEP: f32 = std::f32::consts::FRAC_PI_2;

/// A circular arc as cubic Béziers: starts at `start` radians (0 is 3 o'clock, clockwise on
/// screen) and sweeps `sweep` radians.
pub(crate) fn arc_path(center: Point, radius: f32, start: f32, sweep: f32) -> Path {
    let segments = (sweep.abs() / MAX_SEGMENT_SWEEP).ceil().max(1.0) as u32;
    let step = sweep / segments as f32;
    let handle = 4.0 / 3.0 * (step / 4.0).tan();
    let on_circle = |angle: f32| {
        Point::new(
            center.x + radius * angle.cos(),
            center.y + radius * angle.sin(),
        )
    };
    let tangent = |angle: f32| Point::new(-radius * angle.sin(), radius * angle.cos());
    let mut commands = vec![PathCommand::MoveTo(on_circle(start))];
    for index in 0..segments {
        let from = start + step * index as f32;
        let to = from + step;
        let (from_point, to_point) = (on_circle(from), on_circle(to));
        let (from_tangent, to_tangent) = (tangent(from), tangent(to));
        commands.push(PathCommand::CubicTo(
            Point::new(
                from_point.x + handle * from_tangent.x,
                from_point.y + handle * from_tangent.y,
            ),
            Point::new(
                to_point.x - handle * to_tangent.x,
                to_point.y - handle * to_tangent.y,
            ),
            to_point,
        ));
    }
    Path { commands }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn end_point(path: &Path) -> Point {
        match path.commands.last() {
            Some(PathCommand::CubicTo(_, _, end)) => *end,
            other => panic!("expected a cubic, got {other:?}"),
        }
    }

    #[test]
    fn a_quarter_turn_from_three_oclock_ends_at_six_oclock() {
        let path = arc_path(
            Point::new(10.0, 10.0),
            5.0,
            0.0,
            std::f32::consts::FRAC_PI_2,
        );
        let end = end_point(&path);
        assert!((end.x - 10.0).abs() < 1e-4 && (end.y - 15.0).abs() < 1e-4);
        assert_eq!(path.commands.len(), 2);
    }

    #[test]
    fn long_sweeps_are_split_into_quarter_segments() {
        let path = arc_path(Point::ZERO, 1.0, 0.0, std::f32::consts::PI);
        assert_eq!(path.commands.len(), 3);
    }
}
