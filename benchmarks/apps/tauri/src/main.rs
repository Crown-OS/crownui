use std::env;

use anyhow::{Context, Result};
use tauri::{WebviewUrl, WebviewWindowBuilder};

const PREFIX: &str = "BENCH_";

/// The `BENCH_*` variables as the query string the shared web scenarios read.
fn scenario_query() -> String {
    env::vars()
        .filter_map(|(name, value)| {
            let key = name.strip_prefix(PREFIX)?.to_lowercase();
            Some(format!("{key}={value}"))
        })
        .collect::<Vec<_>>()
        .join("&")
}

fn dimension(name: &str) -> Result<f64> {
    env::var(name)
        .with_context(|| format!("{name} is not set; run through the harness"))?
        .parse()
        .with_context(|| format!("{name} is not a number"))
}

fn main() -> Result<()> {
    let (width, height) = (dimension("BENCH_WIDTH")?, dimension("BENCH_HEIGHT")?);
    let page = format!("index.html?{}", scenario_query());
    tauri::Builder::default()
        .setup(move |app| {
            WebviewWindowBuilder::new(app, "main", WebviewUrl::App(page.into()))
                .title("Tauri bench")
                .inner_size(width, height)
                .build()?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .context("running the Tauri app")
}
