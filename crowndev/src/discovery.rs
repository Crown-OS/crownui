use std::{fs, path::Path};

use anyhow::{Result, bail};
use mcp::AppEndpoint;

/// Live app sockets in `socket_dir`, sorted by name then pid.
pub(crate) fn discover_apps(socket_dir: &Path) -> Vec<AppEndpoint> {
    let Ok(entries) = fs::read_dir(socket_dir) else {
        return Vec::new();
    };
    let mut running: Vec<AppEndpoint> = entries
        .filter_map(|entry| AppEndpoint::from_path(entry.ok()?.path()))
        .filter(|endpoint| is_process_alive(endpoint.pid))
        .collect();
    running.sort_by(|left, right| {
        left.app_name
            .cmp(&right.app_name)
            .then(left.pid.cmp(&right.pid))
    });
    running
}

/// The app called `wanted` (name or name-pid), or the only running app when `wanted` is `None`.
pub(crate) fn select_app(mut apps: Vec<AppEndpoint>, wanted: Option<&str>) -> Result<AppEndpoint> {
    let running = apps
        .iter()
        .map(AppEndpoint::label)
        .collect::<Vec<_>>()
        .join(", ");
    let Some(wanted) = wanted else {
        return match <[AppEndpoint; 1]>::try_from(apps) {
            Ok([only]) => Ok(only),
            Err(apps) if apps.is_empty() => bail!("no CrownUI app is running"),
            Err(_) => bail!("several apps run; pick one of [{running}]"),
        };
    };
    if let Some(index) = apps.iter().position(|app| app.label() == wanted) {
        return Ok(apps.swap_remove(index));
    }
    let mut named = apps.into_iter().filter(|app| app.app_name == wanted);
    match (named.next(), named.next()) {
        (Some(app), None) => Ok(app),
        (Some(_), Some(_)) => {
            bail!("several `{wanted}` apps run; pick one of [{running}] by name-pid")
        }
        (None, _) => bail!("no running app named `{wanted}`; running: [{running}]"),
    }
}

fn is_process_alive(pid: u32) -> bool {
    Path::new("/proc").join(pid.to_string()).exists()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn endpoint(name: &str, pid: u32) -> AppEndpoint {
        AppEndpoint::new(Path::new("/run/crownui"), name, pid)
    }

    #[test]
    fn apps_are_selected_by_name_label_or_uniqueness() -> Result<()> {
        let apps = vec![
            endpoint("notes", 1),
            endpoint("counter", 2),
            endpoint("counter", 3),
        ];
        assert_eq!(select_app(apps.clone(), Some("notes"))?.pid, 1);
        assert_eq!(select_app(apps.clone(), Some("counter-3"))?.pid, 3);
        assert!(select_app(apps.clone(), Some("counter")).is_err());
        assert!(select_app(apps.clone(), Some("calculator")).is_err());
        assert!(select_app(apps, None).is_err());
        assert_eq!(select_app(vec![endpoint("solo", 9)], None)?.pid, 9);
        assert!(select_app(Vec::new(), None).is_err());
        Ok(())
    }
}
