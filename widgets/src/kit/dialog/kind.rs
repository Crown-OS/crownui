use reactive::Signal;
use view::{Cx, Kind, NodeId, StyleProp, ViewSeq, component, show};

use super::sheet::dialog_sheet;
use crate::{stack::vstack, util::window_cover::WindowCover};

/// The kind behind [`dialog`](super::dialog).
pub struct DialogKind<F> {
    pub(super) open: Signal<bool>,
    pub(super) content: F,
    pub(super) dismiss_on_scrim: bool,
}

impl<F, C> Kind for DialogKind<F>
where
    F: FnMut() -> C + 'static,
    C: ViewSeq,
{
    const NAME: &'static str = "kit_dialog";

    fn mount(self, cx: &mut Cx, node: NodeId) {
        let Self {
            open,
            mut content,
            dismiss_on_scrim,
        } = self;
        cx.set_default_style(node, StyleProp::Position(layouting::Position::Absolute));
        cx.set_children(
            node,
            show(
                open,
                move || {
                    let body = content();
                    component(move |cx| {
                        let cover = WindowCover::track(cx, node);
                        cover.fill(dialog_sheet(body, open, dismiss_on_scrim))
                    })
                },
                || vstack(()),
            ),
        );
    }
}
