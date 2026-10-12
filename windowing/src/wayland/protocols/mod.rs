pub mod agent_access;
pub mod app_icons;
pub mod background_effects;
pub mod dmabuf;
pub mod input;
pub mod screencast;
pub mod surface_animation;
pub mod surface_visibility;
pub mod virtual_output;

use crownos_protocols::{
    agent_access::v1::client::crownos_agent_access_manager_v1::CrownosAgentAccessManagerV1,
    input::v1::client::crownos_input_manager_v1::CrownosInputManagerV1,
    virtual_output::v1::client::crownos_virtual_output_manager_v1::CrownosVirtualOutputManagerV1,
};
use sctk::reexports::client::{QueueHandle, globals::GlobalList};

use crate::{
    WindowId,
    wayland::{event_loop::WaylandPlatform, util::no_events::NoEvents},
};
use agent_access::{AgentAccessDecision, AgentAccessId};
use app_icons::AppIconsGlobal;
use background_effects::{MaterialCapabilities, MaterialGlobals};
use input::{InputCaptureEvent, InputCaptureId, InputInjectorId};
use screencast::{ScreencastEvent, ScreencastGlobal, ScreencastId};
use surface_animation::{SurfaceAnimationEvent, SurfaceAnimationGlobals, SurfaceAnimationId};
use surface_visibility::{SurfaceVisibility, SurfaceVisibilityGlobal};
use virtual_output::{VirtualOutputEvent, VirtualOutputId};

/// The CrownOS globals the compositor offers; each is absent on other compositors
/// and the privileged ones are absent for sandboxed clients.
pub(crate) struct CrownGlobals {
    pub(crate) material: Option<MaterialGlobals>,
    pub(crate) screencast: Option<ScreencastGlobal>,
    pub(crate) virtual_outputs: Option<CrownosVirtualOutputManagerV1>,
    pub(crate) input: Option<CrownosInputManagerV1>,
    pub(crate) visibility: Option<SurfaceVisibilityGlobal>,
    pub(crate) agent_access: Option<CrownosAgentAccessManagerV1>,
    pub(crate) surface_animation: Option<SurfaceAnimationGlobals>,
    pub(crate) app_icons: Option<AppIconsGlobal>,
}

impl CrownGlobals {
    pub(crate) fn bind(globals: &GlobalList, qh: &QueueHandle<WaylandPlatform>) -> Self {
        Self {
            material: MaterialGlobals::bind(globals, qh),
            screencast: ScreencastGlobal::bind(globals, qh),
            virtual_outputs: globals.bind(qh, 1..=1, NoEvents).ok(),
            input: globals.bind(qh, 1..=1, NoEvents).ok(),
            visibility: SurfaceVisibilityGlobal::bind(globals, qh),
            agent_access: globals.bind(qh, 1..=1, NoEvents).ok(),
            surface_animation: SurfaceAnimationGlobals::bind(globals, qh),
            app_icons: AppIconsGlobal::bind(globals, qh),
        }
    }
}

#[derive(Debug)]
pub enum CrownEvent {
    MaterialCapabilitiesChanged(MaterialCapabilities),
    Screencast {
        session: ScreencastId,
        event: ScreencastEvent,
    },
    VirtualOutput {
        output: VirtualOutputId,
        event: VirtualOutputEvent,
    },
    InputCapture {
        capture: InputCaptureId,
        event: InputCaptureEvent,
    },
    /// The injector stopped working; drop it.
    InputInjectorRevoked(InputInjectorId),
    /// How the compositor shows `window`; `thumbnail_scale` is 1.0 normally and below
    /// 1.0 while the window is a thumbnail in an overview.
    SurfaceVisibility {
        window: WindowId,
        state: SurfaceVisibility,
        thumbnail_scale: f64,
    },
    AgentAccess {
        request: AgentAccessId,
        decision: AgentAccessDecision,
    },
    SurfaceAnimation {
        surface: SurfaceAnimationId,
        event: SurfaceAnimationEvent,
    },
    /// The user activated the icon of `app_id` on `window` and the app has no windows: start it.
    AppIconLaunch {
        window: WindowId,
        app_id: String,
    },
}
