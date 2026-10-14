mod material;
mod surface;

pub use crownos_protocols::background_effects::v1::client::crownos_background_effect_manager_v1::Capability as MaterialCapabilities;
use crownos_protocols::background_effects::v1::client::{
    crownos_background_effect_manager_v1::{self, CrownosBackgroundEffectManagerV1},
    crownos_shape_manager_v1::CrownosShapeManagerV1,
};
pub use material::{Argb, BackgroundEffect, Blur, Shadow, Shape, ShapePrimitive};
use sctk::{
    dispatch2::Dispatch2,
    reexports::client::{
        Connection, QueueHandle, globals::GlobalList, protocol::wl_surface::WlSurface,
    },
};
pub(crate) use surface::{BackgroundEffectSurface, EffectProxies};

use super::CrownEvent;
use crate::wayland::{
    event_loop::{QueuedEvent, WaylandPlatform},
    util::{bitfield::known_flags, no_events::NoEvents},
};

pub(crate) struct MaterialGlobals {
    effects: CrownosBackgroundEffectManagerV1,
    shapes: CrownosShapeManagerV1,
    capabilities: MaterialCapabilities,
}

impl MaterialGlobals {
    pub(crate) fn bind(globals: &GlobalList, qh: &QueueHandle<WaylandPlatform>) -> Option<Self> {
        Some(Self {
            effects: globals.bind(qh, 1..=1, EffectManagerData).ok()?,
            shapes: globals.bind(qh, 1..=1, NoEvents).ok()?,
            capabilities: MaterialCapabilities::empty(),
        })
    }

    pub(crate) fn effect_surface(
        &self,
        surface: &WlSurface,
        qh: &QueueHandle<WaylandPlatform>,
    ) -> BackgroundEffectSurface {
        BackgroundEffectSurface(EffectProxies {
            effect: self.effects.get_effect_surface(surface, qh, NoEvents),
            shapes: self.shapes.clone(),
            queue_handle: qh.clone(),
        })
    }
}

impl WaylandPlatform {
    /// Effects the compositor currently renders; empty when it lacks `crownos_background_effects`.
    /// A window relying on a withdrawn effect should paint its own background instead.
    pub fn material_capabilities(&self) -> MaterialCapabilities {
        self.crown
            .material
            .as_ref()
            .map_or(MaterialCapabilities::empty(), |material| {
                material.capabilities
            })
    }
}

pub(crate) struct EffectManagerData;

impl Dispatch2<CrownosBackgroundEffectManagerV1, WaylandPlatform> for EffectManagerData {
    fn event(
        &self,
        state: &mut WaylandPlatform,
        _: &CrownosBackgroundEffectManagerV1,
        event: crownos_background_effect_manager_v1::Event,
        _: &Connection,
        _: &QueueHandle<WaylandPlatform>,
    ) {
        let crownos_background_effect_manager_v1::Event::Capabilities { capabilities } = event
        else {
            return;
        };
        let capabilities = known_flags(capabilities);
        if let Some(material) = state.crown.material.as_mut() {
            material.capabilities = capabilities;
        }
        state
            .pending_events
            .push(QueuedEvent::Crown(CrownEvent::MaterialCapabilitiesChanged(
                capabilities,
            )));
    }
}
