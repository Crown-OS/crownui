use view::{Element, ViewSeq};

use super::kind::DialogKind;

/// The options of a [`dialog`](super::dialog), chained before its modifiers.
pub trait DialogOptions: Sized {
    /// Whether pressing the scrim around the card closes the dialog; on by default.
    fn dismiss_on_scrim(self, dismiss: bool) -> Self;
}

impl<F, C> DialogOptions for Element<DialogKind<F>>
where
    F: FnMut() -> C + 'static,
    C: ViewSeq,
{
    fn dismiss_on_scrim(self, dismiss: bool) -> Self {
        self.configure(|kind| kind.dismiss_on_scrim = dismiss)
    }
}
