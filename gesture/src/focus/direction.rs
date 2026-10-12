use geometry::Point;
use input::NamedKey;

use crate::CROSS_AXIS_WEIGHT;

/// A direction for arrow-key focus navigation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    /// Towards smaller y.
    Up,
    /// Towards larger y.
    Down,
    /// Towards smaller x.
    Left,
    /// Towards larger x.
    Right,
}

impl Direction {
    /// The direction of an arrow key.
    pub fn from_arrow(key: NamedKey) -> Option<Self> {
        match key {
            NamedKey::ArrowUp => Some(Self::Up),
            NamedKey::ArrowDown => Some(Self::Down),
            NamedKey::ArrowLeft => Some(Self::Left),
            NamedKey::ArrowRight => Some(Self::Right),
            _ => None,
        }
    }

    /// How far `to` lies from `from` in this direction: the distance along the direction's axis
    /// plus [`CROSS_AXIS_WEIGHT`] times the offset across it, or `None` when `to` is not strictly
    /// ahead.
    pub fn weighted_distance(self, from: Point, to: Point) -> Option<f32> {
        let offset = to - from;
        let (ahead, across) = match self {
            Self::Up => (-offset.y, offset.x),
            Self::Down => (offset.y, offset.x),
            Self::Left => (-offset.x, offset.y),
            Self::Right => (offset.x, offset.y),
        };
        (ahead > 0.0).then(|| ahead + CROSS_AXIS_WEIGHT * across.abs())
    }
}
