use sctk::{
    dispatch2::Dispatch2,
    reexports::{
        client::{
            Connection, Proxy, QueueHandle,
            protocol::{wl_pointer::WlPointer, wl_surface::WlSurface},
        },
        protocols::wp::pointer_gestures::zv1::client::{
            zwp_pointer_gesture_hold_v1::{self, ZwpPointerGestureHoldV1},
            zwp_pointer_gesture_pinch_v1::{self, ZwpPointerGesturePinchV1},
            zwp_pointer_gesture_swipe_v1::{self, ZwpPointerGestureSwipeV1},
            zwp_pointer_gestures_v1::ZwpPointerGesturesV1,
        },
    },
};

use super::gesture_tracker::{GestureInput, GestureKind, GestureTracker};
use crate::{
    WindowEvent,
    wayland::{event_loop::WaylandPlatform, window::surface_window_id},
};

const HOLD_GESTURE_VERSION: u32 = 3;

/// The swipe, pinch and (from version 3) hold gesture objects of one seat's pointer.
pub(crate) struct SeatGestures {
    swipe: ZwpPointerGestureSwipeV1,
    pinch: ZwpPointerGesturePinchV1,
    hold: Option<ZwpPointerGestureHoldV1>,
    swipe_tracker: GestureTracker,
    pinch_tracker: GestureTracker,
    hold_tracker: GestureTracker,
}

impl SeatGestures {
    pub(crate) fn new(
        manager: &ZwpPointerGesturesV1,
        pointer: &WlPointer,
        qh: &QueueHandle<WaylandPlatform>,
    ) -> Self {
        Self {
            swipe: manager.get_swipe_gesture(pointer, qh, GestureData),
            pinch: manager.get_pinch_gesture(pointer, qh, GestureData),
            hold: (manager.version() >= HOLD_GESTURE_VERSION)
                .then(|| manager.get_hold_gesture(pointer, qh, GestureData)),
            swipe_tracker: GestureTracker::new(GestureKind::Swipe),
            pinch_tracker: GestureTracker::new(GestureKind::Pinch),
            hold_tracker: GestureTracker::new(GestureKind::Hold),
        }
    }
}

impl Drop for SeatGestures {
    fn drop(&mut self) {
        self.swipe.destroy();
        self.pinch.destroy();
        if let Some(hold) = &self.hold {
            hold.destroy();
        }
    }
}

impl WaylandPlatform {
    fn begin_input(&self, surface: &WlSurface, fingers: u32) -> Option<GestureInput> {
        let window = surface_window_id(surface);
        self.windows
            .contains_key(&window)
            .then_some(GestureInput::Begin { window, fingers })
    }

    fn track_gesture(
        &mut self,
        input: Option<GestureInput>,
        tracker_of: impl Fn(&mut SeatGestures) -> Option<&mut GestureTracker>,
    ) {
        let Some(input) = input else {
            return;
        };
        let routed = self
            .seats
            .iter_mut()
            .filter_map(|devices| devices.gestures.as_mut())
            .find_map(tracker_of)
            .and_then(|tracker| tracker.track(input));
        if let Some((window_id, gesture)) = routed {
            self.push_window_event(window_id, WindowEvent::TouchpadGesture(gesture));
        }
    }
}

/// User data of every gesture object; the owning seat is found by comparing proxies.
pub(crate) struct GestureData;

impl Dispatch2<ZwpPointerGestureSwipeV1, WaylandPlatform> for GestureData {
    fn event(
        &self,
        state: &mut WaylandPlatform,
        swipe: &ZwpPointerGestureSwipeV1,
        event: zwp_pointer_gesture_swipe_v1::Event,
        _: &Connection,
        _: &QueueHandle<WaylandPlatform>,
    ) {
        let input = match event {
            zwp_pointer_gesture_swipe_v1::Event::Begin {
                surface, fingers, ..
            } => state.begin_input(&surface, fingers),
            zwp_pointer_gesture_swipe_v1::Event::Update { dx, dy, .. } => {
                Some(GestureInput::swipe_update(dx, dy))
            }
            zwp_pointer_gesture_swipe_v1::Event::End { cancelled, .. } => {
                Some(GestureInput::end(cancelled))
            }
            _ => None,
        };
        state.track_gesture(input, |gestures| {
            (&gestures.swipe == swipe).then_some(&mut gestures.swipe_tracker)
        });
    }
}

impl Dispatch2<ZwpPointerGesturePinchV1, WaylandPlatform> for GestureData {
    fn event(
        &self,
        state: &mut WaylandPlatform,
        pinch: &ZwpPointerGesturePinchV1,
        event: zwp_pointer_gesture_pinch_v1::Event,
        _: &Connection,
        _: &QueueHandle<WaylandPlatform>,
    ) {
        let input = match event {
            zwp_pointer_gesture_pinch_v1::Event::Begin {
                surface, fingers, ..
            } => state.begin_input(&surface, fingers),
            zwp_pointer_gesture_pinch_v1::Event::Update {
                dx,
                dy,
                scale,
                rotation,
                ..
            } => Some(GestureInput::pinch_update(dx, dy, scale, rotation)),
            zwp_pointer_gesture_pinch_v1::Event::End { cancelled, .. } => {
                Some(GestureInput::end(cancelled))
            }
            _ => None,
        };
        state.track_gesture(input, |gestures| {
            (&gestures.pinch == pinch).then_some(&mut gestures.pinch_tracker)
        });
    }
}

impl Dispatch2<ZwpPointerGestureHoldV1, WaylandPlatform> for GestureData {
    fn event(
        &self,
        state: &mut WaylandPlatform,
        hold: &ZwpPointerGestureHoldV1,
        event: zwp_pointer_gesture_hold_v1::Event,
        _: &Connection,
        _: &QueueHandle<WaylandPlatform>,
    ) {
        let input = match event {
            zwp_pointer_gesture_hold_v1::Event::Begin {
                surface, fingers, ..
            } => state.begin_input(&surface, fingers),
            zwp_pointer_gesture_hold_v1::Event::End { cancelled, .. } => {
                Some(GestureInput::end(cancelled))
            }
            _ => None,
        };
        state.track_gesture(input, |gestures| {
            (gestures.hold.as_ref() == Some(hold)).then_some(&mut gestures.hold_tracker)
        });
    }
}
