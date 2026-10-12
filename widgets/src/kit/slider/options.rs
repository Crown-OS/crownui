use view::Element;

use super::kind::SliderKind;

/// The options of a [`slider`](super::slider), chained before its listeners and modifiers.
pub trait SliderOptions: Sized {
    /// The value range; the default is `0.0..=1.0`.
    fn range(self, min: f32, max: f32) -> Self;

    /// Snaps every pointer and keyboard value to multiples of `step` from the range's start.
    fn step(self, step: f32) -> Self;
}

impl SliderOptions for Element<SliderKind> {
    fn range(self, min: f32, max: f32) -> Self {
        self.configure(|kind| (kind.scale.min, kind.scale.max) = (min, max))
    }

    fn step(self, step: f32) -> Self {
        self.configure(|kind| kind.scale.step = Some(step))
    }
}
