use view::{Element, Styled, ViewSeq};

use crate::stack::{StackKind, vstack};

const ITEM_GAP: f32 = 4.0;

/// Sidebar rows stacked with the sidebar's own row gap.
pub fn sidebar_column<Items: ViewSeq>(items: Items) -> Element<StackKind<Items>> {
    vstack(items).gap(ITEM_GAP)
}
