use anyhow::Result;

use crate::{
    report::{Report, Step},
    workspace,
};

const PUBLIC_API_CRATES: [&str; 6] = ["crownui", "view", "widgets", "graphics", "windowing", "reactive"];

/// Writes (or with `check`, compares) `api/<crate>.txt` using `cargo public-api`.
pub(crate) fn run(check: bool) -> Result<Report> {
    let (installed, _) = workspace::run("cargo", &["public-api", "--version"])?;
    if !installed {
        return Ok(Report::new(
            "api",
            vec![Step {
                name: "cargo-public-api is not installed".into(),
                ok: false,
                detail: "install it with `cargo install cargo-public-api` (needs a nightly toolchain)".into(),
            }],
        ));
    }
    let root = workspace::root()?;
    std::fs::create_dir_all(root.join("api"))?;
    let mut steps = Vec::new();
    for name in PUBLIC_API_CRATES {
        let (ok, listing) = workspace::run("cargo", &["public-api", "-p", name, "-sss"])?;
        let path = root.join("api").join(format!("{name}.txt"));
        if !ok {
            steps.push(Step {
                name: format!("public-api {name}"),
                ok: false,
                detail: listing,
            });
            continue;
        }
        if check {
            let recorded = std::fs::read_to_string(&path).unwrap_or_default();
            steps.push(Step {
                name: format!("api/{name}.txt up to date"),
                ok: recorded == listing,
                detail: if recorded == listing {
                    String::new()
                } else {
                    "public API changed; run `cargo xtask api` and review the diff".into()
                },
            });
        } else {
            std::fs::write(&path, listing)?;
            steps.push(Step {
                name: format!("wrote api/{name}.txt"),
                ok: true,
                detail: String::new(),
            });
        }
    }
    Ok(Report::new("api", steps))
}
