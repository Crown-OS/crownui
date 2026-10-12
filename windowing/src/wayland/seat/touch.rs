use std::time::Duration;

use geometry::Point;
use input::{PointerButton, PointerEvent, PointerId, PointerType};
use sctk::{
    reexports::client::{
        Connection, QueueHandle,
        protocol::{wl_surface::WlSurface, wl_touch::WlTouch},
    },
    seat::touch::TouchHandler,
};

use crate::{
    WindowEvent, WindowId,
    wayland::{event_loop::WaylandPlatform, window::surface_window_id},
};

/// Touch ids are offset so they never collide with [`PointerId::MOUSE`].
const FIRST_TOUCH_POINTER: u64 = 1;

pub(crate) struct TouchPoint {
    id: i32,
    window_id: WindowId,
    position: Point,
    timestamp: Duration,
}

impl TouchPoint {
    fn pointer_event(&self, platform: &WaylandPlatform) -> PointerEvent {
        PointerEvent {
            pointer_id: PointerId(FIRST_TOUCH_POINTER + u64::from(self.id.unsigned_abs())),
            pointer_type: PointerType::Touch,
            position: self.position,
            button: Some(PointerButton::Primary),
            modifiers: platform.modifiers,
            timestamp: self.timestamp,
        }
    }
}

fn point((x, y): (f64, f64)) -> Point {
    Point::new(x as f32, y as f32)
}

impl TouchHandler for WaylandPlatform {
    fn down(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &WlTouch,
        _: u32,
        time: u32,
        surface: WlSurface,
        id: i32,
        position: (f64, f64),
    ) {
        let window_id = surface_window_id(&surface);
        if !self.windows.contains_key(&window_id) {
            return;
        }
        let touch = TouchPoint {
            id,
            window_id,
            position: point(position),
            timestamp: Duration::from_millis(u64::from(time)),
        };
        let event = touch.pointer_event(self);
        self.push_window_event(window_id, WindowEvent::PointerDown(event));
        self.touch_points.push(touch);
    }

    fn up(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &WlTouch, _: u32, time: u32, id: i32) {
        if let Some(index) = self.touch_points.iter().position(|touch| touch.id == id) {
            let mut touch = self.touch_points.swap_remove(index);
            touch.timestamp = Duration::from_millis(u64::from(time));
            let event = touch.pointer_event(self);
            self.push_window_event(touch.window_id, WindowEvent::PointerUp(event));
        }
    }

    fn motion(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &WlTouch,
        time: u32,
        id: i32,
        position: (f64, f64),
    ) {
        let Some(index) = self.touch_points.iter().position(|touch| touch.id == id) else {
            return;
        };
        let touch = &mut self.touch_points[index];
        touch.position = point(position);
        touch.timestamp = Duration::from_millis(u64::from(time));
        let window_id = touch.window_id;
        let event = self.touch_points[index].pointer_event(self);
        self.push_window_event(window_id, WindowEvent::PointerMove(event));
    }

    fn shape(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &WlTouch, _: i32, _: f64, _: f64) {}

    fn orientation(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &WlTouch, _: i32, _: f64) {}

    fn cancel(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &WlTouch) {
        for touch in std::mem::take(&mut self.touch_points) {
            let event = touch.pointer_event(self);
            self.push_window_event(touch.window_id, WindowEvent::PointerCancel(event));
        }
    }
}
