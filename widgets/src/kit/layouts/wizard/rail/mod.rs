mod kind;
mod stage;
mod step;

pub use kind::WizardRailKind;

use view::{Element, IntoProp};

/// The step indicator: one dot per step on a connected track, completed steps checked and filled,
/// the current one ringed. `current` is a zero-based index; past the end every step reads complete.
pub fn wizard_rail(
    steps: impl IntoIterator<Item = impl Into<String>>,
    current: impl IntoProp<usize>,
) -> Element<WizardRailKind> {
    Element::new(WizardRailKind {
        labels: steps.into_iter().map(Into::into).collect(),
        current: current.into_prop(),
    })
}
