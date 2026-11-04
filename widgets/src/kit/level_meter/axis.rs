use geometry::{Point, Rect, Size};

/// Which way a meter rises: rightwards, or upwards from the bottom.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum MeterAxis {
    Horizontal,
    Vertical,
}

impl MeterAxis {
    pub(super) fn length(self, size: Size) -> f32 {
        match self {
            Self::Horizontal => size.width,
            Self::Vertical => size.height,
        }
    }

    pub(super) fn thickness(self, size: Size) -> f32 {
        match self {
            Self::Horizontal => size.height,
            Self::Vertical => size.width,
        }
    }

    pub(super) fn stretch(self, size: Size, from: f32, to: f32) -> Rect {
        match self {
            Self::Horizontal => Rect::new(Point::new(from, 0.0), Size::new(to - from, size.height)),
            Self::Vertical => Rect::new(
                Point::new(0.0, size.height - to),
                Size::new(size.width, to - from),
            ),
        }
    }
}
