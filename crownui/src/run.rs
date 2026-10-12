#[cfg(all(feature = "wayland", feature = "impeller"))]
use runtime::{AppConfig, RunError, Runner};
#[cfg(all(feature = "wayland", feature = "impeller"))]
use view::View;

/// Runs `root` in a window on Wayland, rendered by Impeller and laid out by taffy, until the
/// window closes.
///
/// Call it from `main` before starting other threads: it limits the Vulkan loader to the
/// compositor's GPU driver through the process environment.
#[cfg(all(feature = "wayland", feature = "impeller"))]
pub fn run(root: impl View, config: AppConfig) -> Result<(), RunError> {
    let mut runner = Runner::<impeller::ImpellerBackend, taffy_layout::TaffyEngine>::new(
        root,
        taffy_layout::TaffyEngine::new(),
        config,
    )?;
    let event_loop = windowing::wayland::EventLoop::new()
        .map_err(|error| RunError::Platform(error.to_string()))?;
    if let Some(main_device) = event_loop.main_device() {
        // SAFETY: CrownUI has started no threads yet and `run` requires the app to have none.
        unsafe { impeller::select_vulkan_driver(main_device) };
    }
    event_loop
        .run_app(&mut runner)
        .map_err(|error| RunError::Platform(error.to_string()))?;
    match runner.take_failure() {
        Some(failure) => Err(failure),
        None => Ok(()),
    }
}
