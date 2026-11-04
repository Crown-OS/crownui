use geometry::{Point, Size};

/// Where a menu opened at `point` goes: to the right of and below it, flipped to the left or
/// above where it would cross the window's edge, and kept inside the window when it fits
/// neither way.
pub(super) fn place_at(point: Point, panel: Size, window: Size) -> Point {
    let axis = |start: f32, extent: f32, limit: f32| {
        if limit <= 0.0 || start + extent <= limit {
            return start;
        }
        if start - extent >= 0.0 {
            return start - extent;
        }
        (limit - extent).max(0.0)
    };
    Point::new(
        axis(point.x, panel.width, window.width),
        axis(point.y, panel.height, window.height),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const PANEL: Size = Size::new(340.0, 200.0);
    const WINDOW: Size = Size::new(800.0, 600.0);

    #[test]
    fn a_menu_with_room_opens_right_of_and_below_the_pointer() {
        assert_eq!(
            place_at(Point::new(100.0, 80.0), PANEL, WINDOW),
            Point::new(100.0, 80.0)
        );
    }

    #[test]
    fn a_menu_near_the_right_and_bottom_edges_flips_left_and_up() {
        assert_eq!(
            place_at(Point::new(700.0, 500.0), PANEL, WINDOW),
            Point::new(360.0, 300.0)
        );
    }

    #[test]
    fn a_menu_that_fits_neither_way_is_pinned_inside_the_window() {
        let narrow = Size::new(400.0, 300.0);
        assert_eq!(
            place_at(Point::new(200.0, 150.0), PANEL, narrow),
            Point::new(60.0, 100.0)
        );
    }

    #[test]
    fn an_unknown_window_never_flips() {
        assert_eq!(
            place_at(Point::new(700.0, 500.0), PANEL, Size::ZERO),
            Point::new(700.0, 500.0)
        );
    }
}
