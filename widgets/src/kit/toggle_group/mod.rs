mod item;
mod kind;
mod metrics;
mod options;
mod segment;
mod selection;

pub use item::ToggleItem;
pub use kind::ToggleGroupKind;
pub use options::ToggleGroupOptions;

use reactive::Signal;
use view::{Element, Semantic};

/// A single-select row of equal segments bound to the index in `selected`; a raised pill springs
/// to the chosen segment. Arrows, Home and End move the choice while the group has focus.
pub fn toggle_group(
    items: impl IntoIterator<Item = ToggleItem>,
    selected: Signal<usize>,
) -> Element<ToggleGroupKind> {
    Element::new(ToggleGroupKind {
        items: items.into_iter().collect(),
        selected,
        stretch: false,
    })
    .focusable(true)
}
