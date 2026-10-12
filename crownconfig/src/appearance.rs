pub use crownos_config::schema::{AccentColor, AnimationProfile, Appearance};

use crate::{ConfigError, load_section, save_section};

/// Reads `appearance.ron`, writing the defaults when it is missing.
pub fn load_appearance() -> Appearance {
    load_section(Appearance::SECTION)
}

/// Writes `appearance.ron`.
pub fn save_appearance(appearance: &Appearance) -> Result<(), ConfigError> {
    save_section(Appearance::SECTION, appearance)
}
