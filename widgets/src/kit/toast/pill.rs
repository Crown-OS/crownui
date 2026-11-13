use semantics::Role;
use view::{Cx, Element, Kind, NodeId, Semantic, StyleProp, Styled};

use super::metrics::*;
use crate::{
    kit::{INTER_FAMILY, icon::tinted_icon, use_theme},
    text::text,
    util::{
        glass::glass_surface,
        themed::{bind_themed_style, themed},
    },
};

/// The kind behind [`toast`](super::toast).
pub struct ToastKind {
    message: String,
    pub(super) icon: Option<&'static str>,
}

impl Kind for ToastKind {
    const NAME: &'static str = "kit_toast";

    fn mount(self, cx: &mut Cx, node: NodeId) {
        let theme = use_theme(cx);
        cx.set_role(node, Role::Group);
        let plate = glass_surface(cx, node, theme, RADIUS);
        bind_themed_style(cx, node, theme, |theme| {
            StyleProp::TextColor(theme.menu.text)
        });
        let glyph = self.icon.map(|body| {
            tinted_icon(body, ICON_STROKE, themed(theme, |theme| theme.menu.text))
                .size(ICON_SIZE)
                .shrink(0.0)
        });
        let label = text(self.message);
        cx.set_children(node, (plate, glyph, label));
    }
}

/// A notification pill: `message` (and an optional leading icon) on a glass surface. A
/// [`toast_host`](super::toast_host) shows these for [`Toasts::show`](super::Toasts::show).
pub fn toast(message: impl Into<String>) -> Element<ToastKind> {
    let message = message.into();
    Element::new(ToastKind {
        message: message.clone(),
        icon: None,
    })
    .label(message)
    .flex_row()
    .items_center()
    .gap(ICON_GAP)
    .shrink(0.0)
    .h(HEIGHT)
    .px(H_PADDING)
    .border(BORDER_WIDTH)
    .rounded(RADIUS)
    .text_size(TEXT_SIZE)
    .font_family(INTER_FAMILY)
    .font_medium()
}
