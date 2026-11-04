use reactive::Signal;
use view::{Element, Styled};

use crate::kit::{SearchKind, SearchOptions, TextInputOptions, search};

const SEARCH_WIDTH: f32 = 320.0;
const FOCUSED_SEARCH_WIDTH: f32 = 480.0;

/// The kit [`search`] bar sized for [`sidebar_with_header`](super::sidebar_with_header)'s
/// toolbar: wide enough for a query, springing wider while it is open, and giving way when the
/// content side is narrower. Lead it with a `title_bar_spacer()` and follow it with
/// `window_controls()`. Every [`SearchOptions`] and [`TextInputOptions`] still chains onto it.
pub fn sidebar_search(query: Signal<String>) -> Element<SearchKind> {
    search(query)
        .stretch()
        .widens_on_focus(SEARCH_WIDTH, FOCUSED_SEARCH_WIDTH)
        .shrink(1.0)
}
