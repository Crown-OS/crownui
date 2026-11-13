use std::{fs, path::Path};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::scenario::Scenario;

/// The scenario constants in `apps/spec.toml`, shared by every app.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Spec {
    pub window: WindowSpec,
    pub list: ListSpec,
    pub animation: AnimationSpec,
    pub settings: SettingsSpec,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct WindowSpec {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ListSpec {
    pub rows: u32,
    pub row_height: u32,
    pub swatch: u32,
    pub scroll_speed: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AnimationSpec {
    pub boxes: u32,
    pub box_size: u32,
    pub labels: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SettingsSpec {
    pub pages: u32,
    pub sections: u32,
    pub rows: u32,
    pub sidebar_width: u32,
    pub scroll_speed: u32,
    pub page_seconds: u32,
}

impl Spec {
    pub fn load(path: &Path) -> Result<Self> {
        let text =
            fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
        toml::from_str(&text).with_context(|| format!("parsing {}", path.display()))
    }

    pub fn environment(&self, scenario: Scenario) -> [(&'static str, String); 16] {
        [
            ("BENCH_SCENARIO", scenario.name().to_owned()),
            ("BENCH_WIDTH", self.window.width.to_string()),
            ("BENCH_HEIGHT", self.window.height.to_string()),
            ("BENCH_ROWS", self.list.rows.to_string()),
            ("BENCH_ROW_HEIGHT", self.list.row_height.to_string()),
            ("BENCH_SWATCH", self.list.swatch.to_string()),
            ("BENCH_SCROLL_SPEED", self.list.scroll_speed.to_string()),
            ("BENCH_BOXES", self.animation.boxes.to_string()),
            ("BENCH_BOX_SIZE", self.animation.box_size.to_string()),
            ("BENCH_LABELS", self.animation.labels.to_string()),
            ("BENCH_PAGES", self.settings.pages.to_string()),
            ("BENCH_SECTIONS", self.settings.sections.to_string()),
            ("BENCH_SECTION_ROWS", self.settings.rows.to_string()),
            (
                "BENCH_SIDEBAR_WIDTH",
                self.settings.sidebar_width.to_string(),
            ),
            (
                "BENCH_PAGE_SCROLL_SPEED",
                self.settings.scroll_speed.to_string(),
            ),
            ("BENCH_PAGE_SECONDS", self.settings.page_seconds.to_string()),
        ]
    }
}
