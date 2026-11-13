use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use impeller::Pixels;

use crate::{
    CaptureApp, CaptureError, Theme, capture,
    scene::Scene,
    util::{ascii_json::to_ascii_json, crc32::crc32, file_name::file_name},
};

/// The Figma-side builder, shipped inside every carrier.
pub const FIGMA_BUILDER: &str = include_str!("../figma/builder.js");

const BUILDER_CHUNK: &[u8; 4] = b"crJs";
const SCENE_CHUNK: &[u8; 4] = b"crSc";
const IMAGE_CHUNK: &[u8; 4] = b"crIm";
const PAGE_CHUNK: &[u8; 4] = b"crPg";

/// Every captured state of one app, written as a carrier PNG (uploaded to Figma, where the
/// builder reads its chunks) plus each state's GPU render for comparison.
pub struct Carrier {
    app: String,
    description: String,
    scenes: Vec<Scene>,
    renders: Vec<(String, Pixels)>,
    images: BTreeMap<u64, Vec<u8>>,
}

impl Carrier {
    /// An empty carrier for `app`, described on its Figma page by `description`.
    pub fn new(app: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            app: app.into(),
            description: description.into(),
            scenes: Vec::new(),
            renders: Vec::new(),
            images: BTreeMap::new(),
        }
    }

    /// Captures the app's current state as `name` in `group`, in the run's theme.
    pub fn capture_state(
        &mut self,
        app: &mut CaptureApp,
        group: &str,
        name: &str,
        description: &str,
    ) -> Result<&mut Self, CaptureError> {
        let capture = capture(app, Theme::current(), group, name, description)?;
        Ok(self.add(capture))
    }

    /// Adds one capture.
    pub fn add(&mut self, capture: crate::Capture) -> &mut Self {
        self.images.extend(capture.images);
        self.renders
            .push((file_name(&capture.scene.name), capture.render));
        self.scenes.push(capture.scene);
        self
    }

    /// Writes `<dir>/<app>.<theme>.figma.png` and `<dir>/<app>/<state>-<theme>.png`, into
    /// `$CROWN_DESIGN_DIR` when `dir` is `None`; returns the carrier path.
    pub fn write(&self, dir: Option<&Path>) -> Result<PathBuf, CaptureError> {
        let dir = dir.map_or_else(output_dir, Path::to_path_buf);
        let renders = dir.join(file_name(&self.app));
        std::fs::create_dir_all(&renders).map_err(|source| CaptureError::Write {
            path: renders.clone(),
            source,
        })?;
        for (name, pixels) in &self.renders {
            let path = renders.join(format!("{name}.png"));
            write_file(&path, &impeller::encode_png(pixels)?)?;
        }
        let theme = Theme::current().name().to_ascii_lowercase();
        let carrier = dir.join(format!("{}.{theme}.figma.png", file_name(&self.app)));
        write_file(&carrier, &self.encode()?)?;
        Ok(carrier)
    }

    fn encode(&self) -> Result<Vec<u8>, CaptureError> {
        let mut png = impeller::encode_png(&Pixels {
            width: 1,
            height: 1,
            rgba: vec![0; 4],
        })?;
        let end = png.len() - 12;
        let trailer = png.split_off(end);
        push_chunk(&mut png, BUILDER_CHUNK, FIGMA_BUILDER.as_bytes());
        let page = serde_json::json!({ "app": self.app, "description": self.description });
        push_chunk(
            &mut png,
            PAGE_CHUNK,
            to_ascii_json(&page.to_string()).as_bytes(),
        );
        for scene in &self.scenes {
            let json = to_ascii_json(&serde_json::to_string(scene)?);
            push_chunk(&mut png, SCENE_CHUNK, json.as_bytes());
        }
        for (key, image) in &self.images {
            let mut data = Vec::with_capacity(8 + image.len());
            data.extend_from_slice(&key.to_be_bytes());
            data.extend_from_slice(image);
            push_chunk(&mut png, IMAGE_CHUNK, &data);
        }
        png.extend_from_slice(&trailer);
        Ok(png)
    }
}

/// `$CROWN_DESIGN_DIR`, or `target/design`.
fn output_dir() -> PathBuf {
    std::env::var_os("CROWN_DESIGN_DIR")
        .map_or_else(|| PathBuf::from("target/design"), PathBuf::from)
}

fn push_chunk(png: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
    let length = u32::try_from(data.len()).unwrap_or(u32::MAX);
    png.extend_from_slice(&length.to_be_bytes());
    png.extend_from_slice(kind);
    png.extend_from_slice(data);
    png.extend_from_slice(&crc32(&[kind, data]).to_be_bytes());
}

fn write_file(path: &Path, bytes: &[u8]) -> Result<(), CaptureError> {
    std::fs::write(path, bytes).map_err(|source| CaptureError::Write {
        path: path.to_owned(),
        source,
    })
}
