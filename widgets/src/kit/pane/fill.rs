use geometry::Color;
use graphics::Paint;

use crate::kit::GradientStops;

/// What a pane paints behind its content.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PaneFill {
    /// Nothing: the pane is only a per-corner clip and border.
    None,
    /// One flat color.
    Color(Color),
    /// A two-stop gradient projected top to bottom, the orientation of every accent surface.
    Stops(GradientStops),
}

impl PaneFill {
    pub(super) fn paint(self, height: f32) -> Option<Paint> {
        match self {
            Self::None => None,
            Self::Color(color) => Some(color.into()),
            Self::Stops(stops) => Some(stops.vertical(0.0, height)),
        }
    }
}

#[cfg(test)]
mod tests {
    use geometry::rgb;

    use super::*;

    #[test]
    fn nothing_has_no_paint_and_stops_span_the_height() {
        assert!(PaneFill::None.paint(40.0).is_none());
        let stops = GradientStops::new(rgb(0x0000ff), rgb(0x00ff00));
        assert_eq!(
            PaneFill::Stops(stops).paint(40.0),
            Some(stops.vertical(0.0, 40.0))
        );
    }
}
