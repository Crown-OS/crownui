use serde::{Serialize, de::DeserializeOwned};

use crate::{ConfigError, watch::SectionWatch};

/// Reads `section`, falling back to `T::default()`; a missing file is created with the defaults.
pub fn load_section<T: DeserializeOwned + Serialize + Default>(section: &str) -> T {
    crownos_config::load(section)
}

/// Writes `section` atomically; watchers in this process do not see their own write.
pub fn save_section<T: Serialize>(section: &str, value: &T) -> Result<(), ConfigError> {
    crownos_config::save(section, value).map_err(|source| ConfigError::Write {
        section: section.to_owned(),
        source,
    })
}

/// Follows edits to `section`, starting from `initial`.
pub fn watch_section<T>(section: &str, initial: T) -> SectionWatch<T>
where
    T: DeserializeOwned + Clone + Send + Sync + 'static,
{
    SectionWatch::subscribe(section, initial)
}
