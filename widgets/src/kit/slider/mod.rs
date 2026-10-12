mod input;
mod kind;
mod metrics;
mod options;
mod scale;
mod thumb;

pub use kind::SliderKind;
pub use options::SliderOptions;

use reactive::Signal;
use view::Element;

use crate::util::spring;
use scale::SliderScale;

/// A horizontal slider bound to `value`: an accent-filled track and a raised thumb that glide
/// together on the snappy spring. Drag, click, arrows (Shift for ten times the step), Page keys,
/// Home, End and the wheel all move it; shape it with [`SliderOptions`].
pub fn slider(value: Signal<f32>) -> Element<SliderKind> {
    Element::new(SliderKind {
        value,
        scale: SliderScale::DEFAULT,
    })
    .transition(spring::snappy())
}
