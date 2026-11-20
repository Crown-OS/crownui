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
    keep_alive: bool,
}

impl<G: GraphicsBackend, L: LayoutEngine> Runner<G, L> {
    /// Prepares `root` to run with `config`; nothing is shown until the platform resumes.
    pub fn new(root: impl View, engine: L, config: AppConfig) -> Result<Self, RunError> {
        let mut runner = Self::empty(false);
        runner
            .windows
            .push(WindowRunner::new(root, engine, config)?);
        Ok(runner)
    }

    /// A runner with no windows; open them with [`Runner::open_window`] once the platform runs.
    /// With `keep_alive` the app keeps running after its last window closes.
    pub fn empty(keep_alive: bool) -> Self {
        Self {
            windows: Vec::new(),
            epoch: Instant::now(),
            failure: None,
            keep_alive,
        }
    }

    /// Opens a window showing `root` right away, while the platform runs.
    pub fn open_window<P>(
        &mut self,
        platform: &mut P,
        root: impl View,
        engine: L,
        config: AppConfig,
    ) -> Result<WindowId, RunError>
    where
        P: Platform,
        G: CreateRenderer<P::Surface>,
    {
        let mut window = WindowRunner::new(root, engine, config)?;
        match window.open(platform, self.epoch) {
            Ok(id) => {
                self.windows.push(window);
                Ok(id)
            }
            Err(error) => {
                if let Some(id) = window.window() {
                    platform.close_window(id);
                }
                Err(error)
            }
        }
    }

    /// Closes the window `id` without asking its UI; an unknown id does nothing.
    pub fn close_window<P: Platform>(&mut self, platform: &mut P, id: WindowId) {
        if let Some(index) = self.position(id) {
            self.windows.remove(index);
            platform.close_window(id);
        }
    }

    /// Whether the window `id` is still open.
    pub fn has_window(&self, id: WindowId) -> bool {
        self.position(id).is_some()
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

    /// Closes the windows whose UI agreed to close, exiting once the last one is gone unless the
    /// runner keeps the app alive.
    fn close_approved_windows<P: Platform>(&mut self, platform: &mut P) {
        let open = self.windows.len();
        self.windows.retain_mut(|window| {
            let closing = window.take_close_request();
            if let Some(id) = window.window().filter(|_| closing) {
                platform.close_window(id);
            }
            !closing
        });
        if !self.keep_alive && self.windows.len() < open && self.windows.is_empty() {
            platform.exit();
        }
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
            .try_for_each(|window| window.open(platform, epoch).map(drop));
        if let Err(error) = opened {
            self.fail(platform, error);
        }
    }

    fn window_event(&mut self, platform: &mut P, window_id: WindowId, event: WindowEvent) {
        let Some(index) = self.position(window_id) else {
            return;
        };
        self.windows[index].window_event(platform, &event);
        self.close_approved_windows(platform);
    }

    fn platform_event(&mut self, platform: &mut P, _event: P::Event) {
        for window in &mut self.windows {
            window.sync_capabilities(platform);
        }
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
        self.close_approved_windows(platform);
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
        self.close_approved_windows(platform);
        platform.set_control_flow(match deadline {
            Some(deadline) => ControlFlow::WaitUntil(deadline),
            None => ControlFlow::Wait,
        });
    }
}
