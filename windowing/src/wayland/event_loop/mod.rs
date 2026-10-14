mod active;
mod proxy;

use std::time::{Duration, Instant};

use sctk::reexports::{
    calloop::{self, ping::make_ping},
    calloop_wayland_source::WaylandSource,
    client::{Connection, globals::registry_queue_init},
};

pub use active::WaylandPlatform;
pub use proxy::EventLoopProxy;

use crate::{
    ApplicationHandler, ControlFlow, WindowEvent, WindowId,
    wayland::{WaylandEvent, error::Error, output::OutputEvent, protocols::CrownEvent},
};

pub(crate) enum QueuedEvent {
    Window(WindowId, WindowEvent),
    Output(OutputEvent),
    Crown(CrownEvent),
    WakeUp,
}

pub struct EventLoop {
    event_loop: calloop::EventLoop<'static, WaylandPlatform>,
    state: WaylandPlatform,
    spare_events: Vec<QueuedEvent>,
}

impl EventLoop {
    pub fn new() -> Result<Self, Error> {
        let connection = Connection::connect_to_env()?;
        let (globals, mut event_queue) = registry_queue_init(&connection)?;
        let event_loop = calloop::EventLoop::try_new()?;
        let loop_handle = event_loop.handle();

        let (proxy_waker, proxy_source) = make_ping()?;
        loop_handle
            .insert_source(proxy_source, |_, _, state: &mut WaylandPlatform| {
                state.pending_events.push(QueuedEvent::WakeUp);
            })
            .map_err(|failure| failure.error)?;

        let mut state = WaylandPlatform::new(
            connection.clone(),
            &globals,
            event_queue.handle(),
            loop_handle.clone(),
            proxy_waker,
        )?;
        let default_feedback = state
            .dmabuf_state
            .get_default_feedback(&event_queue.handle())
            .ok();
        event_queue.roundtrip(&mut state)?;
        if let Some(feedback) = default_feedback {
            feedback.destroy();
        }
        WaylandSource::new(connection, event_queue)
            .insert(loop_handle)
            .map_err(|failure| failure.error)?;

        Ok(Self {
            event_loop,
            state,
            spare_events: Vec::new(),
        })
    }

    /// The compositor's main GPU as a `dev_t`, when it advertises dmabuf feedback.
    pub fn main_device(&self) -> Option<u64> {
        self.state.main_device
    }

    pub fn create_proxy(&self) -> EventLoopProxy {
        self.state.create_proxy()
    }

    pub fn run_app<A: ApplicationHandler<WaylandPlatform>>(
        mut self,
        app: &mut A,
    ) -> Result<(), Error> {
        app.resumed(&mut self.state);
        loop {
            self.state.apply_cursor_changes();
            self.state.apply_text_input_changes();
            self.state.queue_requested_redraws();
            self.deliver_pending_events(app);
            if self.state.exit_requested {
                return Ok(());
            }

            app.about_to_wait(&mut self.state);
            self.state.apply_cursor_changes();
            self.state.apply_text_input_changes();
            self.state.queue_requested_redraws();
            let timeout = self.sleep_timeout();
            self.event_loop.dispatch(timeout, &mut self.state)?;
        }
    }

    fn deliver_pending_events<A: ApplicationHandler<WaylandPlatform>>(&mut self, app: &mut A) {
        while !self.state.pending_events.is_empty() {
            let mut events = std::mem::replace(
                &mut self.state.pending_events,
                std::mem::take(&mut self.spare_events),
            );
            for event in events.drain(..) {
                let state = &mut self.state;
                match event {
                    QueuedEvent::Window(window_id, event) => {
                        app.window_event(state, window_id, event)
                    }
                    QueuedEvent::Output(event) => {
                        app.platform_event(state, WaylandEvent::Output(event))
                    }
                    QueuedEvent::Crown(event) => {
                        match &event {
                            CrownEvent::SurfaceVisibility {
                                window,
                                state: visibility,
                                thumbnail_scale,
                            } => app.window_event(
                                state,
                                *window,
                                WindowEvent::Visibility {
                                    visibility: visibility.into(),
                                    thumbnail_scale: *thumbnail_scale as f32,
                                },
                            ),
                            CrownEvent::AppIconLaunch { window, app_id } => app.window_event(
                                state,
                                *window,
                                WindowEvent::LaunchApp(app_id.clone()),
                            ),
                            CrownEvent::AgentAccess { decision, .. } => {
                                let decision = crate::AgentAccess::from(decision.clone());
                                if decision == crate::AgentAccess::Revoked {
                                    state.agent_access_request = None;
                                }
                                app.agent_access(state, decision);
                            }
                            _ => {}
                        }
                        app.platform_event(state, WaylandEvent::Crown(event));
                    }
                    QueuedEvent::WakeUp => app.proxy_wake_up(state),
                }
            }
            self.spare_events = events;
        }
    }

    fn sleep_timeout(&self) -> Option<Duration> {
        if !self.state.pending_events.is_empty() {
            return Some(Duration::ZERO);
        }
        match self.state.control_flow {
            ControlFlow::Wait => None,
            ControlFlow::Poll => Some(Duration::ZERO),
            ControlFlow::WaitUntil(deadline) => {
                Some(deadline.saturating_duration_since(Instant::now()))
            }
        }
    }
}
