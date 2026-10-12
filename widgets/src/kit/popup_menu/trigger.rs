use reactive::Signal;
use semantics::Role;
use view::{Cx, Element, Kind, Listener, NodeId, Semantic, Styled, bind};
use windowing::CursorIcon;

use super::{
    entry::RowSpec, handles::MenuHandles, keyboard::handle_key, metrics::*, panel::panel_height,
};
use crate::{
    kit::{icon::tinted_icon, use_theme},
    stack::{vstack, zstack},
    util::{interaction::Interaction, spring, themed::themed},
};

/// The kind behind the small icon pill that opens a popup menu.
pub struct PopupMenuTriggerKind {
    icon: &'static str,
    rows: Vec<RowSpec>,
    handles: MenuHandles,
    disabled: Signal<bool>,
}

impl Kind for PopupMenuTriggerKind {
    const NAME: &'static str = "kit_popup_menu_trigger";

    fn mount(self, cx: &mut Cx, node: NodeId) {
        let theme = use_theme(cx);
        let interaction = Interaction::track(cx, node);
        let Self {
            icon,
            rows,
            handles,
            disabled,
        } = self;
        let height = panel_height(&rows);
        cx.set_role(node, Role::Button);

        cx.add_listener(
            node,
            Listener::Click(Box::new(move |cx, _| {
                if !disabled.get(cx) {
                    handles.popup.toggle(cx, height);
                }
            })),
        );
        cx.add_listener(
            node,
            Listener::Blur(Box::new(move |cx, _| handles.close(cx))),
        );
        cx.add_listener(
            node,
            Listener::KeyDown(Box::new(move |cx, event| {
                if !disabled.get(cx) {
                    handle_key(cx, event, handles, &rows);
                }
            })),
        );

        let wash = move |runtime: &mut reactive::Runtime| {
            let engaged = handles.popup.is_open(runtime) || interaction.is_hovered(runtime);
            if engaged && !disabled.get(runtime) {
                1.0
            } else {
                0.0
            }
        };
        let hover_wash = vstack(())
            .absolute()
            .inset(0.0)
            .rounded(TRIGGER_RADIUS)
            .bg(themed(theme, |theme| theme.surface.hover))
            .opacity(bind(wash));
        let resting_glyph = tinted_icon(
            icon,
            ICON_STROKE,
            bind(move |runtime| {
                let dimmed = disabled.get(runtime);
                theme.with(runtime, |theme| {
                    if dimmed {
                        theme.text.disabled
                    } else {
                        theme.text.icon_muted
                    }
                })
            }),
        )
        .size(TRIGGER_ICON);
        let engaged_glyph = tinted_icon(icon, ICON_STROKE, themed(theme, |theme| theme.text.icon))
            .size(TRIGGER_ICON)
            .opacity(bind(wash));
        let glyph = zstack((resting_glyph, engaged_glyph))
            .w(TRIGGER_ICON)
            .h(TRIGGER_ICON);
        cx.set_children(node, (hover_wash, glyph));
    }
}

pub(super) fn popup_menu_trigger(
    icon: &'static str,
    label: String,
    rows: Vec<RowSpec>,
    handles: MenuHandles,
    disabled: Signal<bool>,
) -> Element<PopupMenuTriggerKind> {
    Element::new(PopupMenuTriggerKind {
        icon,
        rows,
        handles,
        disabled,
    })
    .label(label)
    .disabled(disabled)
    .relative()
    .flex_row()
    .items_center()
    .justify_center()
    .w(TRIGGER_WIDTH)
    .h(TRIGGER_HEIGHT)
    .shrink(0.0)
    .cursor(CursorIcon::Pointer)
    .transition(spring::snappy())
}
