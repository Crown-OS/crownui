use crate::util::{constant_time::constant_time_eq, uid::current_uid};

/// Credentials of a connecting process, from `SO_PEERCRED`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PeerInfo {
    /// User id of the peer.
    pub uid: u32,
    /// Process id of the peer, when the kernel reports it.
    pub pid: Option<i32>,
}

/// Decides whether a connecting peer may use the app's MCP server.
pub trait AccessGate {
    /// Whether `peer`, presenting `token` (from the `TOKEN` handshake line), may connect.
    fn authorize(&mut self, peer: &PeerInfo, token: Option<&str>) -> bool;
}

/// Admits any process of the user running the app; tokens are ignored.
#[derive(Debug, Clone, Copy, Default)]
pub struct SameUserGate;

impl AccessGate for SameUserGate {
    fn authorize(&mut self, peer: &PeerInfo, _token: Option<&str>) -> bool {
        peer.uid == current_uid()
    }
}

/// Admits processes of the same user that present the expected token.
#[derive(Debug, Clone)]
pub struct TokenGate {
    /// The token clients must present.
    pub expected: String,
}

impl TokenGate {
    /// A gate expecting `expected`.
    pub fn new(expected: impl Into<String>) -> Self {
        Self {
            expected: expected.into(),
        }
    }
}

impl AccessGate for TokenGate {
    fn authorize(&mut self, peer: &PeerInfo, token: Option<&str>) -> bool {
        peer.uid == current_uid()
            && token
                .is_some_and(|token| constant_time_eq(token.as_bytes(), self.expected.as_bytes()))
    }
}
