use std::ffi::c_void;

use ash::vk::Handle;
use graphics::GraphicsError;
use impellers::{DisplayList, VkSwapChain};

use super::{frame_error, wayland::WaylandSurfacePointers};
use crate::{
    error::InitError,
    vulkan::{SurfaceExtent, create_context, create_wayland_surface, destroy_surface},
};

/// A Vulkan swapchain on a Wayland surface; fields drop swapchain first, context last.
pub(crate) struct VulkanTarget {
    swapchain: Option<VkSwapChain>,
    extent: SurfaceExtent,
    context: impellers::Context,
}

impl VulkanTarget {
    pub(crate) fn new(window: WaylandSurfacePointers) -> Result<Self, InitError> {
        let context = create_context()?;
        let instance = context
            .get_vulkan_info()
            .map_err(InitError::Impeller)?
            .vk_instance;
        // SAFETY: the instance is the context's own; the caller keeps the Wayland objects alive
        // longer than the renderer.
        let surface = unsafe { create_wayland_surface(instance, window.display, window.surface) }?;
        let Some(extent) = SurfaceExtent::claim(surface) else {
            // SAFETY: the surface was just created on `instance` and never shared.
            unsafe { destroy_surface(instance, surface) }?;
            return Err(InitError::SurfaceSlotsExhausted(SurfaceExtent::capacity()));
        };
        // SAFETY: the surface belongs to the context's instance; Impeller takes ownership of it.
        let swapchain =
            unsafe { context.create_new_vulkan_swapchain(surface.as_raw() as *mut c_void) }.ok_or(
                InitError::Impeller("ImpellerVulkanSwapchainCreateNew failed"),
            )?;
        Ok(Self {
            swapchain: Some(swapchain),
            extent,
            context,
        })
    }

    pub(crate) fn context(&self) -> &impellers::Context {
        &self.context
    }

    pub(crate) fn resize(&mut self, width: u32, height: u32) {
        self.extent.set_wanted(width, height);
    }

    pub(crate) fn presented_size(&self) -> Option<(u32, u32)> {
        self.extent.swapchain_extent()
    }

    pub(crate) fn present(&mut self, frame: &DisplayList) -> Result<bool, GraphicsError> {
        let Some(mut surface) = self
            .swapchain
            .as_mut()
            .and_then(VkSwapChain::acquire_next_surface_new)
        else {
            return Ok(false);
        };
        surface.draw_display_list(frame).map_err(frame_error)?;
        surface.present().map_err(frame_error)?;
        Ok(true)
    }

    pub(crate) fn release_swapchain(&mut self) {
        self.swapchain = None;
    }
}
