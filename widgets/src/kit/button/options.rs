use view::Element;

use super::{
    kind::ButtonKind,
    skin::{ButtonSize, ButtonVariant},
};

/// The options of a [`button`](super::button), chained before its listeners and modifiers.
pub trait ButtonOptions: Sized {
    /// Which weight the button carries; the default is [`ButtonVariant::Secondary`].
    fn variant(self, variant: ButtonVariant) -> Self;

    /// The geometry preset; the default is [`ButtonSize::Medium`].
    fn control_size(self, size: ButtonSize) -> Self;

    /// A Lucide icon from [`icons`](crate::kit::icons) drawn before the label, in the label's color.
    fn leading_icon(self, body: &'static str) -> Self;

    /// A width floor; the button is never narrower than this.
    fn min_width(self, width: f32) -> Self;

    /// Takes the whole width its parent offers.
    fn stretch(self) -> Self {
        self.min_width(f32::INFINITY)
    }
}

impl ButtonOptions for Element<ButtonKind> {
    fn variant(self, variant: ButtonVariant) -> Self {
        self.configure(|kind| kind.variant = variant)
    }

    fn control_size(self, size: ButtonSize) -> Self {
        self.configure(|kind| kind.size = size)
    }

    fn leading_icon(self, body: &'static str) -> Self {
        self.configure(|kind| kind.icon = Some(body))
    }

    fn min_width(self, width: f32) -> Self {
        self.configure(|kind| kind.min_width = width)
    }
}
