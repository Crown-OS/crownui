use std::process::{Command, Stdio};

use anyhow::{Context, Result};

use crate::{
    report::{Report, Step},
    workspace,
};

/// Benchmarks CrownUI against GPUI, Qt, Electron and Tauri with the harness in `benchmarks/`
/// (its own Cargo workspace, so the other toolkits never enter `Cargo.lock`). Arguments pass
/// through: `--frameworks`, `--scenarios`, `--runs`, `--duration`, `--settle`, `--no-build`.
/// Progress streams to stderr; the results table becomes the report.
pub(crate) fn run(arguments: &[String]) -> Result<Report> {
    let benchmarks = workspace::root()?.join("benchmarks");
    let output = Command::new(env!("CARGO"))
        .args(["run", "--quiet", "--release", "--manifest-path"])
        .arg(benchmarks.join("harness").join("Cargo.toml"))
        .arg("--")
        .args(arguments)
        .env("CARGO_TARGET_DIR", benchmarks.join("target"))
        .stderr(Stdio::inherit())
        .output()
        .context("running the comparison harness")?;
    Ok(Report::new(
        "compare",
        vec![Step {
            name: "benchmarks/harness".into(),
            ok: output.status.success(),
            detail: String::from_utf8_lossy(&output.stdout).trim().to_owned(),
        }],
    ))
}
