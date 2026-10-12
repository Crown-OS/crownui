mod bundled_crate;
mod manifest;
mod source;

use std::{fs, path::Path};

use anyhow::{Context, Result};
use bundled_crate::{BUNDLED_CRATES, BundledCrate};

use crate::{
    report::{Report, Step},
    util::{copy_dir::copy_dir, files_with_extension::files_with_extension},
    workspace,
};

const BUNDLE_DIRECTORY: &str = "target/bundle/crownui";
const FACADE_DIRECTORY: &str = "crownui";
const ROOT_FILES: &[&str] = &["LICENSE-MIT", "LICENSE-APACHE", "Cargo.lock"];
const SKIPPED_IN_FACADE: &[&str] = &["target", "tests", "Cargo.toml"];
const SKIPPED_IN_CRATES: &[&str] = &[
    "target",
    "tests",
    "examples",
    "benches",
    "Cargo.toml",
    "Cargo.lock",
    ".git",
    ".github",
    ".gitignore",
];

/// Assembles the single publishable `crownui` crate in `target/bundle/crownui`, every library
/// compiled in as a module, and verifies it with `cargo package`.
pub(crate) fn run() -> Result<Report> {
    let root = workspace::root()?;
    let bundle = root.join(BUNDLE_DIRECTORY);
    assemble(&root, &bundle)?;
    let manifest_path = bundle.join("Cargo.toml");
    let target_directory = root.join("target");
    let (packaged, detail) = workspace::run(
        "cargo",
        &[
            "package",
            "--allow-dirty",
            "--manifest-path",
            &manifest_path.to_string_lossy(),
            "--target-dir",
            &target_directory.to_string_lossy(),
        ],
    )?;
    Ok(Report::new(
        "bundle",
        vec![
            Step {
                name: format!("assembled {}", bundle.display()),
                ok: true,
                detail: String::new(),
            },
            Step {
                name: "cargo package".into(),
                ok: packaged,
                detail,
            },
        ],
    ))
}

fn assemble(root: &Path, bundle: &Path) -> Result<()> {
    if bundle.exists() {
        fs::remove_dir_all(bundle)?;
    }
    copy_dir(&root.join(FACADE_DIRECTORY), bundle, SKIPPED_IN_FACADE)?;
    for file in ROOT_FILES {
        fs::copy(root.join(file), bundle.join(file)).with_context(|| format!("copying {file}"))?;
    }
    fs::write(bundle.join("Cargo.toml"), manifest::render(root)?)?;
    for bundled in BUNDLED_CRATES {
        let destination = bundle.join(bundled.module);
        copy_dir(
            &root.join(bundled.directory),
            &destination,
            SKIPPED_IN_CRATES,
        )
        .with_context(|| format!("copying {}", bundled.directory))?;
        rewrite_crate(&destination.join("src"), bundled)?;
    }
    rewrite_facade(&bundle.join("src"))
}

fn rewrite_facade(source_directory: &Path) -> Result<()> {
    let root_path = source_directory.join("lib.rs");
    for path in files_with_extension(source_directory, "rs")? {
        let text = fs::read_to_string(&path)?;
        let rewritten = if path == root_path {
            source::import_bundle_in_inline_modules(&text)
                + &source::module_declarations(BUNDLED_CRATES)
        } else {
            source::import_bundle(&text)
        };
        fs::write(&path, rewritten)?;
    }
    Ok(())
}

fn rewrite_crate(source_directory: &Path, bundled: &BundledCrate) -> Result<()> {
    let files = files_with_extension(source_directory, "rs")?
        .into_iter()
        .map(|path| fs::read_to_string(&path).map(|text| (path, text)))
        .collect::<Result<Vec<_>, _>>()?;
    let exported_macros: Vec<&str> = files
        .iter()
        .flat_map(|(_, text)| source::exported_macros(text))
        .collect();
    for (path, text) in &files {
        fs::write(path, source::rewrite(text, bundled, &exported_macros))?;
    }
    Ok(())
}
