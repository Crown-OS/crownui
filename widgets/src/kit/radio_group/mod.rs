mod kind;
mod metrics;
mod navigation;

pub use kind::RadioGroupKind;

use reactive::Signal;
use view::{Element, Semantic};

use crate::util::spring;

/// A vertical group of radio rows bound to the index in `selected` (`None` while nothing is
/// chosen). The whole row is the target; the ring thickens on the crisp spring. Arrows move the
/// choice, wrapping at the ends, while the group has focus.
pub fn radio_group<Label: Into<String>>(
    options: impl IntoIterator<Item = Label>,
    selected: Signal<Option<usize>>,
) -> Element<RadioGroupKind> {
    Element::new(RadioGroupKind {
        options: options.into_iter().map(Into::into).collect(),
        selected,
    })
    .focusable(true)
    .transition(spring::crisp())
}
