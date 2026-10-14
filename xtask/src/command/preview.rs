use anyhow::{Context, Result, bail};
use geometry::Size;
use impeller::{ImpellerBackend, Offscreen};
use runtime::InlineApp;
use taffy_layout::TaffyEngine;

use crate::report::{Report, Step};

const PREVIEW_SIZE: Size = Size::new(480.0, 320.0);
const PREVIEW_SCALE: f32 = 2.0;

/// `preview --list`, `preview <name>` (text snapshot) or `preview <name> --png <path>` (GPU render).
pub(crate) fn run(arguments: &[String]) -> Result<Report> {
    let catalog = widgets::kit::previews();
    if arguments.iter().any(|argument| argument == "--list") {
        let steps = catalog
            .iter()
            .map(|preview| Step {
                name: preview.name.to_owned(),
                ok: true,
                detail: String::new(),
            })
            .collect();
        return Ok(Report::new("preview", steps));
    }
    let Some(name) = arguments
        .iter()
        .find(|argument| !argument.starts_with("--"))
    else {
        bail!("usage: cargo xtask preview [--list | <name> [--png <path>]]");
    };
    let png_path = arguments
        .iter()
        .position(|argument| argument == "--png")
        .and_then(|index| arguments.get(index + 1));
    let preview = catalog
        .iter()
        .find(|preview| preview.name == name)
        .with_context(|| format!("no preview named {name:?}; see `cargo xtask preview --list`"))?;
    let step = match png_path {
        Some(path) => render_png(preview, path)?,
        None => {
            let mut app = crowntest::TestApp::with_size(
                preview.view(),
                PREVIEW_SIZE.width,
                PREVIEW_SIZE.height,
            );
            app.settle();
            Step {
                name: format!("preview {name}"),
                ok: true,
                detail: app.snapshot(),
            }
        }
    };
    Ok(Report::new("preview", vec![step]))
}

fn render_png(preview: &widgets::kit::Preview, path: &str) -> Result<Step> {
    let surface = Offscreen {
        width: (PREVIEW_SIZE.width * PREVIEW_SCALE) as u32,
        height: (PREVIEW_SIZE.height * PREVIEW_SCALE) as u32,
        scale: PREVIEW_SCALE,
    };
    let mut app = InlineApp::<ImpellerBackend, TaffyEngine>::with_surface(
        preview.view(),
        TaffyEngine::new(),
        PREVIEW_SIZE,
        PREVIEW_SCALE,
        &surface,
    )
    .context("creating the offscreen renderer")?;
    app.settle().context("rendering the preview")?;
    let pixels = app
        .renderer_mut()
        .read_pixels()
        .context("reading back pixels")?;
    let png = impeller::encode_png(&pixels).context("encoding PNG")?;
    std::fs::write(path, png).with_context(|| format!("writing {path}"))?;
    Ok(Step {
        name: format!(
            "rendered {} to {path} ({}x{})",
            preview.name, pixels.width, pixels.height
        ),
        ok: true,
        detail: String::new(),
    })
}
