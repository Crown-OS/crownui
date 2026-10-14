use std::time::Duration;

use crownos_protocols::input::v1::client::crownos_input_capture_v1::{
    self, ButtonState, CrownosInputCaptureV1, KeyState,
};
use sctk::{
    dispatch2::Dispatch2,
    reexports::client::{Connection, Proxy, QueueHandle, WEnum, backend::ObjectId},
};

use super::{CaptureEdges, ElementState, ScrollAxis};
use crate::wayland::{
    error::Error,
    event_loop::{QueuedEvent, WaylandPlatform},
    output::{Output, OutputId},
    protocols::CrownEvent,
    util::{bitfield::known_flags, u64_halves::join_u64},
};

/// Input seen while captured; buttons and keys are evdev codes, before any keymap.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CapturedInput {
    Motion {
        dx: f64,
        dy: f64,
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
    Modifiers {
        depressed: u32,
        latched: u32,
        locked: u32,
        group: u32,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum InputCaptureEvent {
    /// The pointer went through `edge` of `output` at `position` along that edge.
    Entered {
        output: Option<OutputId>,
        edge: CaptureEdges,
        position: f64,
    },
    /// One hardware event's worth of input, stamped on CLOCK_MONOTONIC.
    Input {
        events: Vec<CapturedInput>,
        time: Duration,
    },
    /// Input is back with the local seat and the capture is armed again.
    Released,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InputCaptureId(ObjectId);

/// Takes over the default seat's input when the pointer leaves the layout through an armed edge.
#[derive(Debug)]
pub struct InputCapture {
    proxy: CrownosInputCaptureV1,
}

impl WaylandPlatform {
    /// Creates a disarmed capture for the default seat.
    pub fn create_input_capture(&mut self) -> Result<InputCapture, Error> {
        let manager = self
            .crown
            .input
            .as_ref()
            .ok_or(Error::MissingGlobal("crownos_input_manager_v1"))?;
        let proxy = manager.create_capture(&self.default_seat()?, &self.queue_handle, CaptureData);
        let connection = &self.connection;
        self.capture_batches
            .retain(|id, _| connection.object_info(id.clone()).is_ok());
        Ok(InputCapture { proxy })
    }
}

impl InputCapture {
    pub fn id(&self) -> InputCaptureId {
        InputCaptureId(self.proxy.id())
    }

    /// Replaces the armed edges; an empty set disarms.
    pub fn arm(&self, edges: CaptureEdges) {
        self.proxy.arm(edges);
    }

    /// Ends the capture and puts the pointer at `x`, `y` in `output`'s logical space.
    pub fn release(&self, output: &Output, x: f64, y: f64) {
        self.proxy.release(&output.proxy, x, y);
    }
}

impl Drop for InputCapture {
    fn drop(&mut self) {
        self.proxy.destroy();
    }
}

pub(crate) struct CaptureData;

impl Dispatch2<CrownosInputCaptureV1, WaylandPlatform> for CaptureData {
    fn event(
        &self,
        state: &mut WaylandPlatform,
        capture: &CrownosInputCaptureV1,
        event: crownos_input_capture_v1::Event,
        _: &Connection,
        _: &QueueHandle<WaylandPlatform>,
    ) {
        use crownos_input_capture_v1::Event;

        let captured = match event {
            Event::Motion { dx, dy } => CapturedInput::Motion { dx, dy },
            Event::Button { button, state } => CapturedInput::Button {
                button,
                state: match state {
                    WEnum::Value(ButtonState::Pressed) => ElementState::Pressed,
                    _ => ElementState::Released,
                },
            },
            Event::Axis {
                axis,
                value,
                value120,
            } => CapturedInput::Axis {
                axis: match axis {
                    WEnum::Value(crownos_input_capture_v1::Axis::HorizontalScroll) => {
                        ScrollAxis::Horizontal
                    }
                    _ => ScrollAxis::Vertical,
                },
                value,
                value120,
            },
            Event::Key { key, state } => CapturedInput::Key {
                key,
                state: match state {
                    WEnum::Value(KeyState::Pressed) => ElementState::Pressed,
                    _ => ElementState::Released,
                },
            },
            Event::Modifiers {
                depressed,
                latched,
                locked,
                group,
            } => CapturedInput::Modifiers {
                depressed,
                latched,
                locked,
                group,
            },
            Event::Entered {
                output,
                edge,
                position,
            } => {
                return push_capture_event(
                    state,
                    capture,
                    InputCaptureEvent::Entered {
                        output: OutputId::of(&output),
                        edge: known_flags(edge),
                        position,
                    },
                );
            }
            Event::Frame {
                time_usec_hi,
                time_usec_lo,
            } => {
                let events = state
                    .capture_batches
                    .remove(&capture.id())
                    .unwrap_or_default();
                let time = Duration::from_micros(join_u64(time_usec_hi, time_usec_lo));
                return push_capture_event(
                    state,
                    capture,
                    InputCaptureEvent::Input { events, time },
                );
            }
            Event::Released => {
                state.capture_batches.remove(&capture.id());
                return push_capture_event(state, capture, InputCaptureEvent::Released);
            }
            _ => return,
        };
        state
            .capture_batches
            .entry(capture.id())
            .or_default()
            .push(captured);
    }
}

fn push_capture_event(
    state: &mut WaylandPlatform,
    capture: &CrownosInputCaptureV1,
    event: InputCaptureEvent,
) {
    state
        .pending_events
        .push(QueuedEvent::Crown(CrownEvent::InputCapture {
            capture: InputCaptureId(capture.id()),
            event,
        }));
}
