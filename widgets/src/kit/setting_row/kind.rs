use view::{Cx, Kind, NodeId, Styled, View};

use crate::{
    kit::{TEXT_SIZE, use_theme},
    text::text,
    util::themed::themed,
};

pub(super) const PADDING_VERTICAL: f32 = 14.0;
pub(super) const PADDING_HORIZONTAL: f32 = 18.0;
pub(super) const GAP: f32 = 12.0;

/// The kind behind [`setting_row`](super::setting_row).
pub struct SettingRowKind<C> {
    pub(super) label: String,
    pub(super) control: C,
}

impl<C: View> Kind for SettingRowKind<C> {
    const NAME: &'static str = "kit_setting_row";

    fn mount(self, cx: &mut Cx, node: NodeId) {
        let theme = use_theme(cx);
        let label = text(self.label)
            .text_size(TEXT_SIZE)
            .grow(1.0)
            .text_color(themed(theme, |theme| theme.text.body));
        cx.set_children(node, (label, self.control));
    }
}
