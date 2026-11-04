use geometry::Point;
use windowing::{ResizeEdge, dpi::LogicalSize};

/// How far in from a client-decorated window's edge a press resizes it.
const BAND_WIDTH: f32 = 6.0;
/// How far along an edge from a corner a press in the band grabs the corner.
const CORNER_REACH: f32 = 16.0;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Side {
    Start,
    End,
}

fn side(offset: f32, length: f32, reach: f32) -> Option<Side> {
    if offset < reach {
        Some(Side::Start)
    } else if offset >= length - reach {
        Some(Side::End)
    } else {
        None
    }
}

/// The edge a press at `position` resizes a window of `size` from, if it lands in the band.
pub(crate) fn edge_at(position: Point, size: LogicalSize) -> Option<ResizeEdge> {
    let (width, height) = (size.width as f32, size.height as f32);
    let horizontal = side(position.x, width, BAND_WIDTH);
    let vertical = side(position.y, height, BAND_WIDTH);
    let horizontal =
        horizontal.or(side(position.x, width, CORNER_REACH).filter(|_| vertical.is_some()));
    let vertical =
        vertical.or(side(position.y, height, CORNER_REACH).filter(|_| horizontal.is_some()));
    Some(match (horizontal, vertical) {
        (None, None) => return None,
        (Some(Side::Start), None) => ResizeEdge::Left,
        (Some(Side::End), None) => ResizeEdge::Right,
        (None, Some(Side::Start)) => ResizeEdge::Top,
        (None, Some(Side::End)) => ResizeEdge::Bottom,
        (Some(Side::Start), Some(Side::Start)) => ResizeEdge::TopLeft,
        (Some(Side::End), Some(Side::Start)) => ResizeEdge::TopRight,
        (Some(Side::Start), Some(Side::End)) => ResizeEdge::BottomLeft,
        (Some(Side::End), Some(Side::End)) => ResizeEdge::BottomRight,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const SIZE: LogicalSize = LogicalSize::new(800, 600);

    #[test]
    fn the_middle_of_the_window_resizes_nothing() {
        assert_eq!(edge_at(Point::new(400.0, 300.0), SIZE), None);
    }

    #[test]
    fn each_edge_grabs_its_own_side() {
        assert_eq!(
            edge_at(Point::new(2.0, 300.0), SIZE),
            Some(ResizeEdge::Left)
        );
        assert_eq!(
            edge_at(Point::new(797.0, 300.0), SIZE),
            Some(ResizeEdge::Right)
        );
        assert_eq!(edge_at(Point::new(400.0, 1.0), SIZE), Some(ResizeEdge::Top));
        assert_eq!(
            edge_at(Point::new(400.0, 598.0), SIZE),
            Some(ResizeEdge::Bottom)
        );
    }

    #[test]
    fn a_corner_reaches_along_both_edges() {
        assert_eq!(
            edge_at(Point::new(12.0, 2.0), SIZE),
            Some(ResizeEdge::TopLeft)
        );
        assert_eq!(
            edge_at(Point::new(797.0, 590.0), SIZE),
            Some(ResizeEdge::BottomRight)
        );
        assert_eq!(edge_at(Point::new(12.0, 12.0), SIZE), None);
    }
}
