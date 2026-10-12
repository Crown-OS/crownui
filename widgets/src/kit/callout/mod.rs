mod kind;
mod options;

pub use kind::CalloutKind;
pub use options::CalloutOptions;

use view::{Element, IntoProp, Styled};

use crate::{
    kit::{INTER_FAMILY, Tone},
    util::{fit_parent::FitParent, theme_fade::theme_fade},
};

use kind::{BORDER_WIDTH, ICON_GAP, PADDING, RADIUS};

/// A tinted notice block in `tone`, headed by `title`; add the wrapping explanation with [`CalloutOptions::body`].
pub fn callout(tone: Tone, title: impl IntoProp<String>) -> Element<CalloutKind> {
    Element::new(CalloutKind {
        tone,
        title: title.into_prop(),
        body: None,
        icon: None,
    })
    .flex_row()
    .items_start()
    .self_start()
    .fit_parent_width()
    .gap(ICON_GAP)
    .p(PADDING + BORDER_WIDTH)
    .border(BORDER_WIDTH)
    .rounded(RADIUS)
    .font_family(INTER_FAMILY)
    .transition(theme_fade())
}
