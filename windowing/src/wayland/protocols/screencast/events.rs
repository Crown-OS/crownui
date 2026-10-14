use std::{os::fd::OwnedFd, time::Duration};

use crownos_protocols::screencast::v1::client::{
    crownos_screencast_manager_v1::{self, CrownosScreencastManagerV1},
    crownos_screencast_session_v1::{self, CrownosScreencastSessionV1, StopReason},
};
use sctk::{
    dispatch2::Dispatch2,
    reexports::client::{Connection, Proxy, QueueHandle},
};

use super::ScreencastId;
use crate::wayland::{
    event_loop::{QueuedEvent, WaylandPlatform},
    protocols::CrownEvent,
    util::{bitfield::known_flags, u64_halves::join_u64},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DmabufFormat {
    pub fourcc: u32,
    pub modifiers: Vec<u64>,
}

/// What every buffer attached to the ring must look like.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BufferConstraints {
    pub width: u32,
    pub height: u32,
    pub formats: Vec<DmabufFormat>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DamageRect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScreencastFrame {
    pub slot: u32,
    /// Changed regions in buffer pixels; their union may exceed what really changed.
    pub damage: Vec<DamageRect>,
    /// CLOCK_MONOTONIC time at which the content was sampled.
    pub timestamp: Duration,
    /// Point to wait on before reading the slot when timelines are set, zero otherwise.
    pub acquire_point: u64,
}

/// A sealed, read-only memfd of premultiplied ARGB8888 pixels.
#[derive(Debug)]
pub struct CursorImage {
    pub fd: OwnedFd,
    pub width: u32,
    pub height: u32,
    pub stride: u32,
}

#[derive(Debug)]
pub enum ScreencastEvent {
    /// Allocate and attach buffers matching these; every slot has been emptied.
    Constraints(BufferConstraints),
    Frame(ScreencastFrame),
    CursorMoved {
        position: (i32, i32),
        hotspot: (i32, i32),
    },
    CursorLeft,
    /// `image` is `None` when the cursor was hidden.
    CursorShape {
        serial: u32,
        hotspot: (i32, i32),
        image: Option<CursorImage>,
    },
    Stopped(StopReason),
}

#[derive(Default)]
pub(crate) struct PendingScreencast {
    constraints: BufferConstraints,
    damage: Vec<DamageRect>,
    cursor_image: Option<CursorImage>,
}

pub(crate) struct ScreencastSessionData;

impl Dispatch2<CrownosScreencastSessionV1, WaylandPlatform> for ScreencastSessionData {
    fn event(
        &self,
        state: &mut WaylandPlatform,
        session: &CrownosScreencastSessionV1,
        event: crownos_screencast_session_v1::Event,
        _: &Connection,
        _: &QueueHandle<WaylandPlatform>,
    ) {
        use crownos_screencast_session_v1::Event;

        let pending = state.screencast_batches.entry(session.id()).or_default();
        let screencast_event = match event {
            Event::Constraints { width, height } => {
                pending.constraints = BufferConstraints {
                    width,
                    height,
                    formats: Vec::new(),
                };
                return;
            }
            Event::Format { format } => {
                pending.constraints.formats.push(DmabufFormat {
                    fourcc: format,
                    modifiers: Vec::new(),
                });
                return;
            }
            Event::Modifier {
                format,
                modifier_hi,
                modifier_lo,
            } => {
                if let Some(entry) = pending
                    .constraints
                    .formats
                    .iter_mut()
                    .rfind(|entry| entry.fourcc == format)
                {
                    entry.modifiers.push(join_u64(modifier_hi, modifier_lo));
                }
                return;
            }
            Event::ConstraintsDone => {
                ScreencastEvent::Constraints(std::mem::take(&mut pending.constraints))
            }
            Event::Damage {
                x,
                y,
                width,
                height,
            } => {
                pending.damage.push(DamageRect {
                    x,
                    y,
                    width,
                    height,
                });
                return;
            }
            Event::Frame {
                index,
                tv_sec_hi,
                tv_sec_lo,
                tv_nsec,
                acquire_point_hi,
                acquire_point_lo,
            } => ScreencastEvent::Frame(ScreencastFrame {
                slot: index,
                damage: std::mem::take(&mut pending.damage),
                timestamp: Duration::new(join_u64(tv_sec_hi, tv_sec_lo), tv_nsec),
                acquire_point: join_u64(acquire_point_hi, acquire_point_lo),
            }),
            Event::CursorPosition {
                x,
                y,
                hotspot_x,
                hotspot_y,
            } => ScreencastEvent::CursorMoved {
                position: (x, y),
                hotspot: (hotspot_x, hotspot_y),
            },
            Event::CursorLeave => ScreencastEvent::CursorLeft,
            Event::CursorBuffer {
                fd,
                width,
                height,
                stride,
            } => {
                pending.cursor_image = Some(CursorImage {
                    fd,
                    width,
                    height,
                    stride,
                });
                return;
            }
            Event::CursorShape {
                serial,
                hotspot_x,
                hotspot_y,
            } => ScreencastEvent::CursorShape {
                serial,
                hotspot: (hotspot_x, hotspot_y),
                image: pending.cursor_image.take(),
            },
            Event::Stopped { reason } => {
                ScreencastEvent::Stopped(reason.into_result().unwrap_or(StopReason::Revoked))
            }
            _ => return,
        };
        if matches!(screencast_event, ScreencastEvent::Stopped(_)) {
            state.screencast_batches.remove(&session.id());
        }
        state
            .pending_events
            .push(QueuedEvent::Crown(CrownEvent::Screencast {
                session: ScreencastId(session.id()),
                event: screencast_event,
            }));
    }
}

pub(crate) struct ScreencastManagerData;

impl Dispatch2<CrownosScreencastManagerV1, WaylandPlatform> for ScreencastManagerData {
    fn event(
        &self,
        state: &mut WaylandPlatform,
        _: &CrownosScreencastManagerV1,
        event: crownos_screencast_manager_v1::Event,
        _: &Connection,
        _: &QueueHandle<WaylandPlatform>,
    ) {
        let crownos_screencast_manager_v1::Event::Capabilities { capabilities } = event else {
            return;
        };
        if let Some(screencast) = state.crown.screencast.as_mut() {
            screencast.capabilities = known_flags(capabilities);
        }
    }
}
