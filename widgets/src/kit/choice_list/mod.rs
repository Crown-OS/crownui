mod choice;
mod kind;
mod metrics;
mod navigation;
mod row;

pub use choice::Choice;
pub use kind::ChoiceListKind;

use reactive::Signal;
use view::{Element, Semantic};

use crate::util::spring;

/// A stack of selectable surfaces, each with an optional icon tile, a title, a line of detail and
/// a trailing radio, bound to the index in `selected` (`None` while nothing is chosen). It takes
/// the width its parent offers. Hover washes on the snappy spring, selection on the crisp one;
/// arrows move the choice between pickable rows.
pub fn choice_list(
    choices: impl IntoIterator<Item = Choice>,
    selected: Signal<Option<usize>>,
) -> Element<ChoiceListKind> {
    Element::new(ChoiceListKind {
        choices: choices.into_iter().collect(),
        selected,
    })
    .focusable(true)
    .transition(spring::crisp())
}
