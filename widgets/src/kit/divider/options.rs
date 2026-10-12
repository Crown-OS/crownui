use geometry::Color;
use view::{Element, IntoProp};

use super::kind::DividerKind;

/// The options of a [`divider`](super::divider), chained before its modifiers.
pub trait DividerOptions: Sized {
    /// The line's color, replacing the theme's surface border.
    fn color(self, color: impl IntoProp<Color>) -> Self;

    /// The total vertical footprint; the 1px line stays centered in it.
    fn height(self, height: f32) -> Self;
}

impl DividerOptions for Element<DividerKind> {
    fn color(self, color: impl IntoProp<Color>) -> Self {
        let color = color.into_prop();
        self.configure(|kind| kind.color = Some(color))
    }

    fn height(self, height: f32) -> Self {
        self.configure(|kind| kind.height = height)
    }
}
