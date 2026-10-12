use std::{fs, path::Path};

use anyhow::{Context, Result};
use toml::{Table, Value};

const TEMPLATE: &str = "xtask/bundle/manifest.toml";

/// The published manifest: `xtask/bundle/manifest.toml` at the workspace's version.
pub(crate) fn render(root: &Path) -> Result<String> {
    let workspace: Table = toml::from_str(&fs::read_to_string(root.join("Cargo.toml"))?)?;
    let version = workspace
        .get("workspace")
        .and_then(|workspace| workspace.get("package"))
        .and_then(|package| package.get("version"))
        .cloned()
        .context("Cargo.toml has no [workspace.package] version")?;
    let mut manifest: Table = toml::from_str(&fs::read_to_string(root.join(TEMPLATE))?)?;
    manifest
        .get_mut("package")
        .and_then(Value::as_table_mut)
        .context("the bundle manifest has no [package]")?
        .insert("version".into(), version);
    Ok(toml::to_string(&manifest)?)
}
