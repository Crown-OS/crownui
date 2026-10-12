use std::{path::Path, sync::Arc, thread::JoinHandle};

use tokio::sync::{mpsc, oneshot, watch};

use crate::{AppEndpoint, McpRequest, ToolSpec};

pub(crate) type Wakeup = Arc<dyn Fn() + Send + Sync>;

/// The UI thread's end of a running [`McpServer`](crate::McpServer); dropping it shuts
/// the server down and removes the socket.
pub struct McpHandle {
    pub(crate) requests: mpsc::Receiver<McpRequest>,
    pub(crate) tools: watch::Sender<Vec<ToolSpec>>,
    pub(crate) wakeup: watch::Sender<Option<Wakeup>>,
    pub(crate) shutdown: Option<oneshot::Sender<()>>,
    pub(crate) thread: Option<JoinHandle<()>>,
    pub(crate) endpoint: AppEndpoint,
}

impl McpHandle {
    /// Where agents connect.
    pub fn socket_path(&self) -> &Path {
        &self.endpoint.path
    }

    /// Drains queued requests without blocking.
    pub fn pending_requests(&mut self) -> impl Iterator<Item = McpRequest> + '_ {
        std::iter::from_fn(|| self.requests.try_recv().ok())
    }

    /// Publishes the app's tools, replacing the previous set; connected agents are notified.
    pub fn set_tools(&self, tools: Vec<ToolSpec>) {
        self.tools.send_replace(tools);
    }

    /// Registers a callback the server thread runs after queueing a request, so the UI
    /// event loop wakes and calls [`pending_requests`](Self::pending_requests).
    pub fn set_wakeup(&self, wakeup: impl Fn() + Send + Sync + 'static) {
        self.wakeup.send_replace(Some(Arc::new(wakeup)));
    }

    /// Stops the server, waits for its thread and removes the socket.
    pub fn shutdown(self) {}
}

impl Drop for McpHandle {
    fn drop(&mut self) {
        if let Some(shutdown) = self.shutdown.take() {
            let _ = shutdown.send(());
        }
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
        let _ = std::fs::remove_file(&self.endpoint.path);
    }
}
