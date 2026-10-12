mod action_arguments;
mod app_server;
mod builtin_tools;
mod listener;
mod resources;
mod tool_calls;
mod tree_json;
mod ui_bridge;

use std::{
    fs::{self, DirBuilder, Permissions},
    io,
    os::unix::fs::{DirBuilderExt, MetadataExt, PermissionsExt},
    path::Path,
    sync::Arc,
};

use tokio::{
    net::UnixListener,
    sync::{mpsc, oneshot, watch},
};

use crate::{
    AppEndpoint, McpConfig, McpError, McpHandle, rate_limiter::RateLimiter, util::uid::current_uid,
};
use app_server::AppServer;
use ui_bridge::UiBridge;

const REQUEST_QUEUE_CAPACITY: usize = 64;
const SOCKET_DIR_MODE: u32 = 0o700;
const SOCKET_MODE: u32 = 0o600;

/// The in-app MCP server, running on its own thread with a current-thread tokio runtime.
#[derive(Debug)]
pub struct McpServer;

impl McpServer {
    /// Binds `<socket_dir>/<app_name>-<pid>.sock` and starts serving; the UI thread
    /// keeps the returned handle and answers its requests.
    pub fn start(config: McpConfig) -> Result<McpHandle, McpError> {
        let McpConfig {
            app_name,
            socket_dir,
            gate,
            actions_per_second,
        } = config;
        prepare_socket_dir(&socket_dir)?;
        let endpoint = AppEndpoint::new(&socket_dir, &app_name, std::process::id());
        let std_listener = bind_socket(&endpoint.path)?;

        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        let listener = {
            let _context = runtime.enter();
            UnixListener::from_std(std_listener)?
        };
        let (request_sender, requests) = mpsc::channel(REQUEST_QUEUE_CAPACITY);
        let (tools, tool_updates) = watch::channel(Vec::new());
        let (wakeup, wakeup_updates) = watch::channel(None);
        let (shutdown, shutdown_signal) = oneshot::channel();
        let server = AppServer::new(
            app_name.into(),
            UiBridge::new(request_sender, wakeup_updates),
            tool_updates,
            RateLimiter::per_second(actions_per_second).map(Arc::new),
        );
        let thread = std::thread::Builder::new()
            .name(format!("mcp-{}", endpoint.app_name))
            .spawn(move || {
                runtime.block_on(listener::serve(listener, gate, server, shutdown_signal))
            })?;

        Ok(McpHandle {
            requests,
            tools,
            wakeup,
            shutdown: Some(shutdown),
            thread: Some(thread),
            endpoint,
        })
    }
}

fn prepare_socket_dir(socket_dir: &Path) -> Result<(), McpError> {
    let failed = |source: io::Error| McpError::SocketDir {
        path: socket_dir.to_owned(),
        source,
    };
    DirBuilder::new()
        .recursive(true)
        .mode(SOCKET_DIR_MODE)
        .create(socket_dir)
        .map_err(failed)?;
    let metadata = fs::metadata(socket_dir).map_err(failed)?;
    if metadata.uid() != current_uid() {
        return Err(McpError::ForeignSocketDir {
            path: socket_dir.to_owned(),
        });
    }
    if metadata.mode() & 0o777 != SOCKET_DIR_MODE {
        fs::set_permissions(socket_dir, Permissions::from_mode(SOCKET_DIR_MODE)).map_err(failed)?;
    }
    Ok(())
}

fn bind_socket(path: &Path) -> Result<std::os::unix::net::UnixListener, McpError> {
    let failed = |source: io::Error| McpError::Bind {
        path: path.to_owned(),
        source,
    };
    match fs::remove_file(path) {
        Err(error) if error.kind() != io::ErrorKind::NotFound => return Err(failed(error)),
        _ => {}
    }
    let listener = std::os::unix::net::UnixListener::bind(path).map_err(failed)?;
    fs::set_permissions(path, Permissions::from_mode(SOCKET_MODE)).map_err(failed)?;
    listener.set_nonblocking(true).map_err(failed)?;
    Ok(listener)
}
