mod focus_trap;
mod kind;
mod metrics;
mod options;
mod panel;
mod sheet;

pub use kind::DialogKind;
pub use options::DialogOptions;
pub use panel::DialogPanelKind;
pub use sheet::DialogSheetKind;

use reactive::Signal;
use view::{Element, ViewSeq};

/// A modal sheet inside the window: while `open` is true, `content()` sits in a centered card
/// over a scrim that covers the whole window. Focus moves into the card and Tab cycles inside
/// it; Escape or pressing the scrim sets `open` to false.
pub fn dialog<F, C>(open: Signal<bool>, content: F) -> Element<DialogKind<F>>
where
    F: FnMut() -> C + 'static,
    C: ViewSeq,
{
    Element::new(DialogKind {
        open,
        content,
        dismiss_on_scrim: true,
    })
}
