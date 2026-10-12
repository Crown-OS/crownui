use std::time::Duration;

use geometry::Point;

use crate::Modifiers;

/// Identifies one pointer (the mouse, one finger, one pen) for its whole contact.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PointerId(pub u64);

impl PointerId {
    /// The mouse, which every platform reports as a single pointer.
    pub const MOUSE: Self = Self(0);
}

/// The device behind a pointer, like the DOM's `pointerType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PointerType {
    /// Mouse or touchpad cursor.
    Mouse,
    /// A finger on a touchscreen.
    Touch,
    /// A stylus.
    Pen,
}

/// Which button changed, like the DOM's `button`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PointerButton {
    /// Usually the left button, or a touch contact.
    Primary,
    /// Usually the right button.
    Secondary,
    /// Usually the wheel button.
    Auxiliary,
    /// The "back" side button.
    Back,
    /// The "forward" side button.
    Forward,
    /// Any other button, by platform code.
    Other(u32),
}

/// A pointer down, up, move, enter, leave or cancel, like the DOM's `PointerEvent`.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PointerEvent {
    /// Which pointer.
    pub pointer_id: PointerId,
    /// What kind of device.
    pub pointer_type: PointerType,
    /// Position in logical pixels relative to the window.
    pub position: Point,
    /// The button that changed; `None` for moves, enters and leaves.
    pub button: Option<PointerButton>,
    /// Modifiers held at the time.
    pub modifiers: Modifiers,
    /// Monotonic event time.
    pub timestamp: Duration,
}

impl PointerEvent {
    /// A mouse event at `position` with nothing else set.
    pub fn mouse(position: Point) -> Self {
        Self {
            pointer_id: PointerId::MOUSE,
            pointer_type: PointerType::Mouse,
            position,
            button: None,
            modifiers: Modifiers::NONE,
            timestamp: Duration::ZERO,
        }
    }
}
