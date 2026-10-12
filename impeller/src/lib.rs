#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

mod backend;
mod composite;
mod dmabuf;
mod error;
mod offscreen;
mod pixels;
mod png_encoding;
mod renderer;
mod resources;
mod target;
mod text;
mod translate;
mod util;
mod vulkan;

pub use backend::ImpellerBackend;
pub use dmabuf::{DmabufImage, DmabufPlane, RenderNode};
pub use offscreen::Offscreen;
pub use pixels::Pixels;
pub use png_encoding::encode_png;
pub use renderer::ImpellerRenderer;
pub use target::RenderApi;
pub use text::{ImpellerTextLayout, ImpellerTextSystem};
pub use vulkan::select_vulkan_driver;
