use reactive::{Memo, Signal};
use semantics::Role;
use view::{Cx, Edges, Element, Kind, NodeId, Semantic, StyleProp, Styled, bind};

use super::{hover::AnchorFrame, metrics::*};
use crate::{
    kit::{INTER_FAMILY, use_theme},
    text::text,
    util::{glass::glass_surface, spring, themed::bind_themed_style},
};

/// The kind behind the small label a [`tooltip`](super::tooltip) shows.
pub struct TooltipBubbleKind {
    label: Memo<String>,
    frame: Signal<AnchorFrame>,
}

impl Kind for TooltipBubbleKind {
    const NAME: &'static str = "kit_tooltip_bubble";

    fn mount(self, cx: &mut Cx, node: NodeId) {
        let theme = use_theme(cx);
        let label = self.label;
        let frame = self.frame;
        cx.set_role(node, Role::Tooltip);
        let size = cx.observe_size(node);
        cx.bind_style(node, move |runtime| {
            let left = frame.get(runtime).label_left(size.get(runtime).width);
            StyleProp::Margin(Edges::Left, left)
        });
        let plate = glass_surface(cx, node, theme, RADIUS);
        bind_themed_style(cx, node, theme, |theme| {
            StyleProp::TextColor(theme.menu.text)
        });
        cx.set_children(node, (plate, text(bind(move |runtime| label.get(runtime)))));
    }
}

pub(super) fn tooltip_bubble(
    label: Memo<String>,
    frame: Signal<AnchorFrame>,
    flipped: bool,
) -> Element<TooltipBubbleKind> {
    let slide = if flipped { 4.0 } else { -4.0 };
    Element::new(TooltipBubbleKind { label, frame })
        .label(bind(move |runtime| label.get(runtime)))
        .shrink(0.0)
        .max_w(MAX_WIDTH)
        .px(H_PADDING)
        .py(V_PADDING)
        .border(BORDER_WIDTH)
        .rounded(RADIUS)
        .text_size(TEXT_SIZE)
        .leading(LINE_HEIGHT)
        .font_family(INTER_FAMILY)
        .pointer_events_none()
        .origin(0.5, if flipped { 1.0 } else { 0.0 })
        .enter(|style| style.opacity(0.0).scale(APPEAR_SCALE).translate_y(slide))
        .exit(|style| style.opacity(0.0).scale(APPEAR_SCALE))
        .transition(spring::crisp())
}
