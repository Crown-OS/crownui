use view::{Element, Styled, ViewSeq};

use super::metrics::{CARD_HEIGHT, HERO_CARD_RADIUS, HERO_WIDTH};
use crate::{
    kit::{PaneKind, PaneOptions, Radii, pane},
    stack::{StackKind, vstack},
};

/// The card's left panel: the accent gradient, rounded down the outer edge and square at the seam, with `content` centred in it.
pub fn hero_pane<C: ViewSeq>(content: C) -> Element<PaneKind<Element<StackKind<C>>>> {
    pane(
        vstack(content)
            .w(HERO_WIDTH)
            .h(CARD_HEIGHT)
            .items_center()
            .justify_center(),
    )
    .accent_gradient()
    .radii(Radii::left(HERO_CARD_RADIUS))
    .shrink(0.0)
}
