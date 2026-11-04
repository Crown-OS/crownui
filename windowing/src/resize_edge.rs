use crate::CursorIcon;

/// The edge or corner of a window an interactive resize drags.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(missing_docs)]
pub enum ResizeEdge {
    Top,
    Bottom,
    Left,
    Right,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl ResizeEdge {
    /// The cursor shown while the pointer rests on this edge.
    pub const fn cursor(self) -> CursorIcon {
        match self {
            Self::Top | Self::Bottom => CursorIcon::NsResize,
            Self::Left | Self::Right => CursorIcon::EwResize,
            Self::TopLeft | Self::BottomRight => CursorIcon::NwseResize,
            Self::TopRight | Self::BottomLeft => CursorIcon::NeswResize,
        }
    }
}
