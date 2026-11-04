pub use crownos_config::schema::{Files, FilesKey, SortKey, TagColor, ViewMode};

use crate::{ConfigError, SectionWatch, load_section, save_section, watch_section};

/// Reads `files.ron`, the file manager's settings, writing the defaults when it is missing.
pub fn load_files() -> Files {
    load_section(Files::SECTION)
}

/// Writes `files.ron`.
pub fn save_files(files: &Files) -> Result<(), ConfigError> {
    save_section(Files::SECTION, files)
}

/// Follows edits to `files.ron`, starting from `initial`.
pub fn watch_files(initial: Files) -> SectionWatch<Files> {
    watch_section(Files::SECTION, initial)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_section_round_trips_through_ron_with_its_defaults() {
        let files = Files {
            show_hidden: true,
            view_mode: ViewMode::List,
            sort_by: SortKey::Modified,
            ..Files::default()
        };
        let text = ron::to_string(&files).unwrap_or_default();
        let read: Files = ron::from_str(&text).unwrap_or_default();
        assert_eq!(read, files);
        assert_eq!(Files::SECTION, "files");
        assert!(read.dirs_first && read.tags.iter().any(|tag| tag.name == "Red"));
    }
}
