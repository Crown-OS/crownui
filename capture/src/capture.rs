use std::collections::HashMap;

use crowntest::TestApp;
use graphics::ImageId;
use impeller::{Offscreen, Pixels};
use view::{ColorScheme, View};

use crate::{
    CaptureBackend, CaptureError,
    scene::{Scene, SceneMaterial},
    scene_tree::scene_tree,
    util::fnv_hash::fnv_hash,
};

/// A headless app rendered offscreen by Impeller, ready to capture.
pub type CaptureApp = TestApp<CaptureBackend>;

/// The appearance a capture was taken in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    /// Light appearance.
    Light,
    /// Dark appearance.
    Dark,
}

/// Selects the theme of a capture run: `Dark` or `Light` (the default).
pub const THEME_ENV: &str = "CROWN_DESIGN_THEME";

impl Theme {
    /// The run's theme, from [`THEME_ENV`]. Apps that follow the configured appearance also need
    /// `CROWN_CONFIG_DIR` pointing at a matching config (see `scripts/capture-app.sh`).
    pub fn current() -> Self {
        match std::env::var(THEME_ENV) {
            Ok(theme) if theme.eq_ignore_ascii_case("dark") => Self::Dark,
            _ => Self::Light,
        }
    }

    /// The matching color scheme.
    pub fn scheme(self) -> ColorScheme {
        match self {
            Self::Light => ColorScheme::Light,
            Self::Dark => ColorScheme::Dark,
        }
    }

    /// `Light` or `Dark`.
    pub fn name(self) -> &'static str {
        match self {
            Self::Light => "Light",
            Self::Dark => "Dark",
        }
    }
}

/// One captured state: the scene, its images and the GPU render it was taken from.
pub struct Capture {
    /// The editable scene.
    pub scene: Scene,
    /// PNG-encoded images by key.
    pub images: Vec<(u64, Vec<u8>)>,
    /// The frame exactly as Impeller drew it.
    pub render: Pixels,
}

/// Mounts `root` in a `width`×`height` window drawn offscreen at 1×, in the run's theme.
pub fn mount(root: impl View, width: f32, height: f32) -> Result<CaptureApp, CaptureError> {
    let surface = Offscreen {
        width: width.round() as u32,
        height: height.round() as u32,
        scale: 1.0,
    };
    let mut app = TestApp::with_surface(root, width, height, 1.0, &surface)?;
    app.set_color_scheme(Theme::current().scheme());
    app.settle();
    Ok(app)
}

/// Mounts `root` as the real runner would with `config`: its clear color, corner shape, fonts and
/// platform material, in a window of `size` (the config's size when `None`, e.g. pass the output
/// size for layer surfaces).
pub fn mount_app(
    root: impl View,
    config: &runtime::AppConfig,
    size: Option<(f32, f32)>,
) -> Result<CaptureApp, CaptureError> {
    let (width, height) = size.unwrap_or((config.size.width as f32, config.size.height as f32));
    crate::window_look::prepare(
        crate::window_look::WindowLook {
            clear_color: Some(config.clear_color),
            corner_shape: config.corner_shape,
            material: config.material,
        },
        config.fonts.clone(),
    );
    mount(root, width, height)
}

/// Settles animations and captures the current frame as `name` in `group`, taken in `theme`.
pub fn capture(
    app: &mut CaptureApp,
    theme: Theme,
    group: &str,
    name: &str,
    description: &str,
) -> Result<Capture, CaptureError> {
    app.settle();
    let layout = app.ui().inspect_layout();
    let semantics = app.ui().semantic_tree().clone();
    let viewport = app.ui().viewport();
    let renderer = app.inline().renderer();
    let mut images = Vec::with_capacity(renderer.images.len());
    let image_keys: HashMap<ImageId, String> = renderer
        .images
        .iter()
        .map(|(id, pixels)| {
            let key = fnv_hash(&pixels.rgba) ^ u64::from(pixels.width);
            if let Ok(png) = impeller::encode_png(pixels) {
                images.push((key, png));
            }
            (*id, format!("{key:016x}"))
        })
        .collect();
    let root = scene_tree(
        renderer,
        &image_keys,
        &layout,
        &semantics,
        viewport.width,
        viewport.height,
    );
    let scene = Scene {
        name: format!("{name} · {}", theme.name()),
        group: group.to_owned(),
        theme: theme.name().to_owned(),
        description: description.to_owned(),
        width: viewport.width,
        height: viewport.height,
        clear: crate::util::hex_color::hex_color(renderer.composition.clear_color),
        corner_smoothing: match renderer.corner_shape {
            geometry::CornerShape::Round => 0.0,
            geometry::CornerShape::Squircle => 0.6,
        },
        material: renderer.look.material.map(SceneMaterial::from),
        root,
    };
    Ok(Capture {
        scene,
        images,
        render: renderer.read_pixels()?,
    })
}
