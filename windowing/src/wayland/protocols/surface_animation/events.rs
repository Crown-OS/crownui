use crownos_protocols::surface_animation::v1::client::crownos_animated_surface_v1::{
    self, CrownosAnimatedSurfaceV1, Property,
};
use sctk::{
    dispatch2::Dispatch2,
    reexports::client::{Connection, Proxy, QueueHandle, WEnum},
};

use super::{super::CrownEvent, SurfaceAnimationId};
use crate::wayland::event_loop::{QueuedEvent, WaylandPlatform};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SurfaceAnimationEvent {
    /// `property` reached its target and stopped at `value`.
    Settled { property: Property, value: f32 },
    /// A new animation replaced a running one, continuing from `value` and
    /// `velocity` (units per second) at its start time.
    Interrupted {
        property: Property,
        value: f32,
        velocity: f32,
    },
}

pub(crate) struct AnimatedSurfaceData;

impl Dispatch2<CrownosAnimatedSurfaceV1, WaylandPlatform> for AnimatedSurfaceData {
    fn event(
        &self,
        state: &mut WaylandPlatform,
        animation: &CrownosAnimatedSurfaceV1,
        event: crownos_animated_surface_v1::Event,
        _: &Connection,
        _: &QueueHandle<WaylandPlatform>,
    ) {
        let event = match event {
            crownos_animated_surface_v1::Event::Settled {
                property: WEnum::Value(property),
                value,
            } => SurfaceAnimationEvent::Settled {
                property,
                value: value as f32,
            },
            crownos_animated_surface_v1::Event::Interrupted {
                property: WEnum::Value(property),
                value,
                velocity,
            } => SurfaceAnimationEvent::Interrupted {
                property,
                value: value as f32,
                velocity: velocity as f32,
            },
            _ => return,
        };
        state
            .pending_events
            .push(QueuedEvent::Crown(CrownEvent::SurfaceAnimation {
                surface: SurfaceAnimationId(animation.id()),
                event,
            }));
    }
}
