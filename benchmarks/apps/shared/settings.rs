//! What the settings scenario shows, shared by the CrownUI and GPUI apps; `apps/web/settings.js`
//! and `apps/qt/Settings.qml` follow the same rules.

use std::time::Duration;

use crate::config::SettingsConfig;

/// Sidebar pages: a title and the Lucide icon file in `apps/web/icons`.
pub const PAGES: [(&str, &str); 8] = [
    ("Wi-Fi", "wifi"),
    ("Bluetooth", "bluetooth"),
    ("Network", "globe"),
    ("Display", "monitor"),
    ("Sound", "volume-2"),
    ("Notifications", "bell"),
    ("Battery & Power", "battery-full"),
    ("Appearance", "palette"),
];

/// What sits at the end of a row.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Control {
    Switch(bool),
    Slider(f32),
    Value(&'static str),
    Chevron,
}

pub const VALUE_TEXT: &str = "Automatic";

pub fn page_title(page: usize) -> &'static str {
    PAGES[page % PAGES.len()].0
}

pub fn page_icon(page: usize) -> &'static str {
    PAGES[page % PAGES.len()].1
}

pub fn section_title(section: usize) -> String {
    format!("SECTION {}", section + 1)
}

/// Row `row` of section `section`, numbered across the page.
pub fn row_index(config: SettingsConfig, section: usize, row: usize) -> usize {
    section * config.rows + row
}

pub fn row_title(page: usize, index: usize) -> String {
    format!("{} option {}", page_title(page), index + 1)
}

pub fn row_description(index: usize) -> String {
    format!("Describes what option {} changes", index + 1)
}

pub fn control(page: usize, index: usize) -> Control {
    match (page + index) % 4 {
        0 => Control::Switch((page + index).is_multiple_of(3)),
        1 => Control::Slider(((page * 7 + index * 13) % 100) as f32 / 100.0),
        2 => Control::Value(VALUE_TEXT),
        _ => Control::Chevron,
    }
}

/// The page open `elapsed` after the first frame, and how long it has been open.
pub fn open_page(config: SettingsConfig, elapsed: Duration) -> (usize, f32) {
    let seconds = elapsed.as_secs_f32();
    let opened = (seconds / config.page_seconds).floor();
    (
        opened as usize % config.pages,
        seconds - opened * config.page_seconds,
    )
}

/// How far a page open for `seconds` has scrolled, wrapping at the end of its `travel`.
pub fn scroll_offset(config: SettingsConfig, seconds: f32, travel: f32) -> f32 {
    seconds * config.scroll_speed % travel.max(1.0)
}
