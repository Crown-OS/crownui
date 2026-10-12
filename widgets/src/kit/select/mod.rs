mod handles;
mod keyboard;
mod kind;
mod metrics;
mod options;
mod panel;
mod row;
mod skin;
mod type_ahead;
mod value_label;

pub use kind::SelectKind;
pub use options::SelectOptions;
pub use panel::SelectPanelKind;
pub use row::SelectRowKind;

use reactive::Signal;
use view::Element;

/// A dropdown over `options` bound two-way to `selected`, the index of the chosen option.
/// Picking fires DOM `change` with the option's text. Arrow keys, Home and End move the highlight
/// of the open panel, Enter picks, Escape closes, and typing jumps to the option that starts with it.
pub fn select(
    options: Signal<Vec<String>>,
    selected: Signal<Option<usize>>,
) -> Element<SelectKind> {
    kind::select_element(options, selected)
}
