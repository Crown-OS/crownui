use reactive::Signal;
use view::{Cx, Element};

use super::{
    kind::{FocusBasis, SearchKind},
    suggestion::Suggestion,
};
use crate::kit::{TextInputKind, TextInputOptions};

/// The options of a [`search`](super::search) bar, chained before its listeners and modifiers; the
/// field's own options come from [`TextInputOptions`].
pub trait SearchOptions: Sized {
    /// The rows the popup offers, in order. Filtering them against the query is the caller's
    /// business, so any matching rule works.
    fn suggestions(self, suggestions: Signal<Vec<Suggestion>>) -> Self;

    /// The most rows the popup shows; the default is eight.
    fn max_suggestions(self, max: usize) -> Self;

    /// Called with a suggestion's index when it is clicked, or highlighted and entered. Without
    /// it, picking a suggestion writes its label into the query.
    fn on_select(self, handler: impl FnMut(&mut Cx, usize) + 'static) -> Self;

    /// Rests at a flex basis of `resting` and springs to `focused` while the bar has focus.
    fn widens_on_focus(self, resting: f32, focused: f32) -> Self;
}

impl SearchOptions for Element<SearchKind> {
    fn suggestions(self, suggestions: Signal<Vec<Suggestion>>) -> Self {
        self.configure(|kind| kind.suggestions = Some(suggestions))
    }

    fn max_suggestions(self, max: usize) -> Self {
        self.configure(|kind| kind.max_suggestions = max)
    }

    fn on_select(self, handler: impl FnMut(&mut Cx, usize) + 'static) -> Self {
        self.configure(|kind| kind.on_select = Some(Box::new(handler)))
    }

    fn widens_on_focus(self, resting: f32, focused: f32) -> Self {
        self.configure(|kind| kind.focus_basis = Some(FocusBasis { resting, focused }))
    }
}

impl TextInputOptions for Element<SearchKind> {
    fn configure_input(self, change: impl FnOnce(&mut TextInputKind)) -> Self {
        self.configure(|kind| change(&mut kind.input))
    }
}
