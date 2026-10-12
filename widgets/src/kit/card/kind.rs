use view::{Cx, Kind, NodeId, StyleProp, ViewSeq};

use crate::{kit::use_theme, util::themed::bind_themed_style};

pub(super) const RADIUS: f32 = 14.0;
pub(super) const BORDER_WIDTH: f32 = 1.0;

/// The kind behind [`card`](super::card).
pub struct CardKind<C> {
    pub(super) content: C,
}

impl<C: ViewSeq> Kind for CardKind<C> {
    const NAME: &'static str = "kit_card";

    fn mount(self, cx: &mut Cx, node: NodeId) {
        let theme = use_theme(cx);
        bind_themed_style(cx, node, theme, |theme| {
            StyleProp::Background(Some(theme.surface.raised))
        });
        bind_themed_style(cx, node, theme, |theme| {
            StyleProp::BorderColor(theme.surface.border)
        });
        cx.set_children(node, self.content);
    }
}
