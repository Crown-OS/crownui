use serde::{Serialize, de::DeserializeOwned};

use crate::{ConfigError, watch::SectionWatch};

/// Reads `section`: the defaults of the installed release with the user's changes on top.
pub fn load_section<T: DeserializeOwned + Serialize + Default>(section: &str) -> T {
    crownos_config::load(section)
}

/// Writes the keys of `section` that the user changed, atomically; watchers in this process do
/// not see their own write.
pub fn save_section<T: Serialize + DeserializeOwned + Default>(
    section: &str,
    value: &T,
) -> Result<(), ConfigError> {
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
