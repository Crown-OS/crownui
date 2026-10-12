use geometry::Vector;

/// The direction a [`DragRecognizer`](crate::DragRecognizer) is locked to.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum DragAxis {
    /// Any direction.
    #[default]
    Free,
    /// Only horizontal movement; vertical-dominant drags are rejected.
    Horizontal,
    /// Only vertical movement; horizontal-dominant drags are rejected.
    Vertical,
}

impl DragAxis {
    /// Whether movement by `offset` runs along this axis at least as much as across it.
    pub fn is_along(self, offset: Vector) -> bool {
        match self {
            Self::Free => true,
            Self::Horizontal => offset.x.abs() >= offset.y.abs(),
            Self::Vertical => offset.y.abs() >= offset.x.abs(),
        }
    }

    /// `vector` with the cross-axis component removed.
    pub fn project(self, vector: Vector) -> Vector {
        match self {
            Self::Free => vector,
            Self::Horizontal => Vector::new(vector.x, 0.0),
            Self::Vertical => Vector::new(0.0, vector.y),
        }
    }
}
