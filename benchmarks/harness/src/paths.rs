use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

/// Where the suite's sources, build outputs and results live.
pub struct Paths {
    pub apps: PathBuf,
    pub target: PathBuf,
    pub results: PathBuf,
}

impl Paths {
    pub fn discover() -> Result<Self> {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .context("the harness lives inside benchmarks/")?
            .to_path_buf();
        Ok(Self {
            apps: root.join("apps"),
            target: root.join("target"),
            results: root.join("results"),
        })
    }

    pub fn spec(&self) -> PathBuf {
        self.apps.join("spec.toml")
    }
}
