use std::fs;

use anyhow::{Context, Result, bail};

use crate::{
    report::{Report, Step},
    workspace,
};

const CARGO_NOISE: [&str; 7] = [
    "Compiling",
    "Finished",
    "Running",
    "Blocking",
    "Downloading",
    "Downloaded",
    "Updating",
];

/// Runs the frame benchmarks (`runtime/benches/frame`). `--save-baseline NAME` keeps the results
/// under `target/bench/NAME.json`; `--baseline NAME` prints how each scenario moved against them.
/// `--filter`, `--families`, `--sizes` and `--iterations` pass through to the bench.
pub(crate) fn run(arguments: &[String]) -> Result<Report> {
    let bench_dir = workspace::root()?.join("target").join("bench");
    fs::create_dir_all(&bench_dir)?;
    let latest = bench_dir.join("latest.json");
    let mut forwarded = vec!["--json".to_owned(), latest.display().to_string()];
    let mut save = None;
    let mut arguments = arguments.iter();
    while let Some(argument) = arguments.next() {
        let value = arguments
            .next()
            .with_context(|| format!("{argument} needs a value"))?;
        match argument.as_str() {
            "--save-baseline" => save = Some(value.clone()),
            "--baseline" => {
                forwarded.push("--baseline".to_owned());
                forwarded.push(
                    bench_dir
                        .join(format!("{value}.json"))
                        .display()
                        .to_string(),
                );
            }
            "--filter" | "--families" | "--sizes" | "--iterations" => {
                forwarded.push(argument.clone());
                forwarded.push(value.clone());
            }
            other => bail!(
                "unknown bench argument {other:?}; expected --save-baseline, --baseline, --filter, --families, --sizes or --iterations"
            ),
        }
    }
    let mut cargo = vec!["bench", "-p", "runtime", "--bench", "frame", "--"];
    cargo.extend(forwarded.iter().map(String::as_str));
    let (ok, output) = workspace::run("cargo", &cargo)?;
    let mut steps = vec![Step {
        name: "cargo bench -p runtime --bench frame".into(),
        ok,
        detail: without_cargo_noise(&output),
    }];
    if let (true, Some(name)) = (ok, save) {
        let target = bench_dir.join(format!("{name}.json"));
        fs::copy(&latest, &target)
            .with_context(|| format!("saving the baseline to {}", target.display()))?;
        steps.push(Step {
            name: format!("saved baseline {}", target.display()),
            ok: true,
            detail: String::new(),
        });
    }
    Ok(Report::new("bench", steps))
}

fn without_cargo_noise(output: &str) -> String {
    output
        .lines()
        .filter(|line| {
            let trimmed = line.trim_start();
            !trimmed.is_empty() && !CARGO_NOISE.iter().any(|noise| trimmed.starts_with(noise))
        })
        .collect::<Vec<_>>()
        .join("\n")
}
