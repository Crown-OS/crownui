mod dmabuf;
mod egl;
mod framebuffer;
mod gles;
mod headless_egl;
mod offscreen;
mod vulkan;
mod wayland;

use std::fmt::Display;

use graphics::GraphicsError;
use impellers::DisplayList;

use crate::error::InitError;
pub(crate) use dmabuf::DmabufTarget;
use gles::GlesTarget;
use headless_egl::HeadlessPlatform;
pub(crate) use offscreen::OffscreenTarget;
use vulkan::VulkanTarget;
pub(crate) use wayland::WaylandSurfacePointers;

const RENDERER_ENV: &str = "CROWNUI_RENDERER";
pub(crate) const MSAA_SAMPLES: i32 = 4;

/// The GPU API a renderer presents with.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RenderApi {
    /// Vulkan swapchain (the default).
    Vulkan,
    /// OpenGL ES through EGL (`CROWNUI_RENDERER=gles`, or when Vulkan is unavailable).
    Gles,
    /// Offscreen OpenGL ES on the EGL surfaceless platform.
    GlesSurfaceless,
    /// Offscreen OpenGL ES kept current by a pbuffer (no surfaceless platform).
    GlesPbuffer,
}

pub(crate) enum RenderTarget {
    Vulkan(VulkanTarget),
    Gles(GlesTarget),
    Offscreen(OffscreenTarget),
    Dmabuf(DmabufTarget),
}

pub(crate) fn frame_error(error: impl Display) -> GraphicsError {
    GraphicsError::Frame(error.to_string())
}

impl RenderTarget {
    pub(crate) fn create(window: WaylandSurfacePointers) -> Result<Self, InitError> {
        if std::env::var_os(RENDERER_ENV).is_some_and(|value| value == "gles") {
            return GlesTarget::new(window).map(Self::Gles);
        }
        VulkanTarget::new(window)
            .map(Self::Vulkan)
            .or_else(|vulkan| {
                tracing::warn!(%vulkan, "Impeller Vulkan unavailable, falling back to GLES");
                GlesTarget::new(window)
                    .map(Self::Gles)
                    .map_err(|gles| InitError::NoBackend {
                        vulkan: Box::new(vulkan),
                        gles: Box::new(gles),
                    })
            })
    }

    pub(crate) fn api(&self) -> RenderApi {
        match self {
            Self::Vulkan(_) => RenderApi::Vulkan,
            Self::Gles(_) => RenderApi::Gles,
            Self::Offscreen(target) => match target.platform() {
                HeadlessPlatform::Surfaceless => RenderApi::GlesSurfaceless,
                HeadlessPlatform::Pbuffer => RenderApi::GlesPbuffer,
            },
            Self::Dmabuf(_) => RenderApi::GlesSurfaceless,
        }
    }

    pub(crate) fn context(&self) -> &impellers::Context {
        match self {
            Self::Vulkan(target) => target.context(),
            Self::Gles(target) => target.context(),
            Self::Offscreen(target) => target.context(),
            Self::Dmabuf(target) => target.context(),
        }
    }

    /// Makes this target's context current where the API is thread-bound.
    pub(crate) fn activate(&self) -> Result<(), khronos_egl::Error> {
        match self {
            Self::Vulkan(_) => Ok(()),
            Self::Gles(target) => target.activate(),
            Self::Offscreen(target) => target.activate(),
            Self::Dmabuf(target) => target.activate(),
        }
    }

    pub(crate) fn resize(&mut self, width: u32, height: u32) {
        match self {
            Self::Vulkan(target) => target.resize(width, height),
            Self::Gles(target) => target.resize(width, height),
            Self::Offscreen(target) => target.resize(width, height),
            Self::Dmabuf(target) => target.resize(width, height),
        }
    }

    pub(crate) fn presented_size(&self) -> Option<(u32, u32)> {
        match self {
            Self::Vulkan(target) => target.presented_size(),
            Self::Gles(target) => target.presented_size(),
            Self::Offscreen(target) => target.presented_size(),
            Self::Dmabuf(target) => target.presented_size(),
        }
    }

    /// Draws and presents `frame`; `Ok(false)` when no surface was available (e.g. mid-resize).
    pub(crate) fn present(&mut self, frame: &DisplayList) -> Result<bool, GraphicsError> {
        match self {
            Self::Vulkan(target) => target.present(frame),
            Self::Gles(target) => target.present(frame),
            Self::Offscreen(target) => target.present(frame),
            Self::Dmabuf(target) => target.present(frame),
        }
    }

    /// The last frame as `(width, height, bottom-up premultiplied RGBA8)`; offscreen only.
    pub(crate) fn read_rgba(&self) -> Option<(u32, u32, Vec<u8>)> {
        match self {
            Self::Offscreen(target) => Some(target.read_rgba()),
            Self::Dmabuf(target) => Some(target.read_rgba()),
            Self::Vulkan(_) | Self::Gles(_) => None,
        }
    }

    pub(crate) fn release_swapchain(&mut self) {
        if let Self::Vulkan(target) = self {
            target.release_swapchain();
        }
    }
}
