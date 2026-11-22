use pipewire::{context::ContextBox, core::Core, loop_::Loop, main_loop::MainLoopBox};

use crate::CameraError;

/// Connects to PipeWire on the calling thread and runs `body` with its loop and core.
pub(crate) fn run_on_core<R>(
    body: impl FnOnce(&Loop, &Core) -> Result<R, CameraError>,
) -> Result<R, CameraError> {
    pipewire::init();
    let main_loop = MainLoopBox::new(None).map_err(CameraError::pipewire)?;
    let context = ContextBox::new(main_loop.loop_(), None).map_err(CameraError::pipewire)?;
    let core = context.connect(None).map_err(CameraError::pipewire)?;
    body(main_loop.loop_(), &core)
}
