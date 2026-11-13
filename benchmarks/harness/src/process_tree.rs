use std::fs;

use crate::util::proc_fs::stat_fields;

const PARENT_FIELD: usize = 1;

/// `root` and every process descended from it, so multi-process runtimes (Electron's renderer
/// and GPU processes, WebKit's web process) are charged to the app.
pub fn descendants(root: u32) -> Vec<u32> {
    let parents: Vec<(u32, u32)> = fs::read_dir("/proc")
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|entry| entry.file_name().to_str()?.parse::<u32>().ok())
        .filter_map(|pid| Some((pid, stat_fields(pid)?.get(PARENT_FIELD)?.parse().ok()?)))
        .collect();
    let mut tree = vec![root];
    let mut next = 0;
    while let Some(&parent) = tree.get(next) {
        tree.extend(
            parents
                .iter()
                .filter(|(_, candidate)| *candidate == parent)
                .map(|(pid, _)| *pid),
        );
        next += 1;
    }
    tree
}
