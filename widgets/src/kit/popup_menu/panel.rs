use semantics::Role;
use view::{Cx, Element, Kind, Listener, NodeId, StyleProp, Styled};

use super::{
    entry::RowSpec, handles::MenuHandles, metrics::*, row::pickable_row, separator::separator_row,
};
use crate::{
    kit::{INTER_FAMILY, TEXT_SIZE, use_theme},
    util::themed::bind_themed_style,
};

pub(super) fn panel_height(rows: &[RowSpec]) -> f32 {
    PANEL_PADDING * 2.0
        + rows
            .iter()
            .map(|row| {
                if row.separator {
                    SEPARATOR_HEIGHT
                } else {
                    ROW_HEIGHT
                }
            })
            .sum::<f32>()
}

/// The kind behind the floating panel of a popup menu.
pub struct PopupMenuPanelKind {
    rows: Vec<RowSpec>,
    handles: MenuHandles,
}

impl Kind for PopupMenuPanelKind {
    const NAME: &'static str = "kit_popup_menu_panel";

    fn mount(self, cx: &mut Cx, node: NodeId) {
        let theme = use_theme(cx);
        let handles = self.handles;
        cx.set_role(node, Role::Menu);
        bind_themed_style(cx, node, theme, |theme| {
            StyleProp::Background(Some(theme.menu.bg))
        });
        bind_themed_style(cx, node, theme, |theme| {
            StyleProp::BorderColor(theme.menu.border)
        });
        cx.add_listener(
            node,
            Listener::PointerDown(Box::new(|_, event| event.prevent_default())),
        );
        cx.add_listener(
            node,
            Listener::Click(Box::new(|_, event| event.stop_propagation())),
        );
        cx.add_listener(
            node,
            Listener::PointerLeave(Box::new(move |cx, _| handles.highlighted.set(cx, None))),
        );
        let rows: Vec<NodeId> = self
            .rows
            .into_iter()
            .enumerate()
            .map(|(index, spec)| {
                if spec.separator {
                    cx.build(separator_row())
                } else {
                    cx.build(pickable_row(spec, index, handles))
                }
            })
            .collect();
        cx.adopt_children(node, rows);
    }
}

pub(super) fn popup_menu_panel(
    rows: Vec<RowSpec>,
    handles: MenuHandles,
) -> Element<PopupMenuPanelKind> {
    Element::new(PopupMenuPanelKind { rows, handles })
        .flex_col()
        .items_stretch()
        .p(PANEL_PADDING)
        .min_w(PANEL_MIN_WIDTH)
        .max_w(PANEL_MAX_WIDTH)
        .rounded(PANEL_RADIUS)
        .border(1.0)
        .text_size(TEXT_SIZE)
        .font_family(INTER_FAMILY)
}
