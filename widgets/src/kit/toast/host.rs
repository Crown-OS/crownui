use view::{Cx, Kind, NodeId, StyleProp, Styled, View, for_each};

use super::{
    metrics::*,
    options::ToastOptions,
    pill::toast,
    toasts::{ShownToast, use_toasts},
};
use crate::{stack::vstack, util::spring};

/// The kind behind [`toast_host`](super::toast_host).
pub struct ToastHostKind<V> {
    pub(super) content: V,
}

fn key(toast: &ShownToast) -> u64 {
    toast.id
}

impl<V: View> Kind for ToastHostKind<V> {
    const NAME: &'static str = "kit_toast_host";

    fn mount(self, cx: &mut Cx, node: NodeId) {
        let toasts = use_toasts(cx);
        cx.set_default_style(node, StyleProp::Position(layouting::Position::Relative));
        let content = cx.build(self.content);
        cx.set_default_style(content, StyleProp::Grow(1.0));
        let stack = cx.build(
            for_each(toasts.shown, key, |cx, shown| {
                let (message, icon) =
                    shown.with_untracked(cx, |shown| (shown.message.clone(), shown.icon));
                let pill = toast(message);
                match icon {
                    Some(body) => pill.leading_icon(body),
                    None => pill,
                }
                .pointer_events_auto()
                .enter(|style| {
                    style
                        .opacity(0.0)
                        .scale(APPEAR_SCALE)
                        .translate_y(APPEAR_SLIDE)
                })
                .exit(|style| {
                    style
                        .opacity(0.0)
                        .scale(APPEAR_SCALE)
                        .translate_y(APPEAR_SLIDE)
                })
                .transition(spring::snappy())
            })
            .flex_col()
            .items_center()
            .gap(STACK_GAP),
        );
        let layer = cx.build(
            vstack(())
                .absolute()
                .inset(0.0)
                .flex_col()
                .items_center()
                .justify_end()
                .pb(BOTTOM_MARGIN)
                .pointer_events_none()
                .top_layer(),
        );
        cx.adopt_children(layer, vec![stack]);
        cx.adopt_children(node, vec![content, layer]);
    }
}
