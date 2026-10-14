use anyhow::Result;

use crate::{
    report::{Report, Step},
    workspace,
};

/// Clippy and tests for the named crates (or every changed crate), rustfmt on changed files.
pub(crate) fn run(crates: &[&str]) -> Result<Report> {
    let members = workspace::members()?;
    let files = workspace::changed_files()?;
    let changed = workspace::changed_members(&members, &files);
    let targets: Vec<String> = if crates.is_empty() {
        changed.keys().cloned().collect()
    } else {
        crates.iter().map(|name| (*name).to_owned()).collect()
    };
    let mut steps = Vec::new();
    let rust_files: Vec<String> = files
        .iter()
        .filter(|file| file.extension().is_some_and(|extension| extension == "rs") && file.exists())
        .map(|file| file.display().to_string())
        .collect();
    if !rust_files.is_empty() {
        let mut arguments = vec!["--edition", "2024", "--check"];
        arguments.extend(rust_files.iter().map(String::as_str));
        let (ok, detail) = workspace::run("rustfmt", &arguments)?;
        steps.push(Step {
            name: format!("rustfmt --check ({} changed files)", rust_files.len()),
            ok,
            detail,
        });
    }
    for target in &targets {
        let (clippy_ok, clippy_output) = workspace::run(
            "cargo",
            &[
                "clippy",
                "-p",
                target,
                "--all-targets",
                "--",
                "-D",
                "warnings",
            ],
        )?;
        steps.push(Step {
            name: format!("clippy {target}"),
            ok: clippy_ok,
            detail: clippy_output,
        });
        let (test_ok, test_output) = workspace::run("cargo", &["test", "-p", target])?;
        let summary: String = test_output
            .lines()
            .filter(|line| {
                line.starts_with("test result")
                    || line.contains("FAILED")
                    || line.contains("panicked")
            })
            .collect::<Vec<_>>()
            .join("\n");
        steps.push(Step {
            name: format!("test {target}"),
            ok: test_ok,
            detail: if test_ok { summary } else { test_output },
        });
    }
    if targets.is_empty() {
        steps.push(Step {
            name: "no changed crates".into(),
            ok: true,
            detail: String::new(),
        });
    }
    Ok(Report::new("check", steps))
}
