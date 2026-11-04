mod kind;

pub use kind::TagChipKind;

use geometry::Color;
use view::{Element, IntoProp, Styled};

use crate::{kit::INTER_FAMILY, util::theme_fade::theme_fade};

use kind::{BORDER_WIDTH, CHIP_TEXT_SIZE, DOT_GAP, H_PADDING, HEIGHT};

/// A 22px pill naming a tag: a 6px dot in the tag's `dot` color before `label`, on the field
/// background with a hairline border.
pub fn tag_chip(label: impl IntoProp<String>, dot: impl IntoProp<Color>) -> Element<TagChipKind> {
    Element::new(TagChipKind {
        label: label.into_prop(),
        dot: dot.into_prop(),
    })
    .flex_row()
    .items_center()
    .self_start()
    .shrink(0.0)
    .h(HEIGHT)
    .px(H_PADDING)
    .gap(DOT_GAP)
    .rounded(HEIGHT / 2.0)
    .border(BORDER_WIDTH)
    .text_size(CHIP_TEXT_SIZE)
    .font_family(INTER_FAMILY)
    .transition(theme_fade())
}
