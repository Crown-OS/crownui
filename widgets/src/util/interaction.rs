use reactive::{Runtime, Signal};
use view::{Cx, Listener, NodeId};

/// Hover, press and focus as signals, so colors bound to them can also follow the theme
/// (state variants only take literal colors).
#[derive(Clone, Copy)]
pub(crate) struct Interaction {
    hovered: Signal<bool>,
    pressed: Signal<bool>,
    focused: Signal<bool>,
}

impl Interaction {
    pub(crate) fn track(cx: &mut Cx, node: NodeId) -> Self {
        let hovered = cx.signal(false);
        let pressed = cx.signal(false);
        let focused = cx.signal(false);
        let listeners = [
            Listener::PointerEnter(Box::new(move |cx, _| hovered.set(cx, true))),
            Listener::PointerLeave(Box::new(move |cx, _| {
                hovered.set(cx, false);
                pressed.set(cx, false);
            })),
            Listener::PointerDown(Box::new(move |cx, _| pressed.set(cx, true))),
            Listener::PointerUp(Box::new(move |cx, _| pressed.set(cx, false))),
            Listener::PointerCancel(Box::new(move |cx, _| pressed.set(cx, false))),
            Listener::Focus(Box::new(move |cx, _| focused.set(cx, true))),
            Listener::Blur(Box::new(move |cx, _| focused.set(cx, false))),
        ];
        for listener in listeners {
            cx.add_listener(node, listener);
        }
        Self {
            hovered,
            pressed,
            focused,
        }
    }

    pub(crate) fn is_hovered(self, runtime: &mut Runtime) -> bool {
        self.hovered.get(runtime)
    }

    pub(crate) fn is_pressed(self, runtime: &mut Runtime) -> bool {
        self.pressed.get(runtime)
    }

    pub(crate) fn is_focused(self, runtime: &mut Runtime) -> bool {
        self.focused.get(runtime)
    }
}
