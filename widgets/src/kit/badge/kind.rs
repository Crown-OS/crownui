use geometry::Color;
use view::{Cx, Kind, NodeId, Prop, StyleProp, Styled, bind};

use crate::{
    kit::{TEXT_SIZE, icon::tinted_icon},
    text::text,
    util::{color::scale_alpha, shared_prop::shared_prop},
};

pub(super) const RADIUS: f32 = 6.0;
pub(super) const H_PADDING: f32 = 8.0;
pub(super) const V_PADDING: f32 = 3.0;
pub(super) const BADGE_TEXT_SIZE: f32 = TEXT_SIZE - 2.0;
const ICON_SIZE: f32 = 12.0;
const ICON_STROKE: f32 = 1.8;
const ICON_GAP: f32 = 4.0;
const BG_ALPHA: f32 = 0.14;

/// The kind behind [`badge`](super::badge).
pub struct BadgeKind {
    pub(super) label: Prop<String>,
    pub(super) color: Prop<Color>,
    pub(super) icon: Option<&'static str>,
}

impl Kind for BadgeKind {
    const NAME: &'static str = "kit_badge";

    fn mount(self, cx: &mut Cx, node: NodeId) {
        let color = shared_prop(cx, self.color);
        cx.bind_style(node, move |runtime| {
            StyleProp::Background(Some(scale_alpha(color.get(runtime), BG_ALPHA)))
        });
        let glyph = self.icon.map(|body| {
            tinted_icon(body, ICON_STROKE, bind(move |runtime| color.get(runtime))).size(ICON_SIZE)
        });
        let label = text(self.label)
            .text_color(bind(move |runtime| color.get(runtime)))
            .ml(if glyph.is_some() { ICON_GAP } else { 0.0 });
        cx.set_children(node, (glyph, label));
    }
}
