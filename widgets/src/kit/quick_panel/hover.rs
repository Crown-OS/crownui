use reactive::Signal;
use view::{Cx, Listener, NodeId};

/// Whether the pointer is over `node`, for a hover wash that follows the theme.
pub(super) fn track_hover(cx: &mut Cx, node: NodeId) -> Signal<bool> {
    let hovered = cx.signal(false);
    cx.add_listener(
        node,
        Listener::PointerEnter(Box::new(move |cx, _| hovered.set(cx, true))),
    );
    cx.add_listener(
        node,
        Listener::PointerLeave(Box::new(move |cx, _| hovered.set(cx, false))),
    );
    hovered
}
