use std::collections::HashMap;

use impeller::ImpellerBackend;
use runtime::{RunError, Runner};
use taffy_layout::TaffyEngine;
use windowing::{
    AgentAccess, ApplicationHandler, Platform, WindowEvent, WindowId,
    wayland::{Output, OutputEvent, OutputId, WaylandEvent, WaylandPlatform},
};

use super::{
    Surfaces,
    reconcile::{Reconciliation, reconcile},
};

/// Keeps the surfaces of a [`Surfaces`] app in step with the monitors, on a runner that stays
/// alive without windows.
pub(crate) struct SurfaceRunner<App: Surfaces> {
    app: App,
    runner: Runner<ImpellerBackend, TaffyEngine>,
    outputs: HashMap<OutputId, Output>,
    placed: HashMap<OutputId, (u64, Vec<WindowId>)>,
    failure: Option<RunError>,
}

impl<App: Surfaces> SurfaceRunner<App> {
    pub(crate) fn new(app: App) -> Self {
        Self {
            app,
            runner: Runner::empty(true),
            outputs: HashMap::new(),
            placed: HashMap::new(),
            failure: None,
        }
    }

    pub(crate) fn take_failure(&mut self) -> Option<RunError> {
        self.failure.take().or_else(|| self.runner.take_failure())
    }

    fn track_output(&mut self, event: &OutputEvent) {
        match event {
            OutputEvent::Added(output) | OutputEvent::Changed(output) => {
                self.outputs.insert(output.id, output.clone());
            }
            OutputEvent::Removed(id) => {
                self.outputs.remove(id);
            }
        }
    }

    fn forget_closed_windows(&mut self) {
        for (_, windows) in self.placed.values_mut() {
            windows.retain(|&window| self.runner.has_window(window));
        }
    }

    fn reconcile_surfaces(&mut self, platform: &mut WaylandPlatform) {
        self.forget_closed_windows();
        let wanted: HashMap<OutputId, u64> = self
            .outputs
            .values()
            .filter_map(|output| Some((output.id, self.app.placement(output)?)))
            .collect();
        let Reconciliation { close, open } = reconcile(&self.placed, &wanted);
        for output in close {
            for window in self
                .placed
                .remove(&output)
                .into_iter()
                .flat_map(|(_, windows)| windows)
            {
                self.runner.close_window(platform, window);
            }
        }
        for (output, key) in open {
            match self.open_surfaces(platform, output) {
                Ok(windows) => {
                    self.placed.insert(output, (key, windows));
                }
                Err(error) => {
                    self.failure = Some(error);
                    return platform.exit();
                }
            }
        }
    }

    fn open_surfaces(
        &mut self,
        platform: &mut WaylandPlatform,
        output: OutputId,
    ) -> Result<Vec<WindowId>, RunError> {
        let Some(output) = self.outputs.get(&output) else {
            return Ok(Vec::new());
        };
        self.app
            .build(output)
            .into_iter()
            .map(|(root, mut config)| {
                if let Some(layer) = config.layer.as_mut() {
                    layer.output.clone_from(&output.name);
                }
                self.runner
                    .open_window(platform, root, TaffyEngine::new(), config)
            })
            .collect()
    }
}

impl<App: Surfaces> ApplicationHandler<WaylandPlatform> for SurfaceRunner<App> {
    fn resumed(&mut self, platform: &mut WaylandPlatform) {
        self.app.attach(Platform::waker(platform));
        self.runner.resumed(platform);
    }

    fn window_event(
        &mut self,
        platform: &mut WaylandPlatform,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        self.runner.window_event(platform, window_id, event);
    }

    fn platform_event(&mut self, platform: &mut WaylandPlatform, event: WaylandEvent) {
        let output_event = match &event {
            WaylandEvent::Output(output_event) => {
                self.track_output(output_event);
                true
            }
            WaylandEvent::Crown(_) => false,
        };
        self.runner.platform_event(platform, event);
        if output_event {
            self.reconcile_surfaces(platform);
        }
    }

    fn agent_access(&mut self, platform: &mut WaylandPlatform, decision: AgentAccess) {
        self.runner.agent_access(platform, decision);
    }

    fn proxy_wake_up(&mut self, platform: &mut WaylandPlatform) {
        self.runner.proxy_wake_up(platform);
        self.reconcile_surfaces(platform);
    }

    fn about_to_wait(&mut self, platform: &mut WaylandPlatform) {
        self.runner.about_to_wait(platform);
    }
}
