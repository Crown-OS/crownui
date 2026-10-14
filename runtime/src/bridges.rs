use view::Ui;
use windowing::Waker;

use crate::config::AppConfig;

/// Optional connections to the outside world: async tasks, MCP agents, screen readers.
#[derive(Default)]
pub(crate) struct Bridges {
    #[cfg(feature = "tokio")]
    tasks: Option<crossbeam_channel::Receiver<crate::executor::TaskOutput>>,
    #[cfg(feature = "mcp")]
    mcp: Option<crate::mcp_bridge::McpBridge>,
    #[cfg(feature = "accessibility")]
    accessibility: Option<crate::accessibility_bridge::AccessibilityBridge>,
}

impl Bridges {
    pub(crate) fn start(&mut self, config: &AppConfig, ui: &mut Ui, waker: &dyn Fn() -> Waker) {
        #[cfg(feature = "tokio")]
        {
            match crate::executor::TokioExecutor::start(waker()) {
                Ok((executor, outputs)) => {
                    ui.set_task_executor(Box::new(executor));
                    self.tasks = Some(outputs);
                }
                Err(error) => tracing::error!("CUI0407: task executor unavailable: {error}"),
            }
        }
        #[cfg(feature = "accessibility")]
        if config.accessibility {
            self.accessibility = Some(crate::accessibility_bridge::AccessibilityBridge::start(
                &config.title,
                ui,
            ));
        }
        let _ = (config, ui, waker);
    }

    /// Starts the MCP server (after the platform granted agent access, or when it has no gate).
    #[cfg(feature = "mcp")]
    pub(crate) fn start_mcp(&mut self, title: &str, waker: Waker) {
        if self.mcp.is_none() {
            self.mcp = crate::mcp_bridge::McpBridge::start(title, waker);
        }
    }

    /// Without the `mcp` feature there is no server to start.
    #[cfg(not(feature = "mcp"))]
    pub(crate) fn start_mcp(&mut self, _title: &str, _waker: Waker) {}

    /// Stops the MCP server and removes its socket.
    pub(crate) fn stop_mcp(&mut self) {
        #[cfg(feature = "mcp")]
        {
            self.mcp = None;
        }
    }

    pub(crate) fn poll(&mut self, ui: &mut Ui) {
        #[cfg(feature = "tokio")]
        if let Some(outputs) = &self.tasks {
            for (id, output) in outputs.try_iter() {
                ui.complete_task(id, output);
            }
        }
        #[cfg(feature = "mcp")]
        if let Some(mcp) = &mut self.mcp {
            mcp.poll(ui);
        }
        #[cfg(feature = "accessibility")]
        if let Some(accessibility) = &mut self.accessibility {
            accessibility.poll(ui);
        }
        let _ = ui;
    }

    pub(crate) fn after_frame(&mut self, ui: &mut Ui) {
        #[cfg(feature = "mcp")]
        if let Some(mcp) = &mut self.mcp {
            mcp.after_frame(ui);
        }
        #[cfg(feature = "accessibility")]
        if let Some(accessibility) = &mut self.accessibility {
            accessibility.after_frame(ui);
        }
        let _ = ui;
    }

    pub(crate) fn set_scale(&mut self, scale: f64) {
        #[cfg(feature = "accessibility")]
        if let Some(accessibility) = &mut self.accessibility {
            accessibility.set_scale(scale);
        }
        let _ = scale;
    }

    pub(crate) fn set_frames(&mut self, frames: Vec<inspect::FrameStats>) {
        #[cfg(feature = "mcp")]
        if let Some(mcp) = &mut self.mcp {
            mcp.set_frames(frames);
            return;
        }
        let _ = frames;
    }

    pub(crate) fn set_window_focused(&mut self, focused: bool) {
        #[cfg(feature = "accessibility")]
        if let Some(accessibility) = &mut self.accessibility {
            accessibility.set_window_focused(focused);
        }
        let _ = focused;
    }
}
