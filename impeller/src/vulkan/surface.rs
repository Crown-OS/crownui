use std::ffi::c_void;

use ash::vk::{self, Handle};

use super::loader::entry;
use crate::error::InitError;

/// Creates a `VkSurfaceKHR` for a Wayland surface on Impeller's instance.
///
/// # Safety
/// `instance` must be the live instance of an Impeller Vulkan context, and `display` and
/// `surface` live `wl_display` / `wl_surface` pointers that outlive the returned surface.
pub(crate) unsafe fn create_wayland_surface(
    instance: *mut c_void,
    display: *mut c_void,
    surface: *mut c_void,
) -> Result<vk::SurfaceKHR, InitError> {
    let entry = entry()?;
    // SAFETY: the caller guarantees `instance` is a live VkInstance.
    let instance =
        unsafe { ash::Instance::load(entry.static_fn(), vk::Instance::from_raw(instance as u64)) };
    let wayland_surface = ash::khr::wayland_surface::Instance::new(entry, &instance);
    let create_info = vk::WaylandSurfaceCreateInfoKHR::default()
        .display(display)
        .surface(surface);
    // SAFETY: the caller guarantees both Wayland pointers are live.
    Ok(unsafe { wayland_surface.create_wayland_surface(&create_info, None) }?)
}

/// Destroys a surface that was never handed to Impeller.
///
/// # Safety
/// `instance` must be the live instance `surface` was created on, and nothing may use `surface`.
pub(crate) unsafe fn destroy_surface(
    instance: *mut c_void,
    surface: vk::SurfaceKHR,
) -> Result<(), InitError> {
    let entry = entry()?;
    // SAFETY: the caller guarantees `instance` is a live VkInstance.
    let instance =
        unsafe { ash::Instance::load(entry.static_fn(), vk::Instance::from_raw(instance as u64)) };
    // SAFETY: the caller guarantees `surface` belongs to `instance` and is unused.
    unsafe { ash::khr::surface::Instance::new(entry, &instance).destroy_surface(surface, None) };
    Ok(())
}
