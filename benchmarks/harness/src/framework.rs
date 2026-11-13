use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    str::FromStr,
};

use anyhow::{Context, Result, anyhow, bail};
use serde::Serialize;

use crate::{frame_rate::FrameRate, paths::Paths};

const ELECTRON_LAUNCHER: &str = "/usr/bin/electron";
/// Where GPUI is rebuilt with `apps/gpui/uncapped.patch`, apart from the unpatched build.
const UNCAPPED_GPUI_DIR: &str = "gpui-uncapped";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Framework {
    CrownUi,
    Gpui,
    Qt,
    Electron,
    Tauri,
}

impl Framework {
    pub const ALL: [Self; 5] = [
        Self::CrownUi,
        Self::Gpui,
        Self::Qt,
        Self::Electron,
        Self::Tauri,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::CrownUi => "crownui",
            Self::Gpui => "gpui",
            Self::Qt => "qt",
            Self::Electron => "electron",
            Self::Tauri => "tauri",
        }
    }

    pub fn build(self, paths: &Paths, frame_rate: FrameRate) -> Result<()> {
        match self {
            Self::Gpui if frame_rate.is_uncapped() => build_uncapped_gpui(paths),
            Self::CrownUi | Self::Gpui | Self::Tauri => run_checked(
                Command::new("cargo")
                    .args(["build", "--release", "--manifest-path"])
                    .arg(paths.apps.join(self.name()).join("Cargo.toml"))
                    .env("CARGO_TARGET_DIR", &paths.target),
            ),
            Self::Qt => {
                let build_dir = qt_build_dir(paths);
                run_checked(
                    Command::new("cmake")
                        .arg("-S")
                        .arg(paths.apps.join("qt"))
                        .arg("-B")
                        .arg(&build_dir)
                        .arg("-DCMAKE_BUILD_TYPE=Release"),
                )?;
                run_checked(
                    Command::new("cmake")
                        .arg("--build")
                        .arg(&build_dir)
                        .arg("--parallel"),
                )
            }
            Self::Electron => electron_binary().map(drop),
        }
    }

    pub fn executable(self, paths: &Paths, frame_rate: FrameRate) -> Result<PathBuf> {
        match self {
            Self::Gpui if frame_rate.is_uncapped() => Ok(paths
                .target
                .join(UNCAPPED_GPUI_DIR)
                .join("target/release/gpui-bench")),
            Self::CrownUi | Self::Gpui | Self::Tauri => Ok(paths
                .target
                .join("release")
                .join(format!("{}-bench", self.name()))),
            Self::Qt => Ok(qt_build_dir(paths).join("qt-bench")),
            Self::Electron => electron_binary(),
        }
    }

    pub fn command(self, paths: &Paths, frame_rate: FrameRate) -> Result<Command> {
        let mut command = Command::new(self.executable(paths, frame_rate)?);
        if self == Self::Electron {
            command
                .arg("--ozone-platform=wayland")
                .arg(paths.apps.join("electron"));
        }
        Ok(command)
    }

    /// Files the app needs besides shared system libraries: its executable, or for Electron
    /// the runtime directory and the app's scripts.
    pub fn own_files(self, paths: &Paths) -> Result<Vec<PathBuf>> {
        let executable = self.executable(paths, FrameRate::Display)?;
        Ok(match self {
            Self::Electron => vec![
                executable
                    .parent()
                    .context("the Electron binary has a directory")?
                    .to_path_buf(),
                paths.apps.join("electron"),
                paths.apps.join("web"),
            ],
            _ => vec![executable],
        })
    }
}

impl FromStr for Framework {
    type Err = anyhow::Error;

    fn from_str(name: &str) -> Result<Self> {
        Self::ALL
            .into_iter()
            .find(|framework| framework.name() == name)
            .ok_or_else(|| {
                anyhow!("unknown framework {name:?}; expected crownui, gpui, qt, electron or tauri")
            })
    }
}

/// Builds the GPUI app against a copy of the gpui crate it locks, patched to draw the next frame
/// as soon as the last one is submitted instead of on the compositor's frame callback. The app and
/// its shared sources are copied along, so the app's own lockfile stays untouched.
fn build_uncapped_gpui(paths: &Paths) -> Result<()> {
    let root = paths.target.join(UNCAPPED_GPUI_DIR);
    let patched = root.join("gpui");
    let apps = root.join("apps");
    for stale in [&patched, &apps] {
        if stale.exists() {
            fs::remove_dir_all(stale).with_context(|| format!("removing {}", stale.display()))?;
        }
    }
    fs::create_dir_all(&apps)?;
    run_checked(
        Command::new("cp")
            .arg("-r")
            .arg(gpui_source(paths)?)
            .arg(&patched),
    )?;
    run_checked(
        Command::new("patch")
            .args(["-p1", "--quiet", "-d"])
            .arg(&patched)
            .arg("-i")
            .arg(paths.apps.join("gpui/uncapped.patch")),
    )?;
    for app in ["gpui", "shared"] {
        run_checked(
            Command::new("cp")
                .arg("-r")
                .arg(paths.apps.join(app))
                .arg(apps.join(app)),
        )?;
    }
    run_checked(
        Command::new("cargo")
            .args(["build", "--release", "--manifest-path"])
            .arg(apps.join("gpui/Cargo.toml"))
            .arg("--config")
            .arg(format!(
                "patch.crates-io.gpui.path={:?}",
                patched.display().to_string()
            ))
            .env("CARGO_TARGET_DIR", root.join("target")),
    )
}

/// The registry sources of the gpui crate the GPUI app's lockfile picks.
fn gpui_source(paths: &Paths) -> Result<PathBuf> {
    let output = Command::new("cargo")
        .args(["metadata", "--format-version", "1", "--manifest-path"])
        .arg(paths.apps.join("gpui/Cargo.toml"))
        .output()
        .context("running cargo metadata")?;
    if !output.status.success() {
        bail!(
            "cargo metadata failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let metadata: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    metadata["packages"]
        .as_array()
        .into_iter()
        .flatten()
        .find(|package| package["name"] == "gpui")
        .and_then(|package| package["manifest_path"].as_str())
        .and_then(|manifest| Path::new(manifest).parent())
        .map(Path::to_path_buf)
        .context("the GPUI app does not depend on gpui")
}

fn qt_build_dir(paths: &Paths) -> PathBuf {
    paths.target.join("qt")
}

/// The real Electron binary behind the distribution's `/usr/bin/electron` wrapper script, so
/// startup is not charged for a bash wrapper.
fn electron_binary() -> Result<PathBuf> {
    let version = fs::read_link(ELECTRON_LAUNCHER)
        .with_context(|| format!("{ELECTRON_LAUNCHER} is not installed"))?;
    let binary = Path::new("/usr/lib").join(version).join("electron");
    if !binary.exists() {
        bail!("no Electron binary at {}", binary.display());
    }
    Ok(binary)
}

fn run_checked(command: &mut Command) -> Result<()> {
    let status = command
        .status()
        .with_context(|| format!("running {command:?}"))?;
    if !status.success() {
        bail!("{command:?} failed with {status}");
    }
    Ok(())
}
