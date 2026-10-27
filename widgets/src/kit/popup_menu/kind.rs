use geometry::Size;
use view::{Cx, Element, Kind, NodeId, Styled, bind, component, show};

use super::{
    entry::{MenuEntry, RowSpec},
    handles::MenuHandles,
    metrics::*,
    panel::{panel_height, popup_menu_panel},
    trigger::popup_menu_trigger,
};
use crate::{
    spacer::spacer,
    stack::{hstack, vstack},
    util::anchored_popup::AnchoredPopup,
};

/// The kind behind [`popup_menu`](super::popup_menu).
pub struct PopupMenuKind {
    icon: &'static str,
    label: String,
    entries: Vec<MenuEntry>,
    pub(super) start_open: bool,
}

impl Kind for PopupMenuKind {
    const NAME: &'static str = "kit_popup_menu";

    fn mount(self, cx: &mut Cx, node: NodeId) {
        let disabled = cx.observe_disabled(node);
        let popup = AnchoredPopup::new(cx, node, Size::new(TRIGGER_WIDTH, TRIGGER_HEIGHT));
        let (rows, callbacks): (Vec<RowSpec>, Vec<_>) = self
            .entries
            .into_iter()
            .map(|entry| (entry.spec, entry.on_select))
            .unzip();
        let handles = MenuHandles {
            popup,
            highlighted: cx.signal(None),
            callbacks: cx.store(callbacks),
        };

        if self.start_open {
            popup.reveal(cx, panel_height(&rows));
        }
        let trigger = cx.build(popup_menu_trigger(
            self.icon,
            self.label,
            rows.clone(),
            handles,
            disabled,
        ));
        let overlay = cx.build(
            show(
                popup.open_signal(),
                move || {
                    let rows = rows.clone();
                    component(move |cx| {
                        hstack((
                            spacer(),
                            popup
                                .appear(cx, popup_menu_panel(rows, handles))
                                .shrink(0.0),
                        ))
                    })
                },
                || vstack(()),
            )
            .absolute()
            .left(bind(move |runtime| popup.trailing_lane(runtime).0))
            .w(bind(move |runtime| popup.trailing_lane(runtime).1))
            .top(bind(move |runtime| popup.top(runtime)))
            .top_layer(),
        );
        cx.adopt_children(node, vec![trigger, overlay]);
    }
}

pub(super) fn popup_menu_element(
    icon: &'static str,
    label: String,
    entries: Vec<MenuEntry>,
) -> Element<PopupMenuKind> {
    Element::new(PopupMenuKind {
        icon,
        label,
        entries,
        start_open: false,
    })
    .relative()
    .self_start()
    .w(TRIGGER_WIDTH)
    .h(TRIGGER_HEIGHT)
    .shrink(0.0)
}
