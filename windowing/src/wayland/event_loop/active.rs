use std::collections::HashMap;

use sctk::{
    compositor::CompositorState,
    dmabuf::DmabufState,
    output::OutputState,
    reexports::{
        calloop::{LoopHandle, ping::Ping},
        client::{
            Connection, QueueHandle, backend::ObjectId, globals::GlobalList,
            protocol::wl_seat::WlSeat,
        },
    },
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    seat::SeatState,
    shell::xdg::XdgShell,
    shm::{Shm, ShmHandler},
};

use input::Modifiers;

use super::{EventLoopProxy, QueuedEvent};
use crate::{
    ControlFlow, WindowId,
    wayland::{
    clipboard::{ActiveDrag, DropTarget, OwnedSelection},
    error::Error,
    globals::StandardGlobals,
    protocols::{CrownGlobals, input::CapturedInput, screencast::PendingScreencast},
    seat::{SeatDevices, TouchPoint},
    window::{Window, scaling::ScalingGlobals},
    },
};

/// The running event loop, handed to every [`ApplicationHandler`](crate::ApplicationHandler) callback.
pub struct WaylandPlatform {
    pub(crate) connection: Connection,
    pub(crate) queue_handle: QueueHandle<Self>,
    pub(crate) loop_handle: LoopHandle<'static, Self>,
    pub(crate) registry_state: RegistryState,
    pub(crate) compositor_state: CompositorState,
    pub(crate) xdg_shell: XdgShell,
    pub(crate) shm: Shm,
    pub(crate) seat_state: SeatState,
    pub(crate) output_state: OutputState,
    pub(crate) dmabuf_state: DmabufState,
    pub(crate) main_device: Option<u64>,
    pub(crate) scaling: Option<ScalingGlobals>,
    pub(crate) crown: CrownGlobals,
    pub(crate) standard: StandardGlobals,
    pub(crate) clipboard: Option<OwnedSelection>,
    pub(crate) drop_target: Option<DropTarget>,
    pub(crate) drag: Option<ActiveDrag>,
    pub(crate) input_serial: u32,
    pub(crate) windows: HashMap<WindowId, Window>,
    pub(crate) agent_access_request: Option<crate::wayland::protocols::agent_access::AgentAccessRequest>,
    pub(crate) visibility_observers: HashMap<WindowId, crate::wayland::protocols::surface_visibility::SurfaceVisibilityObserver>,
    pub(crate) seats: Vec<SeatDevices>,
    pub(crate) keyboard_focus: Option<WindowId>,
    pub(crate) modifiers: Modifiers,
    pub(crate) touch_points: Vec<TouchPoint>,
    pub(crate) pending_events: Vec<QueuedEvent>,
    pub(crate) screencast_batches: HashMap<ObjectId, PendingScreencast>,
    pub(crate) capture_batches: HashMap<ObjectId, Vec<CapturedInput>>,
    pub(crate) proxy_waker: Ping,
    pub(crate) control_flow: ControlFlow,
    pub(crate) exit_requested: bool,
}

impl WaylandPlatform {
    pub(crate) fn new(
        connection: Connection,
        globals: &GlobalList,
        queue_handle: QueueHandle<Self>,
        loop_handle: LoopHandle<'static, Self>,
        proxy_waker: Ping,
    ) -> Result<Self, Error> {
        let qh = &queue_handle;
        let required = |interface| move |source| Error::RequiredGlobal { interface, source };
        Ok(Self {
            registry_state: RegistryState::new(globals),
            compositor_state: CompositorState::bind(globals, qh)
                .map_err(required("wl_compositor"))?,
            xdg_shell: XdgShell::bind(globals, qh).map_err(required("xdg_wm_base"))?,
            shm: Shm::bind(globals, qh).map_err(required("wl_shm"))?,
            seat_state: SeatState::new(globals, qh),
            output_state: OutputState::new(globals, qh),
            dmabuf_state: DmabufState::new(globals, qh),
            main_device: None,
            scaling: ScalingGlobals::bind(globals, qh),
            crown: CrownGlobals::bind(globals, qh),
            standard: StandardGlobals::bind(globals, qh),
            clipboard: None,
            drop_target: None,
            drag: None,
            input_serial: 0,
            windows: HashMap::new(),
            agent_access_request: None,
            visibility_observers: HashMap::new(),
            seats: Vec::new(),
            keyboard_focus: None,
            modifiers: Modifiers::NONE,
            touch_points: Vec::new(),
            pending_events: Vec::new(),
            screencast_batches: HashMap::new(),
            capture_batches: HashMap::new(),
            proxy_waker,
            control_flow: ControlFlow::default(),
            exit_requested: false,
            connection,
            queue_handle,
            loop_handle,
        })
    }

    pub fn exit(&mut self) {
        self.exit_requested = true;
    }

    pub fn exiting(&self) -> bool {
        self.exit_requested
    }

    pub fn set_control_flow(&mut self, control_flow: ControlFlow) {
        self.control_flow = control_flow;
    }

    pub fn control_flow(&self) -> ControlFlow {
        self.control_flow
    }

    pub fn create_proxy(&self) -> EventLoopProxy {
        EventLoopProxy {
            waker: self.proxy_waker.clone(),
        }
    }

    pub fn connection(&self) -> &Connection {
        &self.connection
    }

    pub(crate) fn default_seat(&self) -> Result<WlSeat, Error> {
        self.seat_state.seats().next().ok_or(Error::NoSeat)
    }
}

impl ShmHandler for WaylandPlatform {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm
    }
}

impl ProvidesRegistryState for WaylandPlatform {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }

    registry_handlers![OutputState, SeatState];
}

sctk::delegate_registry!(WaylandPlatform);
sctk::delegate_dispatch2!(WaylandPlatform);
