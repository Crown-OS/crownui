mod animation;
#[path = "../../shared/config.rs"]
mod config;
#[path = "../../shared/geometry.rs"]
mod geometry;
mod hello;
mod list;
mod palette;
#[path = "../../shared/settings.rs"]
mod settings;
mod settings_window;

use anyhow::Result;
use crownui::{kit, prelude::*};

use crate::config::{BenchConfig, FrameRate, Scenario};

fn main() -> Result<()> {
    let config = BenchConfig::from_env()?;
    let pacing = match config.frame_rate {
        FrameRate::Display => FramePacing::Display,
        FrameRate::Uncapped => FramePacing::Unpaced,
    };
    let window = AppConfig::new("CrownUI bench")
        .size(config.width, config.height)
        .frame_pacing(pacing);
    match config.scenario {
        Scenario::Hello if std::env::var_os("BENCH_KEEP_AWAKE").is_some() => crownui::run(
            component(|cx| {
                cx.set_idle_inhibited(true);
                hello::view()
            }),
            window,
        )?,
        Scenario::Hello => crownui::run(hello::view(), window)?,
        Scenario::List => crownui::run(list::view(config.list), window)?,
        Scenario::Animation => crownui::run(animation::view(config.animation), window)?,
        Scenario::Settings => crownui::run(
            settings_window::view(config.settings),
            window.font(kit::inter_font().to_vec(), Some(kit::INTER_FAMILY)),
        )?,
    }
    Ok(())
}
