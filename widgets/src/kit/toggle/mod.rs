mod kind;
mod metrics;
mod options;

pub use kind::ToggleKind;
pub use options::ToggleOptions;

use reactive::Signal;
use view::Element;

use crate::util::spring;

/// A pill switch bound to `on`: the track cross-fades from the palette's off gradient to the accent
/// while the knob glides across, all on the snappy spring. Click, Space and Enter flip it.
pub fn toggle(on: Signal<bool>) -> Element<ToggleKind> {
    Element::new(ToggleKind { on, caption: None }).transition(spring::snappy())
}
