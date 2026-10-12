use view::{Cx, Kind, NodeId, Prop, Styled};

use crate::{
    kit::{TEXT_SIZE, Tone, use_theme},
    stack::vstack,
    text::text,
    util::themed::themed,
};

pub(super) const DOT_SIZE: f32 = 7.0;
pub(super) const DOT_GAP: f32 = 6.0;
pub(super) const STATUS_TEXT_SIZE: f32 = TEXT_SIZE - 2.0;

/// The kind behind [`status`](super::status).
pub struct StatusKind {
    pub(super) label: Prop<String>,
    pub(super) tone: Tone,
}

impl Kind for StatusKind {
    const NAME: &'static str = "kit_status";

    fn mount(self, cx: &mut Cx, node: NodeId) {
        let theme = use_theme(cx);
        let tone = self.tone;
        let dot = vstack(())
            .size(DOT_SIZE)
            .rounded(DOT_SIZE / 2.0)
            .shrink(0.0)
            .bg(themed(theme, move |theme| tone.color(theme)));
        let label = text(self.label).text_color(themed(theme, move |theme| tone.text_color(theme)));
        cx.set_children(node, (dot, label));
    }
}
