mod animation;
mod events;
mod subsurface;

pub use animation::{SurfaceAnimation, SurfaceAnimationId};
pub use crownos_protocols::surface_animation::v1::client::crownos_animated_surface_v1::Property as AnimatedProperty;
pub use events::SurfaceAnimationEvent;
pub use subsurface::AnimatedSubsurface;

pub use crate::wayland::util::monotonic::monotonic_now;

use crownos_protocols::surface_animation::v1::client::crownos_surface_animation_manager_v1::CrownosSurfaceAnimationManagerV1;
use sctk::reexports::client::{
    QueueHandle,
    globals::GlobalList,
    protocol::{wl_subcompositor::WlSubcompositor, wl_surface::WlSurface},
};

use crate::wayland::{
    Window, error::Error, event_loop::WaylandPlatform, util::no_events::NoEvents,
};
use events::AnimatedSurfaceData;

const MANAGER: &str = "crownos_surface_animation_manager_v1";

pub(crate) struct SurfaceAnimationGlobals {
    manager: CrownosSurfaceAnimationManagerV1,
    subcompositor: Option<WlSubcompositor>,
}

impl SurfaceAnimationGlobals {
    pub(crate) fn bind(globals: &GlobalList, qh: &QueueHandle<WaylandPlatform>) -> Option<Self> {
        Some(Self {
            manager: globals.bind(qh, 1..=1, NoEvents).ok()?,
            subcompositor: globals.bind(qh, 1..=1, NoEvents).ok(),
        })
    }

    fn animate(&self, surface: &WlSurface, qh: &QueueHandle<WaylandPlatform>) -> SurfaceAnimation {
        SurfaceAnimation {
            proxy: self
                .manager
                .get_animated_surface(surface, qh, AnimatedSurfaceData),
        }
    }
}

impl WaylandPlatform {
    /// Hands `surface`, which must already be a `wl_subsurface`, to the compositor's
    /// springs; events arrive as [`CrownEvent::SurfaceAnimation`](super::CrownEvent::SurfaceAnimation).
    pub fn animate_surface(&self, surface: &WlSurface) -> Result<SurfaceAnimation, Error> {
        let globals = self
            .crown
            .surface_animation
            .as_ref()
            .ok_or(Error::MissingGlobal(MANAGER))?;
        Ok(globals.animate(surface, &self.queue_handle))
    }

    /// A new synchronized subsurface of `parent` with its animation object.
    pub fn create_animated_subsurface(&self, parent: &Window) -> Result<AnimatedSubsurface, Error> {
        let globals = self
            .crown
            .surface_animation
            .as_ref()
            .ok_or(Error::MissingGlobal(MANAGER))?;
        let subcompositor = globals
            .subcompositor
            .as_ref()
            .ok_or(Error::MissingGlobal("wl_subcompositor"))?;
        let qh = &self.queue_handle;
        let surface = self.compositor_state.create_surface(qh);
        let subsurface = subcompositor.get_subsurface(&surface, parent.wl_surface(), qh, NoEvents);
        Ok(AnimatedSubsurface {
            animation: globals.animate(&surface, qh),
            subsurface,
            surface,
            connection: self.connection.clone(),
        })
    }
}
