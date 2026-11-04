use geometry::Vector;
use input::{Key, NamedKey};
use reactive::Signal;
use semantics::Role;
use view::{Cx, Element, Kind, Listener, NodeId, Semantic, Shadow, StyleProp, Styled, ViewSeq};

use super::{focus_trap::FocusTrap, metrics::*};
use crate::{
    kit::{INTER_FAMILY, TEXT_SIZE, use_theme},
    util::{spring, themed::bind_themed_style},
};

/// The kind behind the card a [`dialog`](super::dialog) shows.
pub struct DialogPanelKind<C> {
    content: C,
    open: Signal<bool>,
}

impl<C: ViewSeq> Kind for DialogPanelKind<C> {
    const NAME: &'static str = "kit_dialog_panel";

    fn mount(self, cx: &mut Cx, node: NodeId) {
        let theme = use_theme(cx);
        let open = self.open;
        cx.set_role(node, Role::Dialog);
        bind_themed_style(cx, node, theme, |theme| {
            StyleProp::Background(Some(theme.surface.raised))
        });
        bind_themed_style(cx, node, theme, |theme| {
            StyleProp::BorderColor(theme.surface.border)
        });
        bind_themed_style(cx, node, theme, |theme| {
            StyleProp::TextColor(theme.text.body)
        });
        bind_themed_style(cx, node, theme, |theme| {
            let offset = Vector::new(0.0, SHADOW_OFFSET_Y);
            StyleProp::Shadow(Some(Shadow::new(theme.surface.shadow, SHADOW_BLUR, offset)))
        });
        cx.set_children(node, self.content);
        let trap = FocusTrap::install(cx, node);
        cx.add_listener(
            node,
            Listener::KeyDown(Box::new(move |cx, event| {
                let modifiers = event.modifiers;
                match event.key {
                    Key::Named(NamedKey::Escape) => open.set(cx, false),
                    Key::Named(NamedKey::Tab)
                        if !(modifiers.ctrl_key || modifiers.alt_key || modifiers.meta_key) =>
                    {
                        trap.cycle(cx, modifiers.shift_key);
                    }
                    _ => return,
                }
                event.prevent_default();
                event.stop_propagation();
            })),
        );
    }
}

pub(super) fn dialog_panel<C: ViewSeq>(
    content: C,
    open: Signal<bool>,
) -> Element<DialogPanelKind<C>> {
    Element::new(DialogPanelKind { content, open })
        .focusable(true)
        .flex_col()
        .gap(GAP)
        .p(PADDING)
        .min_w(MIN_WIDTH)
        .max_w(MAX_WIDTH)
        .shrink(0.0)
        .rounded(RADIUS)
        .border(BORDER_WIDTH)
        .text_size(TEXT_SIZE)
        .font_family(INTER_FAMILY)
        .enter(|style| style.opacity(0.0).scale(APPEAR_SCALE))
        .transition(spring::snappy())
}
