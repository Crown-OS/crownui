/// Whether the platform currently shows a window.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum Visibility {
    /// Drawn on screen.
    #[default]
    Visible,
    /// On screen but fully covered (e.g. by a fullscreen window).
    Occluded,
    /// Not drawn anywhere (another workspace, minimized, locked session).
    Hidden,
}

impl Visibility {
    /// Whether rendering frames for the window is pointless right now.
    pub fn is_obscured(self) -> bool {
        self != Self::Visible
    }
}

/// The platform's answer to [`Platform::request_agent_access`](crate::Platform::request_agent_access).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentAccess {
    /// The user allowed AI agents to read and drive this app; the token identifies the grant.
    Granted(String),
    /// The request was refused.
    Denied,
    /// An earlier grant ended; stop exposing the UI.
    Revoked,
}
