mod kind;

pub use kind::SectionKind;

use view::{Element, Semantic, Styled, ViewSeq};

use crate::{kit::INTER_FAMILY, util::theme_fade::theme_fade};

use kind::HEADING_GAP;

/// A titled group of rows on one card surface, like a settings page section.
pub fn section<C: ViewSeq>(title: impl Into<String>, children: C) -> Element<SectionKind<C>> {
    let title = title.into();
    Element::new(SectionKind {
        title: title.clone(),
        children,
    })
    .flex_col()
    .gap(HEADING_GAP)
    .font_family(INTER_FAMILY)
    .label(title)
    .transition(theme_fade())
}
