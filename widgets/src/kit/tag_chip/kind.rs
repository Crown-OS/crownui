use geometry::Color;
use semantics::Role;
use view::{Cx, Kind, NodeId, Prop, StyleProp, Styled, bind};

use crate::{
    kit::{TEXT_SIZE, use_theme},
    stack::vstack,
    text::text,
    util::{shared_prop::shared_prop, themed::bind_themed_style, themed::themed},
};

pub(super) const HEIGHT: f32 = 22.0;
pub(super) const H_PADDING: f32 = 8.0;
pub(super) const DOT_GAP: f32 = 6.0;
pub(super) const CHIP_TEXT_SIZE: f32 = TEXT_SIZE - 3.0;
pub(super) const BORDER_WIDTH: f32 = 1.0;
const DOT_SIZE: f32 = 6.0;

/// The kind behind [`tag_chip`](super::tag_chip).
pub struct TagChipKind {
    pub(super) label: Prop<String>,
    pub(super) dot: Prop<Color>,
}

impl Kind for TagChipKind {
    const NAME: &'static str = "kit_tag_chip";

    fn mount(self, cx: &mut Cx, node: NodeId) {
        let theme = use_theme(cx);
        let dot_color = shared_prop(cx, self.dot);
        cx.set_role(node, Role::Group);
        bind_themed_style(cx, node, theme, |theme| {
            StyleProp::Background(Some(theme.popover.trigger_bg))
        });
        bind_themed_style(cx, node, theme, |theme| {
            StyleProp::BorderColor(theme.popover.border)
        });
        let dot = vstack(())
            .size(DOT_SIZE)
            .rounded(DOT_SIZE / 2.0)
            .shrink(0.0)
            .bg(bind(move |runtime| dot_color.get(runtime)));
        let label = text(self.label).text_color(themed(theme, |theme| theme.text.body));
        cx.set_children(node, (dot, label));
    }
}
