mod kind;
mod options;

pub use kind::StatusKind;
pub use options::StatusOptions;

use view::{Element, IntoProp, Styled};

use crate::{
    kit::{INTER_FAMILY, Tone},
    util::theme_fade::theme_fade,
};

use kind::{DOT_GAP, STATUS_TEXT_SIZE};

/// A colored dot followed by `label`, quiet unless given a [`Tone`] with [`StatusOptions::tone`].
pub fn status(label: impl IntoProp<String>) -> Element<StatusKind> {
    Element::new(StatusKind {
        label: label.into_prop(),
        tone: Tone::default(),
    })
    .flex_row()
    .items_center()
    .self_start()
    .shrink(0.0)
    .gap(DOT_GAP)
    .text_size(STATUS_TEXT_SIZE)
    .font_family(INTER_FAMILY)
    .transition(theme_fade())
}
