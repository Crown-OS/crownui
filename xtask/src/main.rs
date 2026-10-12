//! `cargo xtask <command> [--json]` — see AGENTS.md.

mod command;
mod report;
mod util;
mod workspace;

use anyhow::{Result, bail};

fn main() -> Result<()> {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let json = arguments.iter().any(|argument| argument == "--json");
    let positional: Vec<&str> = arguments
        .iter()
        .map(String::as_str)
        .filter(|argument| !argument.starts_with("--"))
        .collect();
    let flags: Vec<&str> = arguments
        .iter()
        .map(String::as_str)
        .filter(|argument| argument.starts_with("--"))
        .collect();
    let report = match positional.first().copied() {
        Some("check") => command::check::run(&positional[1..])?,
        Some("layers") => command::layers::run()?,
        Some("api") => command::api::run(flags.contains(&"--check"))?,
        Some("snapshot") => command::snapshot::run()?,
        Some("shaders") => command::shaders::run(flags.contains(&"--check"))?,
        Some("bundle") => command::bundle::run()?,
        Some("preview") => {
            let rest: Vec<String> = arguments
                .iter()
                .skip(1)
                .filter(|argument| *argument != "--json")
                .cloned()
                .collect();
            command::preview::run(&rest)?
        }
        Some(other) => bail!(
            "unknown command {other:?}; expected check, layers, api, snapshot, shaders, bundle or preview"
        ),
        None => bail!(
            "usage: cargo xtask <check [crate..] | layers | api [--check] | snapshot | shaders [--check] | bundle | preview [--list | <name> [--png <path>]]> [--json]"
        ),
    };
    report.print(json);
    if !report.ok {
        std::process::exit(1);
    }
    Ok(())
}
