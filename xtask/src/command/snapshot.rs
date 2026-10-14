use anyhow::Result;

use crate::{
    report::{Report, Step},
    workspace,
};

/// Lists pending `.snap.new` files that need review.
pub(crate) fn run() -> Result<Report> {
    let root = workspace::root()?;
    let mut pending = Vec::new();
    let mut stack = vec![root];
    while let Some(directory) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&directory) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path
                .file_name()
                .is_some_and(|name| name == "target" || name == ".git")
            {
                continue;
            }
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|extension| extension == "new") {
                pending.push(path.display().to_string());
            }
        }
    }
    let steps = vec![Step {
        name: format!("{} pending snapshot(s)", pending.len()),
        ok: pending.is_empty(),
        detail: format!(
            "{}\nreview each diff, then rename .snap.new to .snap (or run `cargo insta review`)",
            pending.join("\n")
        ),
    }];
    Ok(Report::new("snapshot", steps))
}
