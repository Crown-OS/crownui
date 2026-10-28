use std::time::Instant;

use graphics::{CreateRenderer, GraphicsBackend, GraphicsError};
use layouting::LayoutEngine;
use view::View;
use windowing::{ApplicationHandler, ControlFlow, Platform, WindowEvent, WindowId};

use crate::{config::AppConfig, window_runner::WindowRunner};

/// Why an app could not start.
#[derive(Debug, thiserror::Error)]
pub enum RunError {
    /// The graphics backend failed to initialize.
    #[error(transparent)]
    Graphics(#[from] GraphicsError),
    /// The platform failed.
    #[error("platform error: {0}")]
    Platform(String),
    /// A thread could not be spawned.
    #[error("could not spawn a thread: {0}")]
    Thread(#[from] std::io::Error),
}

/// Drives a CrownUI app on any [`Platform`] with any graphics backend and layout engine:
/// the UI thread runs here, rendering happens on a dedicated render thread per window.
///
/// Every window has a UI of its own; windows share nothing but the thread they run on, so
/// state they both show reaches each of them the way any outside state does (`cx.spawn`).
pub struct Runner<G: GraphicsBackend, L: LayoutEngine> {
    windows: Vec<WindowRunner<G, L>>,
    epoch: Instant,
    failure: Option<RunError>,
}

impl<G: GraphicsBackend, L: LayoutEngine> Runner<G, L> {
    /// Prepares `root` to run with `config`; nothing is shown until the platform resumes.
    pub fn new(root: impl View, engine: L, config: AppConfig) -> Result<Self, RunError> {
        Ok(Self {
            windows: vec![WindowRunner::new(root, engine, config)?],
            epoch: Instant::now(),
            failure: None,
        })
    }

    /// Adds another window showing `root`, opened with `config` alongside the first.
    pub fn with_window(
        mut self,
        root: impl View,
        engine: L,
        config: AppConfig,
    ) -> Result<Self, RunError> {
        self.windows.push(WindowRunner::new(root, engine, config)?);
        Ok(self)
    }

    /// The error that stopped the app, if any.
    pub fn take_failure(&mut self) -> Option<RunError> {
        self.failure.take()
    }

    fn fail<P: Platform>(&mut self, platform: &mut P, error: RunError) {
        tracing::error!("{error}");
        self.failure = Some(error);
        platform.exit();
    }

    fn position(&self, id: WindowId) -> Option<usize> {
        self.windows
            .iter()
            .position(|window| window.window() == Some(id))
    }
}

impl<P, G, L> ApplicationHandler<P> for Runner<G, L>
where
    P: Platform,
    G: GraphicsBackend + CreateRenderer<P::Surface>,
    L: LayoutEngine,
{
    fn resumed(&mut self, platform: &mut P) {
        let epoch = self.epoch;
        let opened = self
            .windows
            .iter_mut()
            .try_for_each(|window| window.open(platform, epoch));
        if let Err(error) = opened {
            self.fail(platform, error);
        }
    }

    fn window_event(&mut self, platform: &mut P, window_id: WindowId, event: WindowEvent) {
        let Some(index) = self.position(window_id) else {
            return;
        };
        if matches!(event, WindowEvent::CloseRequested) {
            self.windows.remove(index);
            if self.windows.is_empty() {
                platform.exit();
            }
            return;
        }
        self.windows[index].window_event(platform, &event);
    }

    fn agent_access(&mut self, platform: &mut P, decision: windowing::AgentAccess) {
        for window in &mut self.windows {
            window.agent_access(&decision, platform.waker());
        }
    }

    fn proxy_wake_up(&mut self, platform: &mut P) {
        let epoch = self.epoch;
        let pushed = self.windows.iter_mut().try_for_each(|window| {
            window.poll();
            window.push_frame(platform, epoch)
        });
        if let Err(error) = pushed {
            self.fail(platform, error);
        }
    }

    fn about_to_wait(&mut self, platform: &mut P) {
        let epoch = self.epoch;
        let mut deadline: Option<Instant> = None;
        for window in &mut self.windows {
            match window.before_wait(platform, epoch) {
                Ok(wake) => deadline = deadline.into_iter().chain(wake).min(),
                Err(error) => {
                    tracing::error!("{error}");
                    self.failure = Some(error);
                    return platform.exit();
                }
            }
        }
        platform.set_control_flow(match deadline {
            Some(deadline) => ControlFlow::WaitUntil(deadline),
            None => ControlFlow::Wait,
        });
    }
}
