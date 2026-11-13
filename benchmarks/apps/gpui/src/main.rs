mod animation;
mod assets;
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

use anyhow::{Result, anyhow};
use gpui::{App, AppContext, Application, Bounds, WindowBounds, WindowOptions, point, px, size};

use crate::{
    animation::Animation,
    assets::Assets,
    config::{BenchConfig, Scenario},
    hello::Hello,
    list::List,
    settings_window::SettingsWindow,
};

fn main() -> Result<()> {
    let config = BenchConfig::from_env()?;
    Application::new()
        .with_assets(Assets)
        .run(move |cx: &mut App| {
            if let Err(error) = open_window(cx, config) {
                eprintln!("{error:#}");
                cx.quit();
            }
        });
    Ok(())
}

fn open_window(cx: &mut App, config: BenchConfig) -> Result<()> {
    let options = WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(Bounds::new(
            point(px(0.0), px(0.0)),
            size(px(config.width as f32), px(config.height as f32)),
        ))),
        ..WindowOptions::default()
    };
    let opened = match config.scenario {
        Scenario::Hello => cx.open_window(options, |_, cx| cx.new(|_| Hello)).map(drop),
        Scenario::List => cx
            .open_window(options, |_, cx| cx.new(|_| List::new(config.list)))
            .map(drop),
        Scenario::Animation => cx
            .open_window(options, |_, cx| {
                cx.new(|_| Animation::new(config.animation))
            })
            .map(drop),
        Scenario::Settings => cx
            .open_window(options, |_, cx| {
                cx.new(|_| SettingsWindow::new(config.settings))
            })
            .map(drop),
    };
    cx.activate(true);
    opened.map_err(|error| anyhow!("opening the window: {error}"))
}
