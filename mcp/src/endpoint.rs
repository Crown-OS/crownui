use std::path::{Path, PathBuf};

use crate::util::uid::current_uid;

const SOCKET_EXTENSION: &str = "sock";

/// Where a running app's MCP socket lives: `<socket_dir>/<app_name>-<pid>.sock`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppEndpoint {
    /// App name, restricted to `[A-Za-z0-9._-]`.
    pub app_name: String,
    /// Process id of the app.
    pub pid: u32,
    /// Socket path.
    pub path: PathBuf,
}

impl AppEndpoint {
    /// The endpoint of `app_name` running as `pid` in `socket_dir`; other characters
    /// in the name become `_`.
    pub fn new(socket_dir: &Path, app_name: &str, pid: u32) -> Self {
        let app_name: String = app_name
            .chars()
            .map(|character| match character {
                'a'..='z' | 'A'..='Z' | '0'..='9' | '.' | '_' | '-' => character,
                _ => '_',
            })
            .collect();
        let path = socket_dir.join(format!("{app_name}-{pid}.{SOCKET_EXTENSION}"));
        Self {
            app_name,
            pid,
            path,
        }
    }

    /// Parses a socket path following the naming scheme.
    pub fn from_path(path: impl Into<PathBuf>) -> Option<Self> {
        let path = path.into();
        let stem = path
            .file_name()?
            .to_str()?
            .strip_suffix(SOCKET_EXTENSION)?
            .strip_suffix('.')?;
        let (app_name, pid) = stem.rsplit_once('-')?;
        Some(Self {
            app_name: app_name.to_owned(),
            pid: pid.parse().ok()?,
            path,
        })
    }

    /// `<app_name>-<pid>`, unique among running apps.
    pub fn label(&self) -> String {
        format!("{}-{}", self.app_name, self.pid)
    }
}

/// `$XDG_RUNTIME_DIR/crownui`, or a per-user directory under the system temp dir.
pub fn default_socket_dir() -> PathBuf {
    std::env::var_os("XDG_RUNTIME_DIR").map_or_else(
        || std::env::temp_dir().join(format!("crownui-{}", current_uid())),
        |runtime_dir| PathBuf::from(runtime_dir).join("crownui"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn endpoints_round_trip_through_their_path() {
        let endpoint = AppEndpoint::new(Path::new("/run/crownui"), "my app/v2", 42);
        assert_eq!(endpoint.path, Path::new("/run/crownui/my_app_v2-42.sock"));
        assert_eq!(AppEndpoint::from_path(&endpoint.path), Some(endpoint));
        assert_eq!(AppEndpoint::from_path("/run/crownui/notes.txt"), None);
    }
}
