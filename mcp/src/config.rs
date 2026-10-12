use std::path::PathBuf;

use crate::{AccessGate, SameUserGate, default_socket_dir};

/// Default token-bucket rate for `ui_act` and app tools.
pub const DEFAULT_ACTIONS_PER_SECOND: u32 = 20;

/// How to start an [`McpServer`](crate::McpServer).
pub struct McpConfig {
    /// Name agents see; also the socket file prefix.
    pub app_name: String,
    /// Directory holding the socket, created with mode 0700.
    pub socket_dir: PathBuf,
    /// Decides which peers may connect.
    pub gate: Box<dyn AccessGate + Send>,
    /// Sustained rate (and burst) of `ui_act` and app tool calls; `0` disables the limit.
    pub actions_per_second: u32,
}

impl McpConfig {
    /// A configuration with the default socket directory, [`SameUserGate`] and
    /// [`DEFAULT_ACTIONS_PER_SECOND`].
    pub fn new(app_name: impl Into<String>) -> Self {
        Self {
            app_name: app_name.into(),
            socket_dir: default_socket_dir(),
            gate: Box::new(SameUserGate),
            actions_per_second: DEFAULT_ACTIONS_PER_SECOND,
        }
    }

    /// Places the socket in `socket_dir`.
    pub fn socket_dir(mut self, socket_dir: impl Into<PathBuf>) -> Self {
        self.socket_dir = socket_dir.into();
        self
    }

    /// Replaces the access gate.
    pub fn gate(mut self, gate: impl AccessGate + Send + 'static) -> Self {
        self.gate = Box::new(gate);
        self
    }

    /// Sets the action rate limit; `0` disables it.
    pub fn actions_per_second(mut self, actions_per_second: u32) -> Self {
        self.actions_per_second = actions_per_second;
        self
    }
}
