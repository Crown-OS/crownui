use crate::{Insets, Point, Size, Vector};

/// An axis-aligned rectangle in logical pixels.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Rect {
    /// Top-left corner.
    pub origin: Point,
    /// Extent from the origin.
    pub size: Size,
}

impl Rect {
    /// The empty rectangle at the origin.
    pub const ZERO: Self = Self::new(Point::ZERO, Size::ZERO);

    /// Creates a rectangle from its top-left corner and size.
    pub const fn new(origin: Point, size: Size) -> Self {
        Self { origin, size }
    }

    /// Creates a rectangle from `x`, `y`, `width` and `height`.
    pub const fn from_xywh(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self::new(Point::new(x, y), Size::new(width, height))
    }

    /// Left edge.
    pub fn min_x(self) -> f32 {
        self.origin.x
    }

    /// Top edge.
    pub fn min_y(self) -> f32 {
        self.origin.y
    }

    /// Right edge.
    pub fn max_x(self) -> f32 {
        self.origin.x + self.size.width
    }

    /// Bottom edge.
    pub fn max_y(self) -> f32 {
        self.origin.y + self.size.height
    }

    /// Center point.
    pub fn center(self) -> Point {
        Point::new(
            self.origin.x + self.size.width / 2.0,
            self.origin.y + self.size.height / 2.0,
        )
    }

    /// Whether `point` lies inside, counting the top and left edges only.
    pub fn contains(self, point: Point) -> bool {
        point.x >= self.min_x()
            && point.x < self.max_x()
            && point.y >= self.min_y()
            && point.y < self.max_y()
    }

    /// Moves the rectangle by `offset`.
    pub fn translate(self, offset: Vector) -> Self {
        Self::new(self.origin + offset, self.size)
    }

    /// Shrinks the rectangle by `insets`, never below zero size.
    pub fn inset(self, insets: Insets) -> Self {
        Self::from_xywh(
            self.origin.x + insets.left,
            self.origin.y + insets.top,
            (self.size.width - insets.horizontal()).max(0.0),
            (self.size.height - insets.vertical()).max(0.0),
        )
    }

    /// The smallest rectangle covering both.
    pub fn union(self, other: Self) -> Self {
        if self.size.is_empty() {
            return other;
        }
        if other.size.is_empty() {
            return self;
        }
        let min_x = self.min_x().min(other.min_x());
        let min_y = self.min_y().min(other.min_y());
        Self::from_xywh(
            min_x,
            min_y,
            self.max_x().max(other.max_x()) - min_x,
            self.max_y().max(other.max_y()) - min_y,
        )
    }

    /// The overlap of both, if any.
    pub fn intersection(self, other: Self) -> Option<Self> {
        let min_x = self.min_x().max(other.min_x());
        let min_y = self.min_y().max(other.min_y());
        let max_x = self.max_x().min(other.max_x());
        let max_y = self.max_y().min(other.max_y());
        (max_x > min_x && max_y > min_y)
            .then(|| Self::from_xywh(min_x, min_y, max_x - min_x, max_y - min_y))
    }

    /// Rounds the edges outwards onto the device pixel grid at `scale`.
    pub fn snap_out(self, scale: f32) -> Self {
        let snap_down = |value: f32| (value * scale).floor() / scale;
        let snap_up = |value: f32| (value * scale).ceil() / scale;
        let (min_x, min_y) = (snap_down(self.min_x()), snap_down(self.min_y()));
        Self::from_xywh(
            min_x,
            min_y,
            snap_up(self.max_x()) - min_x,
            snap_up(self.max_y()) - min_y,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn union_ignores_empty_rectangles() {
        let rect = Rect::from_xywh(1.0, 2.0, 3.0, 4.0);
        assert_eq!(Rect::ZERO.union(rect), rect);
        assert_eq!(
            rect.union(Rect::from_xywh(10.0, 0.0, 1.0, 1.0)),
            Rect::from_xywh(1.0, 0.0, 10.0, 6.0)
        );
    }

    #[test]
    fn disjoint_rectangles_have_no_intersection() {
        let left = Rect::from_xywh(0.0, 0.0, 1.0, 1.0);
        assert_eq!(left.intersection(Rect::from_xywh(2.0, 0.0, 1.0, 1.0)), None);
    }

    #[test]
    fn snapping_covers_the_original_at_fractional_scale() {
        let snapped = Rect::from_xywh(0.3, 0.3, 10.1, 10.1).snap_out(1.25);
        assert_eq!(snapped, Rect::from_xywh(0.0, 0.0, 11.2, 11.2));
    }
}
