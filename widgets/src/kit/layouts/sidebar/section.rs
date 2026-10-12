use view::{Cx, Element, IntoProp, Kind, NodeId, Prop, Styled, ViewSeq};

use crate::{
    kit::{INTER_FAMILY, TEXT_SIZE, use_theme},
    stack::vstack,
    text::text,
    util::themed::themed,
};

const LABEL_INDENT: f32 = 10.0;
const LABEL_TEXT_SIZE: f32 = TEXT_SIZE - 2.0;
const LABEL_HEIGHT: f32 = 26.0;
const LABEL_GAP: f32 = 2.0;
const ITEM_GAP: f32 = 4.0;

/// The kind behind [`sidebar_section`].
pub struct SidebarSectionKind<Items> {
    label: Prop<String>,
    items: Items,
}

impl<Items: ViewSeq> Kind for SidebarSectionKind<Items> {
    const NAME: &'static str = "kit_sidebar_section";

    fn mount(self, cx: &mut Cx, node: NodeId) {
        let theme = use_theme(cx);
        cx.set_default_style(node, view::StyleProp::Gap(Some(LABEL_GAP), Some(LABEL_GAP)));
        cx.set_default_style(
            node,
            view::StyleProp::FlexDirection(layouting::FlexDirection::Column),
        );
        let caption = vstack(
            text(self.label)
                .text_size(LABEL_TEXT_SIZE)
                .font_medium()
                .font_family(INTER_FAMILY)
                .text_color(themed(theme, |theme| theme.sidebar.subitem_text))
                .truncate(),
        )
        .h(LABEL_HEIGHT)
        .px(LABEL_INDENT)
        .shrink(0.0);
        let rows = vstack(self.items).gap(ITEM_GAP);
        let children = vec![cx.build(caption), cx.build(rows)];
        cx.adopt_children(node, children);
    }
}

/// A muted caption over a block of rows; the caption is not a row, so it never hovers or selects.
pub fn sidebar_section<Items: ViewSeq>(
    label: impl IntoProp<String>,
    items: Items,
) -> Element<SidebarSectionKind<Items>> {
    Element::new(SidebarSectionKind {
        label: label.into_prop(),
        items,
    })
}
