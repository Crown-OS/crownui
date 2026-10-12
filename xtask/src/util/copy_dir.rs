use std::{fs, io, path::Path};

/// Copies `source` into `destination` recursively, leaving out the top-level entries named in `skipped`.
pub(crate) fn copy_dir(source: &Path, destination: &Path, skipped: &[&str]) -> io::Result<()> {
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let name = entry.file_name();
        if name.to_str().is_some_and(|name| skipped.contains(&name)) {
            continue;
        }
        let target = destination.join(&name);
        if entry.file_type()?.is_dir() {
            copy_dir(&entry.path(), &target, &[])?;
        } else {
            fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}
