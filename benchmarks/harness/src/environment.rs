use std::{process::Command, str::FromStr};

use anyhow::bail;
use serde::Serialize;

use crate::{frame_rate::FrameRate, scenario::Scenario, spec::Spec};

/// The crownpositor variable that picks where a client's windows map.
const PLACEMENT_VARIABLE: &str = "WORKSPACE";

/// Where the app windows open.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Placement {
    /// crownpositor's hidden output (60 Hz, scale 1): out of the user's way and the same on
    /// every run, but the screen must still be unlocked.
    Headless,
    /// The focused workspace of the real display, at its refresh rate and scale.
    Desktop,
}

impl Placement {
    pub fn name(self) -> &'static str {
        match self {
            Self::Headless => "headless",
            Self::Desktop => "desktop",
        }
    }
}

impl FromStr for Placement {
    type Err = anyhow::Error;

    fn from_str(name: &str) -> anyhow::Result<Self> {
        match name {
            "headless" => Ok(Self::Headless),
            "desktop" => Ok(Self::Desktop),
            other => bail!("unknown placement {other:?}; expected headless or desktop"),
        }
    }
}

/// Pins every app to the Mesa driver of the display GPU (the AMD iGPU driving the panel), the
/// one the compositor composes on, so no app pays for a cross-GPU copy and none gets a faster
/// discrete GPU.
const MESA_VULKAN_DRIVER: &str = "/usr/share/vulkan/icd.d/radeon_icd.json";
const MESA_EGL_VENDOR: &str = "/usr/share/glvnd/egl_vendor.d/50_mesa.json";

const REMOVED: [&str; 5] = [
    "DISPLAY",
    "DRI_PRIME",
    "__NV_PRIME_RENDER_OFFLOAD",
    "__GLX_VENDOR_LIBRARY_NAME",
    "NODE_OPTIONS",
];

/// Native Wayland for everyone, with the protocol trace the frame counter reads. `DISPLAY` is
/// removed, so a toolkit that cannot speak Wayland fails instead of quietly using XWayland.
pub fn apply(
    command: &mut Command,
    spec: &Spec,
    scenario: Scenario,
    placement: Placement,
    frame_rate: FrameRate,
) {
    for name in REMOVED {
        command.env_remove(name);
    }
    match placement {
        Placement::Headless => command.env(PLACEMENT_VARIABLE, "headless"),
        Placement::Desktop => command.env_remove(PLACEMENT_VARIABLE),
    };
    command
        .env("WAYLAND_DEBUG", "client")
        .env("QT_QPA_PLATFORM", "wayland")
        .env("GDK_BACKEND", "wayland")
        .env("ELECTRON_OZONE_PLATFORM_HINT", "wayland")
        .env("VK_DRIVER_FILES", MESA_VULKAN_DRIVER)
        .env("__EGL_VENDOR_LIBRARY_FILENAMES", MESA_EGL_VENDOR)
        .envs(spec.environment(scenario));
    if frame_rate.is_uncapped() {
        command.env("BENCH_UNCAPPED", "1");
    }
}
