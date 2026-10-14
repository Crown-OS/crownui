use std::time::Duration;

use geometry::{Point, Vector};
use input::{PointerEvent, PointerId, PointerType, WheelDelta, WheelEvent};
use sctk::{
    reexports::client::{Connection, QueueHandle, protocol::wl_pointer::WlPointer},
    seat::pointer::{
        AxisScroll, PointerEvent as SctkPointerEvent, PointerEventKind, PointerHandler,
    },
};

use super::keymap::{pointer_button_from_evdev, sctk_cursor};
use crate::{
    WindowEvent,
    wayland::{event_loop::WaylandPlatform, window::surface_window_id},
};

const STEPS_PER_LINE: f32 = 120.0;

fn wheel_delta(horizontal: &AxisScroll, vertical: &AxisScroll) -> WheelDelta {
    if horizontal.value120 != 0 || vertical.value120 != 0 {
        WheelDelta::Lines(Vector::new(
            horizontal.value120 as f32 / STEPS_PER_LINE,
            vertical.value120 as f32 / STEPS_PER_LINE,
        ))
    } else if horizontal.discrete != 0 || vertical.discrete != 0 {
        WheelDelta::Lines(Vector::new(
            horizontal.discrete as f32,
            vertical.discrete as f32,
        ))
    } else {
        WheelDelta::Pixels(Vector::new(
            horizontal.absolute as f32,
            vertical.absolute as f32,
        ))
    }
}

fn event_time(kind: &PointerEventKind) -> Duration {
    let millis = match kind {
        PointerEventKind::Motion { time }
        | PointerEventKind::Press { time, .. }
        | PointerEventKind::Release { time, .. }
        | PointerEventKind::Axis { time, .. } => *time,
        PointerEventKind::Enter { .. } | PointerEventKind::Leave { .. } => 0,
    };
    Duration::from_millis(u64::from(millis))
}

impl WaylandPlatform {
    pub(crate) fn apply_cursor_changes(&mut self) {
        for (window_id, window) in &mut self.windows {
            let Some(icon) = window.take_cursor_change() else {
                continue;
            };
            self.seats
                .iter()
                .filter(|devices| devices.pointer_focus == Some(*window_id))
                .filter_map(|devices| devices.pointer.as_ref())
                .for_each(|pointer| {
                    let _ = pointer.set_cursor(&self.connection, sctk_cursor(icon));
                });
        }
    }

    fn pointer_event(&self, event: &SctkPointerEvent) -> PointerEvent {
        PointerEvent {
            pointer_id: PointerId::MOUSE,
            pointer_type: PointerType::Mouse,
            position: Point::new(event.position.0 as f32, event.position.1 as f32),
            button: match event.kind {
                PointerEventKind::Press { button, .. }
                | PointerEventKind::Release { button, .. } => {
                    Some(pointer_button_from_evdev(button))
                }
                _ => None,
            },
            modifiers: self.modifiers,
            timestamp: event_time(&event.kind),
        }
    }
}

impl PointerHandler for WaylandPlatform {
    fn pointer_frame(
        &mut self,
        connection: &Connection,
        _: &QueueHandle<Self>,
        pointer: &WlPointer,
        events: &[SctkPointerEvent],
    ) {
        let Some(seat_index) = self.seats.iter().position(|devices| {
            devices
                .pointer
                .as_ref()
                .is_some_and(|themed| themed.pointer() == pointer)
        }) else {
            return;
        };

        for event in events {
            let window_id = surface_window_id(&event.surface);
            let pointer_event = self.pointer_event(event);
            let modifiers = self.modifiers;
            let Some(window) = self.windows.get_mut(&window_id) else {
                continue;
            };
            let devices = &mut self.seats[seat_index];
            let window_event = match event.kind {
                PointerEventKind::Enter { .. } => {
                    devices.pointer_focus = Some(window_id);
                    if let Some(themed) = &devices.pointer {
                        let _ = themed.set_cursor(connection, sctk_cursor(window.cursor()));
                    }
                    WindowEvent::PointerEnter(pointer_event)
                }
                PointerEventKind::Leave { .. } => {
                    devices.pointer_focus = None;
                    WindowEvent::PointerLeave(pointer_event)
                }
                PointerEventKind::Motion { .. } => WindowEvent::PointerMove(pointer_event),
                PointerEventKind::Press { serial, .. } => {
                    self.input_serial = serial;
                    window.record_button_press(devices.seat.clone(), serial);
                    WindowEvent::PointerDown(pointer_event)
                }
                PointerEventKind::Release { .. } => WindowEvent::PointerUp(pointer_event),
                PointerEventKind::Axis {
                    ref horizontal,
                    ref vertical,
                    ..
                } => WindowEvent::Wheel(WheelEvent {
                    delta: wheel_delta(horizontal, vertical),
                    position: pointer_event.position,
                    modifiers,
                }),
            };
            self.push_window_event(window_id, window_event);
        }
    }
}
