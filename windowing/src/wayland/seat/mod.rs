mod gesture_tracker;
mod keyboard;
pub(crate) mod keymap;
mod pointer;
mod pointer_gestures;
mod touch;

pub(crate) use touch::TouchPoint;

use sctk::{
    data_device_manager::data_device::DataDevice,
    reexports::client::{
        Connection, QueueHandle,
        protocol::{wl_keyboard::WlKeyboard, wl_seat::WlSeat, wl_touch::WlTouch},
    },
    seat::{
        Capability, SeatHandler, SeatState,
        pointer::{ThemeSpec, ThemedPointer},
    },
};

use crate::{
    WindowEvent, WindowId,
    wayland::{event_loop::WaylandPlatform, text_input::SeatTextInput},
};
use pointer_gestures::SeatGestures;

pub(crate) struct SeatDevices {
    pub(crate) seat: WlSeat,
    keyboard: Option<WlKeyboard>,
    pub(crate) pointer: Option<ThemedPointer>,
    touch: Option<WlTouch>,
    pub(crate) pointer_focus: Option<WindowId>,
    gestures: Option<SeatGestures>,
    pub(crate) text_input: Option<SeatTextInput>,
    pub(crate) data_device: Option<DataDevice>,
}

impl WaylandPlatform {
    fn seat_index(&mut self, seat: &WlSeat) -> usize {
        self.seats
            .iter()
            .position(|devices| &devices.seat == seat)
            .unwrap_or_else(|| {
                let qh = &self.queue_handle;
                self.seats.push(SeatDevices {
                    seat: seat.clone(),
                    keyboard: None,
                    pointer: None,
                    touch: None,
                    pointer_focus: None,
                    gestures: None,
                    text_input: self
                        .standard
                        .text_input
                        .as_ref()
                        .map(|manager| SeatTextInput::new(manager, seat, qh)),
                    data_device: self
                        .standard
                        .data_device
                        .as_ref()
                        .map(|manager| manager.get_data_device(qh, seat)),
                });
                self.seats.len() - 1
            })
    }
}

impl SeatHandler for WaylandPlatform {
    fn seat_state(&mut self) -> &mut SeatState {
        &mut self.seat_state
    }

    fn new_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, seat: WlSeat) {
        self.seat_index(&seat);
    }

    fn new_capability(
        &mut self,
        _: &Connection,
        qh: &QueueHandle<Self>,
        seat: WlSeat,
        capability: Capability,
    ) {
        let index = self.seat_index(&seat);
        let devices = &mut self.seats[index];
        match capability {
            Capability::Keyboard if devices.keyboard.is_none() => {
                devices.keyboard = self
                    .seat_state
                    .get_keyboard_with_repeat(
                        qh,
                        &seat,
                        None,
                        self.loop_handle.clone(),
                        Box::new(|state, _, event| {
                            let event = state.keyboard_event(event, true);
                            state.queue_key_event(WindowEvent::KeyDown(event));
                        }),
                    )
                    .ok();
            }
            Capability::Pointer if devices.pointer.is_none() => {
                devices.pointer = self
                    .seat_state
                    .get_pointer_with_theme::<_, ()>(
                        qh,
                        &seat,
                        self.shm.wl_shm(),
                        self.compositor_state.create_surface(qh),
                        ThemeSpec::default(),
                    )
                    .ok();
                devices.gestures = self
                    .standard
                    .pointer_gestures
                    .as_ref()
                    .zip(devices.pointer.as_ref())
                    .map(|(manager, themed)| SeatGestures::new(manager, themed.pointer(), qh));
            }
            Capability::Touch if devices.touch.is_none() => {
                devices.touch = self.seat_state.get_touch(qh, &seat).ok();
            }
            _ => {}
        }
    }

    fn remove_capability(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        seat: WlSeat,
        capability: Capability,
    ) {
        let Some(devices) = self.seats.iter_mut().find(|devices| devices.seat == seat) else {
            return;
        };
        match capability {
            Capability::Keyboard => {
                if let Some(keyboard) = devices.keyboard.take() {
                    keyboard.release();
                }
            }
            Capability::Pointer => {
                devices.gestures = None;
                if let Some(pointer) = devices.pointer.take() {
                    pointer.pointer().release();
                }
                devices.pointer_focus = None;
            }
            Capability::Touch => {
                if let Some(touch) = devices.touch.take() {
                    touch.release();
                }
            }
            _ => {}
        }
    }

    fn remove_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, seat: WlSeat) {
        self.seats.retain(|devices| devices.seat != seat);
    }
}
