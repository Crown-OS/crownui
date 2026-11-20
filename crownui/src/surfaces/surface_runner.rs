use std::collections::HashMap;

use impeller::ImpellerBackend;
use runtime::{RunError, Runner};
use taffy_layout::TaffyEngine;
use windowing::{
    AgentAccess, ApplicationHandler, Platform, WindowEvent, WindowId,
    wayland::{Output, OutputId, WaylandEvent, WaylandPlatform},
};

use super::{
    Surfaces,
    announced_outputs::AnnouncedOutputs,
    reconcile::{Reconciliation, reconcile},
};

/// Keeps the surfaces of a [`Surfaces`] app in step with the monitors, on a runner that stays
/// alive without windows.
pub(crate) struct SurfaceRunner<App: Surfaces> {
    app: App,
    runner: Runner<ImpellerBackend, TaffyEngine>,
    outputs: AnnouncedOutputs<Output>,
    placed: HashMap<OutputId, (u64, Vec<WindowId>)>,
    failure: Option<RunError>,
}

impl<App: Surfaces> SurfaceRunner<App> {
    pub(crate) fn new(app: App) -> Self {
        Self {
            app,
            runner: Runner::empty(true),
            outputs: AnnouncedOutputs::default(),
            placed: HashMap::new(),
            failure: None,
        }
    }

    pub(crate) fn take_failure(&mut self) -> Option<RunError> {
        self.failure.take().or_else(|| self.runner.take_failure())
    }

    fn forget_closed_windows(&mut self) {
        for (_, windows) in self.placed.values_mut() {
            windows.retain(|&window| self.runner.has_window(window));
        }
    }

    fn reconcile_surfaces(&mut self, platform: &mut WaylandPlatform) {
        let Some(outputs) = self.outputs.take_out_of_date() else {
            return;
        };
        self.app.outputs(outputs);
        let wanted: HashMap<OutputId, u64> = outputs
            .iter()
            .filter_map(|output| Some((output.id, self.app.placement(output)?)))
            .collect();
        self.forget_closed_windows();
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
        let Some(output) = self.outputs.get(output) else {
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
        if let WaylandEvent::Output(output_event) = &event {
            self.outputs.apply(output_event);
        }
        self.runner.platform_event(platform, event);
    }

    fn agent_access(&mut self, platform: &mut WaylandPlatform, decision: AgentAccess) {
        self.runner.agent_access(platform, decision);
    }

    fn proxy_wake_up(&mut self, platform: &mut WaylandPlatform) {
        self.runner.proxy_wake_up(platform);
        self.outputs.wake();
    }

    fn about_to_wait(&mut self, platform: &mut WaylandPlatform) {
        self.reconcile_surfaces(platform);
        self.runner.about_to_wait(platform);
    }
}
