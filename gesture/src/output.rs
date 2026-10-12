use geometry::{Point, Vector};

/// A recognized gesture, named after the DOM events it corresponds to.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GestureOutput {
    /// A press and release within the slop: `click`, `dblclick` or a triple click.
    Click {
        /// Where the pointer was released.
        position: Point,
        /// 1 for a click, 2 for a double click, 3 for a triple click.
        count: u8,
    },
    /// A press held still long enough.
    LongPress {
        /// Where the pointer was when the press became long.
        position: Point,
    },
    /// A drag began.
    DragStart {
        /// Where the pointer went down.
        position: Point,
    },
    /// A drag moved.
    Drag {
        /// Current pointer position.
        position: Point,
        /// Movement since the previous `Drag`, along the recognizer's axis.
        delta: Vector,
        /// Movement since `DragStart`, along the recognizer's axis.
        total: Vector,
    },
    /// A drag finished with the pointer's release.
    DragEnd {
        /// Where the pointer was released.
        position: Point,
        /// Release velocity in logical pixels per second, along the recognizer's axis.
        velocity: Vector,
    },
    /// A touchpad pinch began.
    PinchStart,
    /// A touchpad pinch moved.
    Pinch {
        /// Scale relative to the start of the pinch.
        scale: f32,
        /// Rotation in degrees since the previous `Pinch`.
        rotation: f32,
        /// Movement of the fingers' center since the previous `Pinch`.
        center_delta: Vector,
    },
    /// A touchpad pinch finished.
    PinchEnd {
        /// Scale change per second at the end of the pinch.
        velocity_scale: f32,
    },
    /// A started drag or pinch was interrupted.
    Cancel,
}
