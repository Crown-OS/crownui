use sctk::reexports::calloop::ping::Ping;

/// Wakes the event loop from any thread.
#[derive(Debug, Clone)]
pub struct EventLoopProxy {
    pub(crate) waker: Ping,
}

impl EventLoopProxy {
    /// Makes the loop call [`ApplicationHandler::proxy_wake_up`](crate::ApplicationHandler::proxy_wake_up).
    pub fn wake_up(&self) {
        self.waker.ping();
    }
}
