#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

mod appearance;
mod error;
mod keybind;
mod section;
mod theme_overrides;
mod theme_settings;
mod watch;

pub use appearance::{AccentColor, AnimationProfile, Appearance, load_appearance, save_appearance};
pub use error::ConfigError;
pub use keybind::{KeyCode, Keybind, Mods};
pub use section::{load_section, save_section, watch_section};
pub use theme_overrides::{
    AccentOverride, COLOR_TOKENS, Font, Radii, ThemeOverrides, is_color_token, load_theme_overrides,
};
pub use theme_settings::{ThemeSettings, watch_theme_settings};
pub use watch::SectionWatch;
