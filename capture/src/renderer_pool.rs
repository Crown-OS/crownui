use std::{cell::Cell, mem::ManuallyDrop};

use impeller::ImpellerRenderer;

/// An idle offscreen renderer and the framebuffer size it was made for.
pub(crate) struct SpareRenderer {
    pub(crate) size: (u32, u32),
    pub(crate) gpu: ImpellerRenderer,
}

thread_local! {
    /// Renderers of unmounted apps, reused by the next mount of the same size. Never dropped:
    /// NVIDIA's `eglDestroyContext` crashes once several headless contexts have drawn, so
    /// captures keep one detached context per window size for the life of the process.
    static SPARE: ManuallyDrop<Cell<Vec<SpareRenderer>>> = const {
        ManuallyDrop::new(Cell::new(Vec::new()))
    };
}

/// Takes an idle renderer of `size`, if any.
pub(crate) fn take_spare(size: (u32, u32)) -> Option<ImpellerRenderer> {
    SPARE.with(|spare| {
        let mut renderers = spare.take();
        let found = renderers
            .iter()
            .position(|renderer| renderer.size == size)
            .map(|index| renderers.swap_remove(index).gpu);
        spare.set(renderers);
        found
    })
}

/// Keeps `gpu` for the next mount of `size`, detached from the thread.
pub(crate) fn keep_spare(size: (u32, u32), gpu: ImpellerRenderer) {
    if let Err(error) = gpu.release_context() {
        eprintln!("capture: detaching an idle GL context failed: {error}");
    }
    SPARE.with(|spare| {
        let mut renderers = spare.take();
        renderers.push(SpareRenderer { size, gpu });
        spare.set(renderers);
    });
}
