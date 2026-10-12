use input::{Key, NamedKey};
use layouting::Position;
use reactive::Signal;
use semantics::Role;
use view::{Cx, Edges, Element, Kind, Listener, NodeId, Semantic, StyleProp, Styled, View, show};

use crate::stack::vstack;

const POPOVER_Z_INDEX: i16 = 100;
const POPOVER_GAP: f32 = 4.0;

/// The kind behind [`popover`].
pub struct PopoverKind<A, F> {
    anchor: A,
    open: Signal<bool>,
    content: F,
}

impl<A, F, C> Kind for PopoverKind<A, F>
where
    A: View,
    F: FnMut() -> C + 'static,
    C: View,
{
    const NAME: &'static str = "popover";

    fn mount(self, cx: &mut Cx, node: NodeId) {
        let Self {
            anchor,
            open,
            mut content,
        } = self;
        cx.set_default_style(node, StyleProp::Position(Position::Relative));
        let anchor = cx.build(anchor);
        let overlay = cx.build(show(
            open,
            move || vstack(content()).z(POPOVER_Z_INDEX).role(Role::Dialog),
            || vstack(()),
        ));
        cx.set_default_style(overlay, StyleProp::Position(Position::Absolute));
        cx.set_default_style(overlay, StyleProp::Inset(Edges::Left, 0.0));
        cx.set_default_style(overlay, StyleProp::Inset(Edges::Top, 0.0));
        cx.set_default_style(overlay, StyleProp::ZIndex(POPOVER_Z_INDEX));
        cx.adopt_children(node, vec![anchor, overlay]);
        cx.add_listener(
            node,
            Listener::KeyDown(Box::new(move |cx, event| {
                if event.key == Key::Named(NamedKey::Escape) && open.get(cx) {
                    open.set(cx, false);
                    event.stop_propagation();
                }
            })),
        );
        let anchor_size = cx.observe_size(anchor);
        cx.bind_style(overlay, move |runtime| {
            StyleProp::Inset(Edges::Top, anchor_size.get(runtime).height + POPOVER_GAP)
        });
    }
}

/// Shows `content()` below `anchor` while `open` is true, layered above siblings. Escape closes
/// it. The anchor typically toggles `open` in its `on_click`.
pub fn popover<A, F, C>(anchor: A, open: Signal<bool>, content: F) -> Element<PopoverKind<A, F>>
where
    A: View,
    F: FnMut() -> C + 'static,
    C: View,
{
    Element::new(PopoverKind {
        anchor,
        open,
        content,
    })
}
