use geometry::Color;
use reactive::Signal;
use view::{Cx, Element, Kind, Listener, NodeId, StyleProp, Styled, ViewSeq};

use super::{metrics::*, panel::dialog_panel};
use crate::{
    kit::{Theme, ThemeMode, use_theme},
    stack::vstack,
    util::{spring, themed::themed},
};

fn scrim_color(theme: &Theme) -> Color {
    let alpha = match theme.mode {
        ThemeMode::Dark => DARK_SCRIM_ALPHA,
        ThemeMode::Light => LIGHT_SCRIM_ALPHA,
    };
    Color {
        a: alpha,
        ..theme.surface.shadow
    }
}

/// The kind behind the window-covering layer of an open [`dialog`](super::dialog): the scrim
/// and the centered card.
pub struct DialogSheetKind<C> {
    content: C,
    open: Signal<bool>,
    dismiss_on_scrim: bool,
}

impl<C: ViewSeq> Kind for DialogSheetKind<C> {
    const NAME: &'static str = "kit_dialog_sheet";

    fn mount(self, cx: &mut Cx, node: NodeId) {
        let theme = use_theme(cx);
        let Self {
            content,
            open,
            dismiss_on_scrim,
        } = self;
        let scrim = cx.build(
            vstack(())
                .absolute()
                .inset(0.0)
                .bg(themed(theme, scrim_color))
                .enter(|style| style.opacity(0.0))
                .transition(spring::crisp()),
        );
        cx.add_listener(
            scrim,
            Listener::PointerDown(Box::new(move |cx, event| {
                event.prevent_default();
                if dismiss_on_scrim {
                    open.set(cx, false);
                }
            })),
        );
        let panel = cx.build(dialog_panel(content, open));
        cx.set_default_style(node, StyleProp::Position(layouting::Position::Absolute));
        cx.adopt_children(node, vec![scrim, panel]);
    }
}

pub(super) fn dialog_sheet<C: ViewSeq>(
    content: C,
    open: Signal<bool>,
    dismiss_on_scrim: bool,
) -> Element<DialogSheetKind<C>> {
    Element::new(DialogSheetKind {
        content,
        open,
        dismiss_on_scrim,
    })
    .flex_col()
    .items_center()
    .justify_center()
    .exit(|style| style.opacity(0.0))
    .transition(spring::crisp())
}
