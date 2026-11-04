use geometry::{Point, Size};
use reactive::{Runtime, Signal};
use view::{Cx, Element, Kind, NodeId, Styled, bind};

/// The window's frame as seen from a host node, so an overlay mounted anywhere can cover the
/// whole window: the host's window origin is measured once it is laid out, the window size
/// follows the root.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct WindowCover {
    origin: Signal<Point>,
    size: Signal<Size>,
}

impl WindowCover {
    pub(crate) fn track(cx: &mut Cx, host: NodeId) -> Self {
        let cover = Self {
            origin: cx.signal(Point::ZERO),
            size: cx.signal(Size::ZERO),
        };
        cover.measure(cx, host);
        cover
    }

    fn measure(self, cx: &mut Cx, host: NodeId) {
        let tree = cx.tree();
        let root = tree
            .root()
            .and_then(|root| Some((root, tree.get(root)?.geometry().rect.size)));
        let origin = tree
            .get(host)
            .map(|host| host.geometry().window_rect.origin);
        let (Some((root, window)), Some(origin)) = (root, origin) else {
            cx.request_animation_frame(move |cx, _| self.measure(cx, host));
            return;
        };
        if window.is_empty() {
            cx.request_animation_frame(move |cx, _| self.measure(cx, host));
            return;
        }
        self.origin.set(cx, origin);
        self.size.set(cx, window);
        let root_scope = cx.root_scope();
        let caller = cx.set_current_scope(root_scope);
        let observed = cx.observe_size(root);
        cx.set_current_scope(caller);
        let size = self.size;
        cx.effect(move |runtime| {
            let window = observed.get(runtime);
            if !window.is_empty() && size.with_untracked(runtime, |current| *current != window) {
                size.set(runtime, window);
            }
        });
    }

    /// The window size, zero until it is known.
    pub(crate) fn size(self, runtime: &mut Runtime) -> Size {
        self.size.get(runtime)
    }

    /// Positions `overlay` (a child of the host) over the whole window, above everything else.
    pub(crate) fn fill<K: Kind>(self, overlay: Element<K>) -> Element<K> {
        let Self { origin, size } = self;
        overlay
            .absolute()
            .left(bind(move |runtime| -origin.get(runtime).x))
            .top(bind(move |runtime| -origin.get(runtime).y))
            .w(bind(move |runtime| size.get(runtime).width))
            .h(bind(move |runtime| size.get(runtime).height))
            .top_layer()
    }
}
