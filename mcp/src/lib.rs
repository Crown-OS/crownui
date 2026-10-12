#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

mod act;
mod config;
mod endpoint;
mod error;
mod gate;
mod handle;
mod handshake;
mod rate_limiter;
mod request;
mod server;
mod tool_spec;
mod util;

pub use act::{ActError, validate_action};
pub use config::{DEFAULT_ACTIONS_PER_SECOND, McpConfig};
pub use endpoint::{AppEndpoint, default_socket_dir};
pub use error::McpError;
pub use gate::{AccessGate, PeerInfo, SameUserGate, TokenGate};
pub use handle::McpHandle;
pub use handshake::connect;
pub use request::{ActOutcome, McpRequest, Responder, Screenshot, TreeSnapshot};
pub use server::McpServer;
pub use tool_spec::ToolSpec;
