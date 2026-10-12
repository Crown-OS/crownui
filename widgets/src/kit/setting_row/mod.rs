mod kind;

pub use kind::SettingRowKind;

use view::{Element, Semantic, Styled, View};

use crate::kit::INTER_FAMILY;

use kind::{GAP, PADDING_HORIZONTAL, PADDING_VERTICAL};

/// A settings row: `label` on the left, `control` on the right.
pub fn setting_row<C: View>(label: impl Into<String>, control: C) -> Element<SettingRowKind<C>> {
    let label = label.into();
    Element::new(SettingRowKind {
        label: label.clone(),
        control,
    })
    .flex_row()
    .items_center()
    .justify_between()
    .gap(GAP)
    .px(PADDING_HORIZONTAL)
    .py(PADDING_VERTICAL)
    .font_family(INTER_FAMILY)
    .label(label)
}
