use view::{Cx, Element, Kind, Listener, NodeId, StyleProp};

/// The kind behind [`title_bar_spacer`].
pub struct TitleBarSpacerKind;

impl Kind for TitleBarSpacerKind {
    const NAME: &'static str = "kit_title_bar_spacer";

    fn mount(self, cx: &mut Cx, node: NodeId) {
        for prop in [
            StyleProp::Grow(1.0),
            StyleProp::AlignSelf(layouting::Align::Stretch),
        ] {
            cx.set_default_style(node, prop);
        }
        cx.add_listener(
            node,
            Listener::PointerDown(Box::new(|cx, event| {
                if event.button == Some(input::PointerButton::Primary) {
                    cx.begin_window_move();
                }
            })),
        );
        cx.add_listener(
            node,
            Listener::DoubleClick(Box::new(|cx, _| cx.toggle_maximize_window())),
        );
    }
}

/// Empty space in a client-drawn title bar that grows like `spacer()`: dragging it moves the
/// window and double-clicking it maximizes or restores the window.
pub fn title_bar_spacer() -> Element<TitleBarSpacerKind> {
    Element::new(TitleBarSpacerKind)
}
