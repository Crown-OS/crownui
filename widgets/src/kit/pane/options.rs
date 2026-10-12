use geometry::Color;
use view::Element;

use super::{fill::PaneFill, kind::PaneKind, radii::Radii};
use crate::kit::{GradientStops, Theme};

/// The options of a [`pane`](super::pane), chained before its modifiers.
pub trait PaneOptions: Sized {
    /// Fills with one flat color.
    fn fill(self, color: Color) -> Self;

    /// Fills with a two-stop gradient projected top to bottom.
    fn gradient(self, stops: GradientStops) -> Self;

    /// Fills with the theme's accent gradient, following the theme as it changes.
    fn accent_gradient(self) -> Self {
        self.themed_fill(|theme| PaneFill::Stops(theme.accent))
    }

    /// Fills with whatever `fill` resolves to in the current theme.
    fn themed_fill(self, fill: impl Fn(&Theme) -> PaneFill + 'static) -> Self;

    /// Per-corner radii; see [`Radii::all`], [`Radii::left`] and [`Radii::right`].
    fn radii(self, radii: Radii) -> Self;

    /// A `width` border drawn inside the pane's edge.
    fn bordered(self, color: Color, width: f32) -> Self {
        self.bordered_with(width, move |_| color)
    }

    /// A `width` border colored by `color` in the current theme.
    fn bordered_with(self, width: f32, color: impl Fn(&Theme) -> Color + 'static) -> Self;
}

impl<C: view::ViewSeq> PaneOptions for Element<PaneKind<C>> {
    fn fill(self, color: Color) -> Self {
        self.themed_fill(move |_| PaneFill::Color(color))
    }

    fn gradient(self, stops: GradientStops) -> Self {
        self.themed_fill(move |_| PaneFill::Stops(stops))
    }

    fn themed_fill(self, fill: impl Fn(&Theme) -> PaneFill + 'static) -> Self {
        self.configure(|kind| kind.fill = Box::new(fill))
    }

    fn radii(self, radii: Radii) -> Self {
        self.configure(|kind| kind.radii = radii)
    }

    fn bordered_with(self, width: f32, color: impl Fn(&Theme) -> Color + 'static) -> Self {
        self.configure(|kind| kind.border = Some((Box::new(color), width)))
    }
}
