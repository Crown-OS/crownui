use geometry::Rect;

/// Mirrors a parent-relative child rect horizontally inside a parent `parent_width` wide.
pub fn mirror_for_rtl(child: Rect, parent_width: f32) -> Rect {
    Rect::from_xywh(
        parent_width - child.max_x(),
        child.min_y(),
        child.size.width,
        child.size.height,
    )
}

/// Rounds a rect's edges to the nearest device pixel at `scale`, keeping shared edges shared.
pub fn snap_to_pixels(rect: Rect, scale: f32) -> Rect {
    let snap = |value: f32| (value * scale).round() / scale;
    let (min_x, min_y) = (snap(rect.min_x()), snap(rect.min_y()));
    Rect::from_xywh(
        min_x,
        min_y,
        snap(rect.max_x()) - min_x,
        snap(rect.max_y()) - min_y,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mirroring_swaps_leading_and_trailing_space() {
        let mirrored = mirror_for_rtl(Rect::from_xywh(10.0, 5.0, 30.0, 20.0), 100.0);
        assert_eq!(mirrored, Rect::from_xywh(60.0, 5.0, 30.0, 20.0));
    }

    #[test]
    fn neighbours_stay_adjacent_after_snapping() {
        let left = snap_to_pixels(Rect::from_xywh(0.0, 0.0, 10.3, 1.0), 1.25);
        let right = snap_to_pixels(Rect::from_xywh(10.3, 0.0, 10.3, 1.0), 1.25);
        assert_eq!(left.max_x(), right.min_x());
    }
}
