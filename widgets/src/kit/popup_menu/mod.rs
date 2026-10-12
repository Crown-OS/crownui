mod entry;
mod flag;
mod handles;
mod keyboard;
mod kind;
mod metrics;
mod options;
mod panel;
mod row;
mod separator;
mod tint;
mod trigger;

pub use entry::{MenuEntry, menu_item, menu_separator};
pub use flag::MenuFlag;
pub use kind::PopupMenuKind;
pub use options::PopupMenuOptions;
pub use panel::PopupMenuPanelKind;
pub use row::PopupMenuRowKind;
pub use separator::PopupMenuSeparatorKind;
pub use trigger::PopupMenuTriggerKind;

use view::Element;

/// An icon pill that opens an anchored menu of `entries`; `label` names the pill for assistive
/// technology. Arrow keys, Home and End move the highlight, Enter picks it, Escape closes.
pub fn popup_menu(
    icon: &'static str,
    label: impl Into<String>,
    entries: impl IntoIterator<Item = MenuEntry>,
) -> Element<PopupMenuKind> {
    kind::popup_menu_element(icon, label.into(), entries.into_iter().collect())
}
