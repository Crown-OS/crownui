use std::{env, str::FromStr};

use anyhow::{Context, Result, bail};

/// The scenario and its constants, handed over by the harness as `BENCH_*` variables.
#[derive(Debug, Clone, Copy)]
pub struct BenchConfig {
    pub scenario: Scenario,
    pub width: u32,
    pub height: u32,
    pub list: ListConfig,
    pub animation: AnimationConfig,
    pub settings: SettingsConfig,
    #[allow(
        dead_code,
        reason = "GPUI's frame rate is chosen when it is built, by patching gpui"
    )]
    pub frame_rate: FrameRate,
}

#[derive(Debug, Clone, Copy)]
pub enum Scenario {
    Hello,
    List,
    Animation,
    Settings,
}

/// Whether the toolkit paces frames by the display or draws them as fast as it can.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameRate {
    Display,
    Uncapped,
}

#[derive(Debug, Clone, Copy)]
pub struct ListConfig {
    pub rows: usize,
    pub row_height: f32,
    pub swatch: f32,
    pub scroll_speed: f32,
    pub viewport_height: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct AnimationConfig {
    pub boxes: usize,
    pub box_size: f32,
    pub labels: usize,
    pub width: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct SettingsConfig {
    pub pages: usize,
    pub sections: usize,
    pub rows: usize,
    pub sidebar_width: f32,
    pub scroll_speed: f32,
    pub page_seconds: f32,
}

impl BenchConfig {
    pub fn from_env() -> Result<Self> {
        let scenario = match variable::<String>("BENCH_SCENARIO")?.as_str() {
            "hello" => Scenario::Hello,
            "list" => Scenario::List,
            "animation" => Scenario::Animation,
            "settings" => Scenario::Settings,
            other => bail!("unknown scenario {other:?}"),
        };
        let width = variable("BENCH_WIDTH")?;
        let height = variable("BENCH_HEIGHT")?;
        Ok(Self {
            scenario,
            width,
            height,
            list: ListConfig {
                rows: variable("BENCH_ROWS")?,
                row_height: variable("BENCH_ROW_HEIGHT")?,
                swatch: variable("BENCH_SWATCH")?,
                scroll_speed: variable("BENCH_SCROLL_SPEED")?,
                viewport_height: height as f32,
            },
            animation: AnimationConfig {
                boxes: variable("BENCH_BOXES")?,
                box_size: variable("BENCH_BOX_SIZE")?,
                labels: variable("BENCH_LABELS")?,
                width: width as f32,
            },
            settings: SettingsConfig {
                pages: variable("BENCH_PAGES")?,
                sections: variable("BENCH_SECTIONS")?,
                rows: variable("BENCH_SECTION_ROWS")?,
                sidebar_width: variable("BENCH_SIDEBAR_WIDTH")?,
                scroll_speed: variable("BENCH_PAGE_SCROLL_SPEED")?,
                page_seconds: variable("BENCH_PAGE_SECONDS")?,
            },
            frame_rate: if env::var_os("BENCH_UNCAPPED").is_some() {
                FrameRate::Uncapped
            } else {
                FrameRate::Display
            },
        })
    }
}

fn variable<T>(name: &str) -> Result<T>
where
    T: FromStr,
    T::Err: std::error::Error + Send + Sync + 'static,
{
    env::var(name)
        .with_context(|| format!("{name} is not set; run through the harness"))?
        .parse()
        .with_context(|| format!("{name} is not a valid value"))
}
