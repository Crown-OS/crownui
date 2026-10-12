mod kind;
mod options;

pub use kind::BadgeKind;
pub use options::BadgeOptions;

use geometry::Color;
use view::{Element, IntoProp, Styled};

use crate::{kit::INTER_FAMILY, util::theme_fade::theme_fade};

use kind::{BADGE_TEXT_SIZE, H_PADDING, RADIUS, V_PADDING};

/// A small pill naming what something is: `label` and an optional leading icon, tinted from one `color`.
pub fn badge(label: impl IntoProp<String>, color: impl IntoProp<Color>) -> Element<BadgeKind> {
    Element::new(BadgeKind {
        label: label.into_prop(),
        color: color.into_prop(),
        icon: None,
    })
    .flex_row()
    .items_center()
    .self_start()
    .shrink(0.0)
    .px(H_PADDING)
    .py(V_PADDING)
    .rounded(RADIUS)
    .text_size(BADGE_TEXT_SIZE)
    .font_family(INTER_FAMILY)
    .transition(theme_fade())
}
