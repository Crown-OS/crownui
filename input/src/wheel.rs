use geometry::{Point, Vector};

use crate::Modifiers;

/// Scroll amount; positive `y` scrolls content down, positive `x` right.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum WheelDelta {
    /// Precise scrolling in logical pixels (touchpads).
    Pixels(Vector),
    /// Notched wheels, in lines.
    Lines(Vector),
}

/// A wheel or touchpad scroll, like the DOM's `WheelEvent`.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WheelEvent {
    /// How far to scroll.
    pub delta: WheelDelta,
    /// Pointer position in logical pixels.
    pub position: Point,
    /// Modifiers held at the time.
    pub modifiers: Modifiers,
}

impl WheelDelta {
    /// The delta in logical pixels, given the height of one line.
    pub fn to_pixels(self, line_height: f32) -> Vector {
        match self {
            Self::Pixels(pixels) => pixels,
            Self::Lines(lines) => lines * line_height,
        }
    }
}
