use geometry::Vector;

/// Where in its lifetime a gesture is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum GesturePhase {
    /// The fingers started the gesture.
    Begin,
    /// The gesture moved.
    Update,
    /// The fingers lifted normally.
    End,
    /// The gesture was interrupted.
    Cancel,
}

/// Multi-finger touchpad gestures, which the DOM does not have.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TouchpadGesture {
    /// Two or more fingers pinching or rotating.
    Pinch {
        /// Lifecycle.
        phase: GesturePhase,
        /// Scale relative to the start of the gesture.
        scale: f32,
        /// Rotation in degrees since the previous event.
        rotation: f32,
        /// Movement of the fingers' center since the previous event.
        delta: Vector,
    },
    /// Three or more fingers swiping.
    Swipe {
        /// Lifecycle.
        phase: GesturePhase,
        /// Number of fingers.
        fingers: u32,
        /// Movement since the previous event.
        delta: Vector,
    },
    /// Fingers resting without moving.
    Hold {
        /// Lifecycle.
        phase: GesturePhase,
        /// Number of fingers.
        fingers: u32,
    },
}
