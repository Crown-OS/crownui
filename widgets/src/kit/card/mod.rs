mod kind;

pub use kind::CardKind;

use view::{Element, Styled, ViewSeq};

use crate::util::{fit_parent::FitParent, theme_fade::theme_fade};

use kind::{BORDER_WIDTH, RADIUS};

/// The kit's card chrome around `content`: the raised surface, a 1px border and rounded corners.
pub fn card<C: ViewSeq>(content: C) -> Element<CardKind<C>> {
    Element::new(CardKind { content })
        .flex_col()
        .self_start()
        .fit_parent_width()
        .p(BORDER_WIDTH)
        .border(BORDER_WIDTH)
        .rounded(RADIUS)
        .transition(theme_fade())
}
