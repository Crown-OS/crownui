use std::{env, fs};

use crate::util::device_number::{major, minor};

const LOADER_DRIVERS_SELECT: &str = "VK_LOADER_DRIVERS_SELECT";

const USER_DRIVER_CHOICES: [&str; 8] = [
    LOADER_DRIVERS_SELECT,
    "VK_LOADER_DRIVERS_DISABLE",
    "VK_DRIVER_FILES",
    "VK_ICD_FILENAMES",
    "VK_ADD_DRIVER_FILES",
    "DRI_PRIME",
    "MESA_VK_DEVICE_SELECT",
    "__NV_PRIME_RENDER_OFFLOAD",
];

const MANIFESTS_BY_KERNEL_DRIVER: [(&str, &str); 12] = [
    ("amdgpu", "*radeon*,*amd*"),
    ("radeon", "*radeon*"),
    ("i915", "*intel*"),
    ("xe", "*intel*"),
    ("nvidia", "*nvidia*"),
    ("nouveau", "*nouveau*"),
    ("msm", "*freedreno*"),
    ("panfrost", "*panfrost*"),
    ("panthor", "*panfrost*"),
    ("v3d", "*broadcom*"),
    ("asahi", "*asahi*"),
    ("virtio_gpu", "*virtio*"),
];

/// Limits the Vulkan loader to the driver of `main_device` (a Linux `dev_t`, e.g. the
/// compositor's GPU), so drivers of other GPUs are never loaded and those GPUs stay asleep.
///
/// Does nothing when the user already chose a driver or GPU through the environment, or when
/// the device's kernel driver has no known Vulkan driver.
///
/// # Safety
/// Writes the process environment: no other thread may read or write it at the same time.
pub unsafe fn select_vulkan_driver(main_device: u64) {
    if USER_DRIVER_CHOICES
        .iter()
        .any(|name| env::var_os(name).is_some())
    {
        return;
    }
    let Some(manifests) = manifest_globs(main_device) else {
        return;
    };
    tracing::debug!(
        manifests,
        "limiting the Vulkan loader to the main device's driver"
    );
    // SAFETY: the caller guarantees no other thread touches the environment concurrently.
    unsafe { env::set_var(LOADER_DRIVERS_SELECT, manifests) };
}

fn manifest_globs(device: u64) -> Option<&'static str> {
    let driver_link = format!(
        "/sys/dev/char/{}:{}/device/driver",
        major(device),
        minor(device)
    );
    let driver = fs::read_link(driver_link).ok()?;
    let driver_name = driver.file_name()?.to_str()?;
    MANIFESTS_BY_KERNEL_DRIVER
        .iter()
        .find(|(kernel_driver, _)| *kernel_driver == driver_name)
        .map(|(_, manifests)| *manifests)
}
