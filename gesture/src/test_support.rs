use std::time::Duration;

use geometry::Point;
use input::{PointerButton, PointerEvent};

use crate::{GestureOutput, Recognizer};

pub(crate) fn at(x: f32, y: f32, millis: u64) -> PointerEvent {
    PointerEvent {
        timestamp: Duration::from_millis(millis),
        ..PointerEvent::mouse(Point::new(x, y))
    }
}

pub(crate) fn down(x: f32, y: f32, millis: u64) -> PointerEvent {
    pressed(x, y, millis, PointerButton::Primary)
}

pub(crate) fn pressed(x: f32, y: f32, millis: u64, button: PointerButton) -> PointerEvent {
    PointerEvent {
        button: Some(button),
        ..at(x, y, millis)
    }
}

pub(crate) fn drain(recognizer: &mut impl Recognizer) -> Vec<GestureOutput> {
    let mut outputs = Vec::new();
    recognizer.drain_outputs(&mut outputs);
    outputs
}
