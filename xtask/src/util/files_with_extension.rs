use std::{
    fs, io,
    path::{Path, PathBuf},
};

/// Every file under `directory`, recursively, whose extension is `extension`.
pub(crate) fn files_with_extension(directory: &Path, extension: &str) -> io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for entry in fs::read_dir(directory)? {
        let path = entry?.path();
        if path.is_dir() {
            files.extend(files_with_extension(&path, extension)?);
        } else if path.extension().is_some_and(|found| found == extension) {
            files.push(path);
        }
    }
    Ok(files)
}
