use std::time::Duration;

use crownos_protocols::input::v1::client::crownos_input_injector_v1::{
    self, ButtonState, CrownosInputInjectorV1, KeyState,
};
use sctk::{
    dispatch2::Dispatch2,
    reexports::client::{Connection, Proxy, QueueHandle, backend::ObjectId},
};

use super::ElementState;
use super::ScrollAxis;
use crate::wayland::{
    error::Error,
    event_loop::{QueuedEvent, WaylandPlatform},
    output::Output,
    protocols::CrownEvent,
    util::u64_halves::split_u64,
};

/// One event of an injected hardware frame. Coordinates are in the output's logical space;
/// buttons and keys are evdev codes.
#[derive(Debug, Clone, Copy)]
pub enum InjectedInput<'a> {
    PointerMotion {
        dx: f64,
        dy: f64,
    },
    PointerMotionAbsolute {
        output: &'a Output,
        x: f64,
        y: f64,
    },
    Button {
        button: u32,
        state: ElementState,
    },
    Axis {
        axis: ScrollAxis,
        value: f64,
        value120: i32,
    },
    Key {
        key: u32,
        state: ElementState,
    },
    /// `id` must not belong to a touch point that is still down.
    TouchDown {
        id: i32,
        output: &'a Output,
        x: f64,
        y: f64,
    },
    /// `id` must belong to a touch point that is down.
    TouchMotion {
        id: i32,
        x: f64,
        y: f64,
    },
    /// `id` must belong to a touch point that is down.
    TouchUp {
        id: i32,
    },
    TouchCancel,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InputInjectorId(ObjectId);

/// A virtual input device feeding one seat. Dropping it releases every key and button it holds.
#[derive(Debug)]
pub struct InputInjector {
    proxy: CrownosInputInjectorV1,
}

impl WaylandPlatform {
    /// Creates an injector for the default seat.
    pub fn create_input_injector(&self) -> Result<InputInjector, Error> {
        let manager = self
            .crown
            .input
            .as_ref()
            .ok_or(Error::MissingGlobal("crownos_input_manager_v1"))?;
        let proxy =
            manager.create_injector(&self.default_seat()?, &self.queue_handle, InjectorData);
        Ok(InputInjector { proxy })
    }
}

impl InputInjector {
    pub fn id(&self) -> InputInjectorId {
        InputInjectorId(self.proxy.id())
    }

    /// Applies `events` in order as one logical hardware event stamped with
    /// `time` on CLOCK_MONOTONIC.
    pub fn send_frame(&self, events: &[InjectedInput<'_>], time: Duration) {
        for event in events {
            self.add(event);
        }
        let micros = u64::try_from(time.as_micros()).unwrap_or(u64::MAX);
        let (high, low) = split_u64(micros);
        self.proxy.frame(high, low);
    }

    fn add(&self, event: &InjectedInput<'_>) {
        let proxy = &self.proxy;
        match *event {
            InjectedInput::PointerMotion { dx, dy } => proxy.pointer_motion(dx, dy),
            InjectedInput::PointerMotionAbsolute { output, x, y } => {
                proxy.pointer_motion_absolute(&output.proxy, x, y)
            }
            InjectedInput::Button { button, state } => proxy.button(
                button,
                match state {
                    ElementState::Pressed => ButtonState::Pressed,
                    ElementState::Released => ButtonState::Released,
                },
            ),
            InjectedInput::Axis {
                axis,
                value,
                value120,
            } => proxy.axis(
                match axis {
                    ScrollAxis::Vertical => crownos_input_injector_v1::Axis::VerticalScroll,
                    ScrollAxis::Horizontal => crownos_input_injector_v1::Axis::HorizontalScroll,
                },
                value,
                value120,
            ),
            InjectedInput::Key { key, state } => proxy.key(
                key,
                match state {
                    ElementState::Pressed => KeyState::Pressed,
                    ElementState::Released => KeyState::Released,
                },
            ),
            InjectedInput::TouchDown { id, output, x, y } => {
                proxy.touch_down(id, &output.proxy, x, y)
            }
            InjectedInput::TouchMotion { id, x, y } => proxy.touch_motion(id, x, y),
            InjectedInput::TouchUp { id } => proxy.touch_up(id),
            InjectedInput::TouchCancel => proxy.touch_cancel(),
        }
    }
}

impl Drop for InputInjector {
    fn drop(&mut self) {
        self.proxy.destroy();
    }
}

pub(crate) struct InjectorData;

impl Dispatch2<CrownosInputInjectorV1, WaylandPlatform> for InjectorData {
    fn event(
        &self,
        state: &mut WaylandPlatform,
        injector: &CrownosInputInjectorV1,
        event: crownos_input_injector_v1::Event,
        _: &Connection,
        _: &QueueHandle<WaylandPlatform>,
    ) {
        if let crownos_input_injector_v1::Event::Revoked = event {
            state
                .pending_events
                .push(QueuedEvent::Crown(CrownEvent::InputInjectorRevoked(
                    InputInjectorId(injector.id()),
                )));
        }
    }
}
