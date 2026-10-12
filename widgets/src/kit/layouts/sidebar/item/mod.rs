mod kind;
mod layers;
mod options;
mod variant;

pub use kind::SidebarItemKind;
pub use options::SidebarItemOptions;
pub use variant::SidebarItemVariant;

use reactive::Signal;
use view::{Element, IntoProp};

fn row(
    variant: SidebarItemVariant,
    label: impl IntoProp<String>,
    icon: Option<&'static str>,
    selected: impl IntoProp<bool>,
) -> Element<SidebarItemKind> {
    Element::new(SidebarItemKind {
        variant,
        label: label.into_prop(),
        icon,
        trailing_icon: None,
        selected: selected.into_prop(),
    })
}

/// A leaf sidebar row with a leading icon; `selected` raises it into the pill.
pub fn sidebar_item(
    label: impl IntoProp<String>,
    icon: &'static str,
    selected: impl IntoProp<bool>,
) -> Element<SidebarItemKind> {
    row(SidebarItemVariant::Item, label, Some(icon), selected)
}

/// A nested sidebar row: no icon, indented under a group.
pub fn sidebar_subitem(
    label: impl IntoProp<String>,
    selected: impl IntoProp<bool>,
) -> Element<SidebarItemKind> {
    row(SidebarItemVariant::Subitem, label, None, selected)
}

/// The top-of-sidebar brand row: a bigger icon and label, never selected.
pub fn sidebar_brand(label: impl IntoProp<String>, icon: &'static str) -> Element<SidebarItemKind> {
    row(SidebarItemVariant::Brand, label, Some(icon), false)
}

pub(super) fn group_header(
    label: impl IntoProp<String>,
    icon: &'static str,
    trailing_icon: &'static str,
    open: Signal<bool>,
) -> Element<SidebarItemKind> {
    row(SidebarItemVariant::GroupHeader, label, Some(icon), open).trailing_icon(trailing_icon)
}
