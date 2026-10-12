use geometry::{Point, Rect};

/// One segment of a [`Path`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PathCommand {
    /// Starts a new contour.
    MoveTo(Point),
    /// Straight line.
    LineTo(Point),
    /// Quadratic Bézier with one control point.
    QuadTo(Point, Point),
    /// Cubic Bézier with two control points.
    CubicTo(Point, Point, Point),
    /// Closes the current contour.
    Close,
}

/// A vector outline in logical pixels.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Path {
    /// Segments in order.
    pub commands: Vec<PathCommand>,
}

impl Path {
    /// An empty path.
    pub fn new() -> Self {
        Self::default()
    }

    /// Starts a contour at `point`.
    pub fn move_to(mut self, point: Point) -> Self {
        self.commands.push(PathCommand::MoveTo(point));
        self
    }

    /// Adds a line to `point`.
    pub fn line_to(mut self, point: Point) -> Self {
        self.commands.push(PathCommand::LineTo(point));
        self
    }

    /// Adds a quadratic curve.
    pub fn quad_to(mut self, control: Point, point: Point) -> Self {
        self.commands.push(PathCommand::QuadTo(control, point));
        self
    }

    /// Adds a cubic curve.
    pub fn cubic_to(mut self, first: Point, second: Point, point: Point) -> Self {
        self.commands.push(PathCommand::CubicTo(first, second, point));
        self
    }

    /// Closes the current contour.
    pub fn close(mut self) -> Self {
        self.commands.push(PathCommand::Close);
        self
    }

    /// The bounding box of all points, control points included.
    pub fn bounds(&self) -> Rect {
        let points = self.commands.iter().flat_map(|command| match *command {
            PathCommand::MoveTo(point) | PathCommand::LineTo(point) => [Some(point), None, None],
            PathCommand::QuadTo(control, point) => [Some(control), Some(point), None],
            PathCommand::CubicTo(first, second, point) => [Some(first), Some(second), Some(point)],
            PathCommand::Close => [None, None, None],
        });
        let mut points = points.flatten();
        let Some(first) = points.next() else {
            return Rect::ZERO;
        };
        let (min, max) = points.fold((first, first), |(min, max), point| {
            (
                Point::new(min.x.min(point.x), min.y.min(point.y)),
                Point::new(max.x.max(point.x), max.y.max(point.y)),
            )
        });
        Rect::from_xywh(min.x, min.y, max.x - min.x, max.y - min.y)
    }
}
