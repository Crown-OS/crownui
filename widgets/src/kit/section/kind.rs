use semantics::Role;
use view::{Cx, Kind, NodeId, Semantic, Styled, ViewSeq};

use crate::{
    kit::{TEXT_SIZE, card, use_theme},
    text::text,
    util::themed::themed,
};

pub(super) const HEADING_GAP: f32 = 8.0;
const HEADING_TEXT_SIZE: f32 = TEXT_SIZE - 2.0;
const HEADING_INSET: f32 = 4.0;

/// The kind behind [`section`](super::section).
pub struct SectionKind<C> {
    pub(super) title: String,
    pub(super) children: C,
}

impl<C: ViewSeq> Kind for SectionKind<C> {
    const NAME: &'static str = "kit_section";

    fn mount(self, cx: &mut Cx, node: NodeId) {
        let theme = use_theme(cx);
        let heading = text(self.title)
            .role(Role::Heading)
            .text_size(HEADING_TEXT_SIZE)
            .font_semibold()
            .px(HEADING_INSET)
            .text_color(themed(theme, |theme| theme.text.muted));
        let rows = card(self.children).self_stretch().overflow_hidden();
        cx.set_children(node, (heading, rows));
    }
}
