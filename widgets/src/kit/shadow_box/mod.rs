mod kind;

pub use kind::SoftShadowKind;

use view::{Element, Styled, ViewSeq};

use crate::util::{fit_parent::FitParent, theme_fade::theme_fade};

/// A soft drop shadow, rounded to `radius`, behind `content`; `radius` should match the content's own corners.
pub fn soft_shadow<C: ViewSeq>(content: C, radius: f32) -> Element<SoftShadowKind<C>> {
    Element::new(SoftShadowKind { content, radius })
        .flex_col()
        .self_start()
        .fit_parent_width()
        .transition(theme_fade())
}
