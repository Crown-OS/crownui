use view::{Element, Styled, View};

use crate::{
    spacer::{SpacerKind, spacer},
    stack::{StackKind, hstack},
};

const BUTTON_GAP: f32 = 10.0;

/// The button bar: `secondary` on the left, `primary` on the right, spare width between them.
pub fn wizard_footer<Secondary: View, Primary: View>(
    secondary: Secondary,
    primary: Primary,
) -> Element<StackKind<(Secondary, Element<SpacerKind>, Primary)>> {
    hstack((secondary, spacer(), primary))
        .gap(BUTTON_GAP)
        .items_center()
}
