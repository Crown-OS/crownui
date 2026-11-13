#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

mod backend;
mod capture;
mod carrier;
mod draw_items;
mod error;
mod finish;
mod renderer;
mod renderer_pool;
mod scene;
mod scene_tree;
mod text;
mod util;
mod window_look;

pub use backend::CaptureBackend;
pub use capture::{Capture, CaptureApp, THEME_ENV, Theme, capture, mount, mount_app};
pub use carrier::{Carrier, FIGMA_BUILDER};
pub use error::CaptureError;
pub use finish::finish;
pub use renderer::CaptureRenderer;
pub use scene::{Fill, Item, Scene, SceneMaterial, SceneNode, Shadow, StrokeSpec, TextItem};
pub use text::{CaptureTextSystem, CapturedText};
