use std::time::Duration;

use geometry::Point;
use input::{PointerButton, PointerEvent, PointerId};

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Press {
    pub pointer: PointerId,
    pub button: PointerButton,
    pub origin: Point,
    pub started: Duration,
}

impl Press {
    pub fn new(event: &PointerEvent, button: PointerButton) -> Self {
        Self {
            pointer: event.pointer_id,
            button,
            origin: event.position,
            started: event.timestamp,
        }
    }

    pub fn follows(&self, event: &PointerEvent) -> bool {
        self.pointer == event.pointer_id
    }

    pub fn is_released_by(&self, event: &PointerEvent) -> bool {
        self.follows(event) && event.button.is_none_or(|button| button == self.button)
    }

    pub fn exceeds_slop(&self, position: Point, slop: f32) -> bool {
        (position - self.origin).length() > slop
    }

    pub fn held_for(&self, now: Duration) -> Duration {
        now.saturating_sub(self.started)
    }
}
