use std::{
    ffi::{CStr, c_char, c_void},
    sync::OnceLock,
};

use ash::vk::{self, Handle};

use super::surface_extents::{
    is_stale, record_driver_min_images, record_mailbox_support, record_swapchain,
    swapchain_image_count, swapchain_present_mode, wanted_extent,
};
use crate::error::InitError;

type VoidFunction = unsafe extern "system" fn();

static ENTRY: OnceLock<ash::Entry> = OnceLock::new();
static REAL_SURFACE_CAPABILITIES: OnceLock<vk::PFN_vkGetPhysicalDeviceSurfaceCapabilitiesKHR> =
    OnceLock::new();
static REAL_SURFACE_PRESENT_MODES: OnceLock<
    Option<vk::PFN_vkGetPhysicalDeviceSurfacePresentModesKHR>,
> = OnceLock::new();
static REAL_CREATE_SWAPCHAIN: OnceLock<vk::PFN_vkCreateSwapchainKHR> = OnceLock::new();
static REAL_ACQUIRE_NEXT_IMAGE: OnceLock<vk::PFN_vkAcquireNextImageKHR> = OnceLock::new();

/// The process-wide Vulkan loader; it stays loaded so interposed pointers never dangle.
pub(crate) fn entry() -> Result<&'static ash::Entry, InitError> {
    if let Some(entry) = ENTRY.get() {
        return Ok(entry);
    }
    // SAFETY: loading libvulkan has no preconditions; the entry is kept for the process lifetime.
    let entry = unsafe { ash::Entry::load() }?;
    Ok(ENTRY.get_or_init(|| entry))
}

/// Creates an Impeller Vulkan context whose swapchains are sized by [`super::SurfaceExtent`].
pub(crate) fn create_context() -> Result<impellers::Context, InitError> {
    let get_instance_proc_addr = entry()?.static_fn().get_instance_proc_addr;
    // SAFETY: the callback only forwards to the loader and never retains the pointers it is given.
    unsafe {
        impellers::Context::new_vulkan(false, |instance, name| {
            resolve_proc(get_instance_proc_addr, instance, name)
        })
    }
    .map_err(InitError::Impeller)
}

/// # Safety
/// `name` must be a valid NUL-terminated string and `instance` null or a live `VkInstance`.
unsafe fn resolve_proc(
    get_instance_proc_addr: vk::PFN_vkGetInstanceProcAddr,
    instance: *mut c_void,
    name: *const c_char,
) -> *mut c_void {
    let instance = vk::Instance::from_raw(instance as u64);
    // SAFETY: forwarded verbatim from Impeller's proc-address request.
    let Some(real) = (unsafe { get_instance_proc_addr(instance, name) }) else {
        return std::ptr::null_mut();
    };
    // SAFETY: Impeller passes a valid C string naming the requested command.
    match unsafe { CStr::from_ptr(name) }.to_bytes() {
        b"vkGetPhysicalDeviceSurfaceCapabilitiesKHR" => {
            // SAFETY: the loader returned this pointer for exactly this command.
            REAL_SURFACE_CAPABILITIES.get_or_init(|| unsafe {
                std::mem::transmute::<VoidFunction, vk::PFN_vkGetPhysicalDeviceSurfaceCapabilitiesKHR>(real)
            });
            // SAFETY: the instance is the live one Impeller resolves its commands from, and the
            // loader returns this pointer for exactly the named command.
            REAL_SURFACE_PRESENT_MODES.get_or_init(|| unsafe {
                get_instance_proc_addr(
                    instance,
                    c"vkGetPhysicalDeviceSurfacePresentModesKHR".as_ptr(),
                )
                .map(|real| {
                    std::mem::transmute::<
                        VoidFunction,
                        vk::PFN_vkGetPhysicalDeviceSurfacePresentModesKHR,
                    >(real)
                })
            });
            surface_capabilities_with_wanted_extent as *mut c_void
        }
        b"vkCreateSwapchainKHR" => {
            // SAFETY: the loader returned this pointer for exactly this command.
            REAL_CREATE_SWAPCHAIN.get_or_init(|| unsafe {
                std::mem::transmute::<VoidFunction, vk::PFN_vkCreateSwapchainKHR>(real)
            });
            create_swapchain_recording_extent as *mut c_void
        }
        b"vkAcquireNextImageKHR" => {
            // SAFETY: the loader returned this pointer for exactly this command.
            REAL_ACQUIRE_NEXT_IMAGE.get_or_init(|| unsafe {
                std::mem::transmute::<VoidFunction, vk::PFN_vkAcquireNextImageKHR>(real)
            });
            acquire_next_image_unless_stale as *mut c_void
        }
        _ => real as *mut c_void,
    }
}

unsafe extern "system" fn surface_capabilities_with_wanted_extent(
    physical_device: vk::PhysicalDevice,
    surface: vk::SurfaceKHR,
    capabilities: *mut vk::SurfaceCapabilitiesKHR,
) -> vk::Result {
    let Some(real) = REAL_SURFACE_CAPABILITIES.get() else {
        return vk::Result::ERROR_INITIALIZATION_FAILED;
    };
    // SAFETY: arguments come straight from Impeller's call into this command.
    let result = unsafe { real(physical_device, surface, capabilities) };
    if result != vk::Result::SUCCESS {
        return result;
    }
    // SAFETY: on success the driver filled the caller-provided struct.
    let capabilities = unsafe { &mut *capabilities };
    record_driver_min_images(surface, capabilities.min_image_count);
    record_mailbox_support(surface, supports_mailbox(physical_device, surface));
    if capabilities.current_extent.width == u32::MAX
        && let Some(extent) = wanted_extent(surface)
    {
        capabilities.current_extent = extent;
        capabilities.min_image_extent = extent;
        capabilities.max_image_extent = extent;
    }
    result
}

fn supports_mailbox(physical_device: vk::PhysicalDevice, surface: vk::SurfaceKHR) -> bool {
    let Some(Some(real)) = REAL_SURFACE_PRESENT_MODES.get() else {
        return false;
    };
    let mut modes = [vk::PresentModeKHR::FIFO; 16];
    let mut count = modes.len() as u32;
    // SAFETY: `count` holds the capacity of `modes`, which the driver fills up to that many.
    let result = unsafe { real(physical_device, surface, &mut count, modes.as_mut_ptr()) };
    matches!(result, vk::Result::SUCCESS | vk::Result::INCOMPLETE)
        && modes[..count as usize].contains(&vk::PresentModeKHR::MAILBOX)
}

unsafe extern "system" fn create_swapchain_recording_extent(
    device: vk::Device,
    create_info: *const vk::SwapchainCreateInfoKHR<'_>,
    allocator: *const vk::AllocationCallbacks<'_>,
    swapchain: *mut vk::SwapchainKHR,
) -> vk::Result {
    let Some(real) = REAL_CREATE_SWAPCHAIN.get() else {
        return vk::Result::ERROR_INITIALIZATION_FAILED;
    };
    // SAFETY: Impeller passes a create info that is valid for the duration of the call.
    let mut info = unsafe { *create_info };
    info.min_image_count = swapchain_image_count(info.surface, info.min_image_count);
    info.present_mode = swapchain_present_mode(info.surface, info.present_mode);
    // SAFETY: the remaining arguments come straight from Impeller's call into this command.
    let result = unsafe { real(device, &info, allocator, swapchain) };
    if result == vk::Result::SUCCESS {
        // SAFETY: on success the driver wrote the new swapchain.
        record_swapchain(info.surface, unsafe { *swapchain }, info.image_extent);
    }
    result
}

unsafe extern "system" fn acquire_next_image_unless_stale(
    device: vk::Device,
    swapchain: vk::SwapchainKHR,
    timeout: u64,
    semaphore: vk::Semaphore,
    fence: vk::Fence,
    image_index: *mut u32,
) -> vk::Result {
    let Some(real) = REAL_ACQUIRE_NEXT_IMAGE.get() else {
        return vk::Result::ERROR_INITIALIZATION_FAILED;
    };
    if is_stale(swapchain) {
        return vk::Result::ERROR_OUT_OF_DATE_KHR;
    }
    // SAFETY: arguments come straight from Impeller's call into this command.
    unsafe { real(device, swapchain, timeout, semaphore, fence, image_index) }
}
