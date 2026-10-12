use geometry::{Rect, Size};
use layouting::NodeLayout;
use taffy::Layout;

pub(crate) fn node_layout(layout: &Layout) -> NodeLayout {
    let overflow = layout.scrollable_overflow_rect;
    NodeLayout {
        rect: Rect::from_xywh(
            layout.location.x,
            layout.location.y,
            layout.size.width,
            layout.size.height,
        ),
        content_size: Size::new(
            overflow.right.max(layout.size.width),
            overflow.bottom.max(layout.size.height),
        ),
    }
}
