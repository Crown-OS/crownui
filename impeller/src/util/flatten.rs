use geometry::Point;
use graphics::{Path, PathCommand};

/// Most straight pieces one curve is cut into, whatever its size.
const MAX_CURVE_PIECES: usize = 256;

/// A straight piece of a flattened path. `start_cap` and `end_cap` mark the ends of an open
/// contour that a stroke caps instead of joining.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Segment {
    pub(crate) from: Point,
    pub(crate) to: Point,
    pub(crate) start_cap: bool,
    pub(crate) end_cap: bool,
}

/// What a contour that ends without `Close` becomes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum OpenContours {
    /// Left open, capped at both ends, as strokes draw it.
    Capped,
    /// Closed by a line back to its start, as fills draw it.
    Closed,
}

/// How a path is cut into segments.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Flattening {
    /// The furthest a segment may stray from its curve, in path units.
    pub(crate) tolerance: f32,
    pub(crate) open_contours: OpenContours,
}

/// Cuts `path` into line segments in drawing order.
pub(crate) fn flatten(path: &Path, flattening: Flattening) -> Vec<Segment> {
    let mut flattener = Flattener {
        segments: Vec::with_capacity(path.commands.len()),
        flattening,
        start: Point::ZERO,
        current: Point::ZERO,
        contour_begins: 0,
    };
    for command in &path.commands {
        match *command {
            PathCommand::MoveTo(point) => flattener.move_to(point),
            PathCommand::LineTo(point) => flattener.line_to(point),
            PathCommand::QuadTo(control, point) => flattener.quad_to(control, point),
            PathCommand::CubicTo(first, second, point) => flattener.cubic_to(first, second, point),
            PathCommand::Close => flattener.close(),
        }
    }
    flattener.end_contour();
    flattener.segments
}

struct Flattener {
    segments: Vec<Segment>,
    flattening: Flattening,
    start: Point,
    current: Point,
    contour_begins: usize,
}

impl Flattener {
    fn move_to(&mut self, point: Point) {
        self.end_contour();
        self.start = point;
        self.current = point;
    }

    fn line_to(&mut self, point: Point) {
        self.segments.push(Segment {
            from: self.current,
            to: point,
            start_cap: false,
            end_cap: false,
        });
        self.current = point;
    }

    /// Uniform steps in `t`: a quadratic strays at most `|p0 - 2p1 + p2| / (4n²)` from n chords.
    fn quad_to(&mut self, control: Point, end: Point) {
        let from = self.current;
        let bend = (from - control) + (end - control);
        for t in inner_steps(bend.length() / 4.0, self.flattening.tolerance) {
            let along = 2.0 * t * (1.0 - t);
            self.line_to(from + (control - from) * along + (end - from) * (t * t));
        }
        self.line_to(end);
    }

    /// Uniform steps in `t`: a cubic strays at most `3 max|p_i - 2p_i+1 + p_i+2| / (4n²)`.
    fn cubic_to(&mut self, first: Point, second: Point, end: Point) {
        let from = self.current;
        let bend = |before: Point, at: Point, after: Point| ((before - at) + (after - at)).length();
        let deviation = bend(from, first, second).max(bend(first, second, end)) * 0.75;
        for t in inner_steps(deviation, self.flattening.tolerance) {
            let rest = 1.0 - t;
            self.line_to(
                from + (first - from) * (3.0 * t * rest * rest)
                    + (second - from) * (3.0 * t * t * rest)
                    + (end - from) * (t * t * t),
            );
        }
        self.line_to(end);
    }

    fn close(&mut self) {
        if self.current != self.start {
            self.line_to(self.start);
        }
        self.contour_begins = self.segments.len();
    }

    fn end_contour(&mut self) {
        match self.flattening.open_contours {
            OpenContours::Closed => self.close(),
            OpenContours::Capped => {
                let contour = &mut self.segments[self.contour_begins..];
                if let [first, ..] = contour {
                    first.start_cap = true;
                }
                if let [.., last] = contour {
                    last.end_cap = true;
                }
                self.contour_begins = self.segments.len();
            }
        }
    }
}

/// The parameters between 0 and 1 that cut a curve into the fewest equal steps in `t` whose
/// chords, straying at most `deviation / n²`, stay within `tolerance`.
fn inner_steps(deviation: f32, tolerance: f32) -> impl Iterator<Item = f32> {
    let pieces = ((deviation / tolerance).sqrt().ceil() as usize).clamp(1, MAX_CURVE_PIECES);
    (1..pieces).map(move |step| step as f32 / pieces as f32)
}

#[cfg(test)]
mod tests {
    use geometry::Vector;

    use super::*;

    /// The Bézier handle length that best approximates a quarter circle.
    const KAPPA: f32 = 0.552_284_8;

    fn stroke(tolerance: f32) -> Flattening {
        Flattening {
            tolerance,
            open_contours: OpenContours::Capped,
        }
    }

    fn fill(tolerance: f32) -> Flattening {
        Flattening {
            tolerance,
            open_contours: OpenContours::Closed,
        }
    }

    fn distance_to_segment(point: Point, from: Point, to: Point) -> f32 {
        let axis: Vector = to - from;
        let offset = point - from;
        let along = (offset.x * axis.x + offset.y * axis.y) / (axis.x * axis.x + axis.y * axis.y);
        (offset - axis * along.clamp(0.0, 1.0)).length()
    }

    fn cubic_point(points: [Point; 4], t: f32) -> Point {
        let [from, first, second, end] = points;
        let rest = 1.0 - t;
        from + (first - from) * (3.0 * t * rest * rest)
            + (second - from) * (3.0 * t * t * rest)
            + (end - from) * (t * t * t)
    }

    #[test]
    fn a_quarter_circle_stays_within_tolerance() {
        let radius = 40.0;
        let handle = radius * KAPPA;
        let points = [
            Point::new(radius, 0.0),
            Point::new(radius, handle),
            Point::new(handle, radius),
            Point::new(0.0, radius),
        ];
        let [from, first, second, end] = points;
        let path = Path::new().move_to(from).cubic_to(first, second, end);
        let tolerance = 0.05;
        let segments = flatten(&path, stroke(tolerance));
        assert!(segments.len() > 4, "{} segments", segments.len());
        for sample in 0..=1000 {
            let point = cubic_point(points, sample as f32 / 1000.0);
            let nearest = segments
                .iter()
                .map(|segment| distance_to_segment(point, segment.from, segment.to))
                .fold(f32::INFINITY, f32::min);
            assert!(nearest <= tolerance, "{point:?} is {nearest} away");
        }
        assert_eq!(segments.last().map(|segment| segment.to), Some(end));
    }

    #[test]
    fn a_polyline_is_unchanged() {
        let points = [
            Point::new(1.0, 2.0),
            Point::new(9.0, 2.0),
            Point::new(9.0, 7.5),
            Point::new(3.0, 11.0),
        ];
        let path = points[1..]
            .iter()
            .fold(Path::new().move_to(points[0]), |path, point| {
                path.line_to(*point)
            });
        let segments = flatten(&path, stroke(0.1));
        let ends: Vec<_> = segments
            .iter()
            .map(|segment| (segment.from, segment.to))
            .collect();
        assert_eq!(
            ends,
            [
                (points[0], points[1]),
                (points[1], points[2]),
                (points[2], points[3])
            ]
        );
        let caps: Vec<_> = segments
            .iter()
            .map(|segment| (segment.start_cap, segment.end_cap))
            .collect();
        assert_eq!(caps, [(true, false), (false, false), (false, true)]);
    }

    #[test]
    fn close_emits_the_closing_segment_and_no_caps() {
        let path = Path::new()
            .move_to(Point::new(0.0, 0.0))
            .line_to(Point::new(4.0, 0.0))
            .line_to(Point::new(4.0, 3.0))
            .close();
        let segments = flatten(&path, stroke(0.1));
        assert_eq!(segments.len(), 3);
        assert_eq!(segments[2].from, Point::new(4.0, 3.0));
        assert_eq!(segments[2].to, Point::new(0.0, 0.0));
        assert!(
            segments
                .iter()
                .all(|segment| !segment.start_cap && !segment.end_cap)
        );
    }

    #[test]
    fn fills_close_every_subpath() {
        let path = Path::new()
            .move_to(Point::new(0.0, 0.0))
            .line_to(Point::new(4.0, 0.0))
            .line_to(Point::new(4.0, 3.0))
            .move_to(Point::new(10.0, 10.0))
            .line_to(Point::new(12.0, 10.0))
            .line_to(Point::new(12.0, 12.0));
        let segments = flatten(&path, fill(0.1));
        assert_eq!(segments.len(), 6);
        assert_eq!(segments[2].to, Point::new(0.0, 0.0));
        assert_eq!(segments[5].to, Point::new(10.0, 10.0));
        assert!(
            segments
                .iter()
                .all(|segment| !segment.start_cap && !segment.end_cap)
        );
    }

    #[test]
    fn strokes_cap_each_open_subpath() {
        let path = Path::new()
            .move_to(Point::new(0.0, 0.0))
            .line_to(Point::new(4.0, 0.0))
            .move_to(Point::new(0.0, 5.0))
            .quad_to(Point::new(2.0, 9.0), Point::new(4.0, 5.0));
        let segments = flatten(&path, stroke(0.01));
        assert!(segments[0].start_cap && segments[0].end_cap);
        assert!(segments[1].start_cap && !segments[1].end_cap);
        assert!(segments.last().is_some_and(|last| last.end_cap));
        assert_eq!(
            segments.iter().filter(|segment| segment.start_cap).count(),
            2
        );
    }
}
