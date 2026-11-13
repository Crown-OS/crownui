use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, Result};
use serde::Serialize;

use crate::{frame_rate::FrameRate, framework::Framework, paths::Paths};

/// Disk an app needs: its own files, and the shared libraries its executable loads (which other
/// apps on the system may share).
#[derive(Debug, Clone, Copy, Serialize)]
pub struct Footprint {
    pub own_bytes: u64,
    pub library_bytes: u64,
}

impl Footprint {
    pub fn measure(framework: Framework, paths: &Paths) -> Result<Self> {
        let own = framework.own_files(paths)?;
        let libraries = shared_libraries(&framework.executable(paths, FrameRate::Display)?)?;
        Ok(Self {
            own_bytes: own.iter().map(|path| size_on_disk(path)).sum(),
            library_bytes: libraries
                .iter()
                .filter(|library| !own.iter().any(|root| library.starts_with(root)))
                .map(|library| size_on_disk(library))
                .sum(),
        })
    }
}

fn shared_libraries(executable: &Path) -> Result<BTreeSet<PathBuf>> {
    let output = Command::new("ldd")
        .arg(executable)
        .output()
        .with_context(|| format!("running ldd on {}", executable.display()))?;
    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| {
            let path = line.split("=>").last()?.split_whitespace().next()?;
            path.starts_with('/').then(|| fs::canonicalize(path).ok())?
        })
        .collect())
}

fn size_on_disk(path: &Path) -> u64 {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.is_dir() => fs::read_dir(path)
            .into_iter()
            .flatten()
            .flatten()
            .map(|entry| size_on_disk(&entry.path()))
            .sum(),
        Ok(metadata) if metadata.is_file() => metadata.len(),
        _ => 0,
    }
}
