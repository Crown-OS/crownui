use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

use crate::{
    report::{Report, Step},
    workspace,
};

const SHADER_DIRECTORY: &str = "impeller/shaders";
const IMPELLER_CACHE_ENV: &str = "IMPELLER_CACHE_DIR";
const DEFAULT_IMPELLER_CACHE: &str = ".impeller_cache";

/// Compiles every `impeller/shaders/*.frag` into the `.iplr` runtime stage committed next to it
/// (or, with `check`, fails when a committed one is stale).
pub(crate) fn run(check: bool) -> Result<Report> {
    let root = workspace::root()?;
    let compiler = impellerc(&root)?;
    let output_directory = root.join("target").join("shaders");
    std::fs::create_dir_all(&output_directory)?;
    let mut sources: Vec<PathBuf> = std::fs::read_dir(root.join(SHADER_DIRECTORY))?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "frag")
        })
        .collect();
    sources.sort();
    let steps = sources
        .iter()
        .map(|source| compile(&compiler, source, &output_directory, check))
        .collect::<Result<_>>()?;
    Ok(Report::new("shaders", steps))
}

fn compile(compiler: &Path, source: &Path, output_directory: &Path, check: bool) -> Result<Step> {
    let committed = source.with_extension("iplr");
    let name = committed
        .file_name()
        .context("shader source has no file name")?
        .to_string_lossy()
        .into_owned();
    let compiled = output_directory.join(&name);
    let arguments = [
        "--runtime-stage-gles".to_owned(),
        "--runtime-stage-gles3".to_owned(),
        "--runtime-stage-vulkan".to_owned(),
        "--iplr".to_owned(),
        "--input-type=frag".to_owned(),
        format!("--input={}", source.display()),
        format!("--sl={}", compiled.display()),
        format!("--spirv={}", compiled.with_extension("spv").display()),
    ];
    let arguments: Vec<&str> = arguments.iter().map(String::as_str).collect();
    let (ok, output) = workspace::run(&compiler.to_string_lossy(), &arguments)?;
    if !ok {
        return Ok(Step {
            name: format!("compile {name}"),
            ok: false,
            detail: output,
        });
    }
    let fresh = std::fs::read(&compiled)?;
    if check {
        let up_to_date = std::fs::read(&committed).is_ok_and(|recorded| recorded == fresh);
        return Ok(Step {
            name: format!("{name} up to date"),
            ok: up_to_date,
            detail: if up_to_date {
                String::new()
            } else {
                "shader source changed; run `cargo xtask shaders`".into()
            },
        });
    }
    std::fs::write(&committed, fresh)?;
    Ok(Step {
        name: format!("wrote {name}"),
        ok: true,
        detail: String::new(),
    })
}

/// The `impellerc` shipped with the Impeller SDK `impellers` downloaded, so compiled shaders
/// match the linked engine.
fn impellerc(root: &Path) -> Result<PathBuf> {
    let cache = std::env::var_os(IMPELLER_CACHE_ENV)
        .map_or_else(|| root.join(DEFAULT_IMPELLER_CACHE), PathBuf::from);
    let compiler = std::fs::read_dir(&cache)
        .with_context(|| {
            format!(
                "no Impeller SDK in {}; build the impeller crate first",
                cache.display()
            )
        })?
        .filter_map(|entry| {
            entry
                .ok()
                .map(|entry| entry.path().join("bin").join("impellerc"))
        })
        .find(|path| path.is_file());
    match compiler {
        Some(compiler) => Ok(compiler),
        None => bail!("no impellerc under {}", cache.display()),
    }
}
