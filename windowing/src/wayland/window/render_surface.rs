use std::time::Duration;

use geometry::Rect;
use raw_window_handle::{
    DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, WindowHandle,
};
use sctk::reexports::client::{
    Connection, Dispatch, EventQueue, QueueHandle,
    protocol::{
        wl_callback::{self, WlCallback},
        wl_region::{self, WlRegion},
    },
};

use super::SurfaceHandle;
use crate::{
    FrameTiming, Material, PlatformSurface, SurfaceError, SurfaceState,
    wayland::{
        error::Error,
        protocols::background_effects::{BackgroundEffect, Shape},
    },
};

/// Frame-callback state dispatched on the render thread's own queue.
#[derive(Debug, Default)]
pub struct FramePacer {
    frame_pending: bool,
    last_frame_time: Option<Duration>,
    pub(super) presented_at: Option<Duration>,
    pub(super) refresh: Option<Duration>,
}

impl Dispatch<WlCallback, ()> for FramePacer {
    fn event(
        pacer: &mut Self,
        _: &WlCallback,
        event: wl_callback::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let wl_callback::Event::Done { callback_data } = event {
            pacer.frame_pending = false;
            pacer.last_frame_time = Some(Duration::from_millis(u64::from(callback_data)));
        }
    }
}

impl Dispatch<WlRegion, ()> for FramePacer {
    fn event(
        _: &mut Self,
        _: &WlRegion,
        _: wl_region::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

/// Paces a render thread by the compositor's frame callbacks and commits per-frame
/// surface state, without going through the event-loop thread.
pub struct RenderSurface {
    handle: SurfaceHandle,
    queue: EventQueue<FramePacer>,
    pacer: FramePacer,
    material: Material,
}

impl RenderSurface {
    pub(crate) fn new(handle: SurfaceHandle) -> Self {
        let queue = handle.connection().new_event_queue();
        Self {
            handle,
            queue,
            pacer: FramePacer::default(),
            material: Material::default(),
        }
    }

    pub fn handle(&self) -> &SurfaceHandle {
        &self.handle
    }

    pub fn frame_pending(&self) -> bool {
        self.pacer.frame_pending
    }

    /// Blocks until the pending frame callback fires; returns at once when none is pending.
    pub fn wait_for_frame(&mut self) -> Result<Option<Duration>, Error> {
        self.queue.dispatch_pending(&mut self.pacer)?;
        while self.pacer.frame_pending {
            self.queue.blocking_dispatch(&mut self.pacer)?;
        }
        Ok(self.pacer.last_frame_time)
    }

    fn region(&self, rects: &[Rect]) -> WlRegion {
        let region = self
            .handle
            .compositor
            .create_region(&self.queue.handle(), ());
        for rect in rects {
            region.add(
                rect.min_x().round() as i32,
                rect.min_y().round() as i32,
                rect.size.width.round() as i32,
                rect.size.height.round() as i32,
            );
        }
        region
    }
}

impl PlatformSurface for RenderSurface {
    fn pre_present_notify(&mut self) {
        if let Some(presentation) = &self.handle.presentation {
            presentation.feedback(self.handle.wl_surface(), &self.queue.handle(), ());
        }
        if std::mem::replace(&mut self.pacer.frame_pending, true) {
            return;
        }
        self.handle.wl_surface().frame(&self.queue.handle(), ());
    }

    fn wait_for_frame(&mut self) -> Result<FrameTiming, SurfaceError> {
        RenderSurface::wait_for_frame(self)
            .map(|callback_time| FrameTiming {
                callback_time,
                presented_at: self.pacer.presented_at,
                refresh: self.pacer.refresh,
            })
            .map_err(|error| SurfaceError::Disconnected(error.to_string()))
    }

    fn apply(&mut self, state: &SurfaceState) {
        let surface = self.handle.wl_surface();
        if state.damage.is_empty() {
            surface.damage(0, 0, i32::MAX, i32::MAX);
        }
        for rect in &state.damage {
            let snapped = rect.snap_out(1.0);
            surface.damage(
                snapped.min_x() as i32,
                snapped.min_y() as i32,
                snapped.size.width as i32,
                snapped.size.height as i32,
            );
        }
        if let Some(material) = state.material {
            self.material = material;
        }
        if let (Some(effect), true) = (
            &self.handle.effect,
            state.material.is_some() || state.material_shapes.is_some(),
        ) {
            let shape = state
                .material_shapes
                .as_deref()
                .map(Shape::from_material_shapes);
            effect.apply(&BackgroundEffect::from_material(&self.material, shape));
        }
        if let Some(rects) = &state.input_region {
            let region = self.region(rects);
            surface.set_input_region(Some(&region));
            region.destroy();
        }
        if let Some(rects) = &state.opaque_region {
            let region = self.region(rects);
            surface.set_opaque_region(Some(&region));
            region.destroy();
        }
    }
}

impl HasWindowHandle for RenderSurface {
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        self.handle.window_handle()
    }
}

impl HasDisplayHandle for RenderSurface {
    fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
        self.handle.display_handle()
    }
}
