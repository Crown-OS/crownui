use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, Result};
use serde::Deserialize;

/// A workspace member and its workspace-internal dependencies.
#[derive(Debug, Clone)]
pub(crate) struct Member {
    pub(crate) name: String,
    pub(crate) directory: PathBuf,
    pub(crate) dependencies: Vec<String>,
}

#[derive(Deserialize)]
struct Metadata {
    packages: Vec<Package>,
    workspace_root: PathBuf,
}

#[derive(Deserialize)]
struct Package {
    name: String,
    manifest_path: PathBuf,
    dependencies: Vec<Dependency>,
}

#[derive(Deserialize)]
struct Dependency {
    name: String,
    path: Option<PathBuf>,
    kind: Option<String>,
}

pub(crate) fn root() -> Result<PathBuf> {
    Ok(metadata()?.workspace_root)
}

fn metadata() -> Result<Metadata> {
    let output = Command::new(env!("CARGO"))
        .args(["metadata", "--format-version", "1", "--no-deps"])
        .output()
        .context("running cargo metadata")?;
    serde_json::from_slice(&output.stdout).context("parsing cargo metadata")
}

/// Every member with its non-dev path dependencies on other members.
pub(crate) fn members() -> Result<Vec<Member>> {
    let metadata = metadata()?;
    let names: Vec<String> = metadata.packages.iter().map(|package| package.name.clone()).collect();
    Ok(metadata
        .packages
        .into_iter()
        .map(|package| Member {
            directory: package
                .manifest_path
                .parent()
                .map(Path::to_path_buf)
                .unwrap_or_default(),
            dependencies: package
                .dependencies
                .into_iter()
                .filter(|dependency| dependency.path.is_some() && dependency.kind.as_deref() != Some("dev"))
                .map(|dependency| dependency.name)
                .filter(|name| names.contains(name))
                .collect(),
            name: package.name,
        })
        .collect())
}

/// Runs a command in the workspace root and returns (success, combined output).
pub(crate) fn run(program: &str, arguments: &[&str]) -> Result<(bool, String)> {
    let output = Command::new(program)
        .args(arguments)
        .current_dir(root()?)
        .env("CARGO_INCREMENTAL", "0")
        .output()
        .with_context(|| format!("running {program} {}", arguments.join(" ")))?;
    let mut text = String::from_utf8_lossy(&output.stdout).into_owned();
    text.push_str(&String::from_utf8_lossy(&output.stderr));
    Ok((output.status.success(), text))
}

/// Files changed relative to `HEAD` (or every tracked-looking file when there is no history).
pub(crate) fn changed_files() -> Result<Vec<PathBuf>> {
    let (has_head, _) = run("git", &["rev-parse", "--verify", "HEAD"])?;
    let (_, listing) = if has_head {
        run("git", &["diff", "--name-only", "HEAD"])?
    } else {
        run("git", &["ls-files", "--others", "--cached", "--exclude-standard"])?
    };
    let (_, untracked) = run("git", &["ls-files", "--others", "--exclude-standard"])?;
    let root = root()?;
    let mut files: Vec<PathBuf> = listing
        .lines()
        .chain(untracked.lines())
        .filter(|line| !line.trim().is_empty())
        .map(|line| root.join(line.trim()))
        .collect();
    files.sort();
    files.dedup();
    Ok(files)
}

/// Maps each member to the changed files inside it.
pub(crate) fn changed_members(members: &[Member], files: &[PathBuf]) -> BTreeMap<String, Vec<PathBuf>> {
    let mut changed: BTreeMap<String, Vec<PathBuf>> = BTreeMap::new();
    for file in files {
        let owner = members
            .iter()
            .filter(|member| file.starts_with(&member.directory))
            .max_by_key(|member| member.directory.as_os_str().len());
        if let Some(member) = owner {
            changed.entry(member.name.clone()).or_default().push(file.clone());
        }
    }
    changed
}
