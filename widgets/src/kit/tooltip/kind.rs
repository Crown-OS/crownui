use view::{Cx, Kind, Listener, NodeId, Prop, StyleProp, Styled, View, bind, component, show};

use super::{bubble::tooltip_bubble, hover::HoverReveal, metrics::*};
use crate::{
    stack::{hstack, vstack},
    util::{anchored_popup::instant, shared_prop::shared_prop},
};

/// The kind behind [`tooltip`](super::tooltip).
pub struct TooltipKind<V> {
    pub(super) label: Prop<String>,
    pub(super) content: V,
    pub(super) start_visible: bool,
}

impl<V: View> Kind for TooltipKind<V> {
    const NAME: &'static str = "kit_tooltip";

    fn mount(self, cx: &mut Cx, node: NodeId) {
        let label = shared_prop(cx, self.label);
        let reveal = HoverReveal::new(cx, node, self.start_visible);
        cx.set_default_style(node, StyleProp::Position(layouting::Position::Relative));
        let anchor = cx.build(self.content);
        let size = cx.observe_size(node);
        let frame = reveal.frame;
        let overlay = cx.build(
            hstack(show(
                reveal.visible,
                move || {
                    component(move |cx| {
                        let above = frame.with_untracked(cx, |frame| frame.flipped);
                        tooltip_bubble(label, frame, above)
                    })
                },
                || vstack(()),
            ))
            .absolute()
            .left(bind(move |runtime| -frame.get(runtime).left))
            .w(bind(move |runtime| frame.get(runtime).window_width))
            .top(bind(move |runtime| {
                if frame.get(runtime).flipped {
                    -(BUBBLE_HEIGHT + GAP)
                } else {
                    size.get(runtime).height + GAP
                }
            }))
            .items_start()
            .pointer_events_none()
            .transition(instant())
            .top_layer(),
        );
        cx.adopt_children(node, vec![anchor, overlay]);
        let listeners = [
            Listener::PointerEnter(Box::new(move |cx, _| reveal.arm(cx))),
            Listener::PointerLeave(Box::new(move |cx, _| reveal.disarm(cx))),
            Listener::PointerDown(Box::new(move |cx, _| reveal.disarm(cx))),
        ];
        for listener in listeners {
            cx.add_listener(node, listener);
        }
    }
}
