use reactive::Signal;
use view::{Element, IntoProp, Listeners, Styled, View};

use super::{
    collapse::{SidebarCollapseKind, sidebar_collapse},
    item::{SidebarItemKind, group_header},
};
use crate::{
    kit::icons,
    stack::{StackKind, vstack},
};

const ITEM_GAP: f32 = 4.0;

type GroupContents<V> = (Element<SidebarItemKind>, Element<SidebarCollapseKind<V>>);

/// An expandable group: a header row that toggles `open` and a collapsing body of `children`.
pub fn sidebar_group<V: View>(
    label: impl IntoProp<String>,
    icon: &'static str,
    open: Signal<bool>,
    children: V,
) -> Element<StackKind<GroupContents<V>>> {
    let header = group_header(label, icon, icons::CHEVRON_DOWN, open)
        .on_click(move |cx, _| open.update(cx, |open| *open = !*open));
    vstack((header, sidebar_collapse(children, open))).gap(ITEM_GAP)
}
