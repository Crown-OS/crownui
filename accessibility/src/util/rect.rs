use geometry::Rect;

pub(crate) fn to_accesskit_rect(rect: Rect) -> accesskit::Rect {
    accesskit::Rect::new(
        f64::from(rect.min_x()),
        f64::from(rect.min_y()),
        f64::from(rect.max_x()),
        f64::from(rect.max_y()),
    )
}
