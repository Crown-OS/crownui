use std::{fs, path::Path};

use mcp::AppEndpoint;

pub(crate) fn discover_apps(socket_dir: &Path) -> Vec<AppEndpoint> {
    let Ok(entries) = fs::read_dir(socket_dir) else {
        return Vec::new();
    };
    let (mut running, dead): (Vec<_>, Vec<_>) = entries
        .filter_map(|entry| AppEndpoint::from_path(entry.ok()?.path()))
        .partition(|endpoint| is_process_alive(endpoint.pid));
    for endpoint in dead {
        let _ = fs::remove_file(endpoint.path);
    }
    running.sort_by(|left, right| {
        left.app_name
            .cmp(&right.app_name)
            .then(left.pid.cmp(&right.pid))
    });
    running
}

pub(crate) fn find_app(mut apps: Vec<AppEndpoint>, wanted: &str) -> Result<AppEndpoint, String> {
    if let Some(index) = apps.iter().position(|app| app.label() == wanted) {
        return Ok(apps.swap_remove(index));
    }
    let running = apps
        .iter()
        .map(AppEndpoint::label)
        .collect::<Vec<_>>()
        .join(", ");
    let mut named = apps.into_iter().filter(|app| app.app_name == wanted);
    match (named.next(), named.next()) {
        (Some(app), None) => Ok(app),
        (Some(_), Some(_)) => Err(format!(
            "several `{wanted}` apps run; pick one of [{running}] by name-pid"
        )),
        (None, _) => Err(format!(
            "no running app named `{wanted}`; running: [{running}]"
        )),
    }
}

fn is_process_alive(pid: u32) -> bool {
    Path::new("/proc").join(pid.to_string()).exists()
}
