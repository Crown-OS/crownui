use std::{
    thread::JoinHandle,
    time::{Duration, Instant},
};

use crossbeam_channel::{Receiver, Sender, TryRecvError};
use geometry::{Color, CornerShape};
use graphics::{CreateRenderer, GraphicsBackend, Renderer};
use windowing::{PlatformSurface, SurfaceState};

use crate::{batch::RenderBatch, render::RenderTree};

/// Messages from the UI thread.
pub(crate) enum RenderMessage<T> {
    Batch(Box<RenderBatch<T>>),
    Viewport { width: u32, height: u32, scale: f32 },
    Paused(bool),
    Shutdown,
}

/// Timings the render thread reports back for one frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RenderReport {
    pub(crate) frame: u64,
    pub(crate) record_us: u32,
    pub(crate) render_us: u32,
    /// The display's refresh interval, when the platform reports one.
    pub(crate) refresh: Option<Duration>,
}

/// The UI thread's handle to a running render thread.
pub(crate) struct RenderThread<T> {
    sender: Sender<RenderMessage<T>>,
    reports: Receiver<RenderReport>,
    thread: Option<JoinHandle<()>>,
}

impl<T: Send + 'static> RenderThread<T> {
    /// Moves `surface` to a new thread that creates a `G` renderer for it, drawing corners as
    /// `corners`, and renders whatever the UI sends, pacing frames by the surface and running
    /// animations on its own clock.
    pub(crate) fn spawn<G, S>(
        surface: S,
        clear_color: Color,
        corners: CornerShape,
        epoch: Instant,
    ) -> std::io::Result<Self>
    where
        G: GraphicsBackend<TextLayout = T> + CreateRenderer<S>,
        S: PlatformSurface,
    {
        let (sender, receiver) = crossbeam_channel::unbounded();
        let (report_sender, reports) = crossbeam_channel::bounded(256);
        let thread = std::thread::Builder::new()
            .name("crownui-render".into())
            .spawn(move || {
                render_loop::<G, S>(
                    surface,
                    receiver,
                    report_sender,
                    clear_color,
                    corners,
                    epoch,
                )
            })?;
        Ok(Self {
            sender,
            reports,
            thread: Some(thread),
        })
    }

    /// Frame reports received since the last call.
    pub(crate) fn reports(&self) -> impl Iterator<Item = RenderReport> + '_ {
        self.reports.try_iter()
    }

    /// Sends a message; returns `false` when the render thread is gone.
    pub(crate) fn send(&self, message: RenderMessage<T>) -> bool {
        self.sender.send(message).is_ok()
    }
}

impl<T> Drop for RenderThread<T> {
    fn drop(&mut self) {
        let _ = self.sender.send(RenderMessage::Shutdown);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

fn render_loop<G, S>(
    mut surface: S,
    receiver: Receiver<RenderMessage<G::TextLayout>>,
    reports: Sender<RenderReport>,
    clear_color: Color,
    corners: CornerShape,
    epoch: Instant,
) where
    G: GraphicsBackend + CreateRenderer<S>,
    S: PlatformSurface,
{
    let mut renderer = match G::create_renderer(&surface) {
        Ok(renderer) => renderer,
        Err(error) => {
            tracing::error!("CUI0402: could not create the renderer: {error}");
            return;
        }
    };
    renderer.set_corner_shape(corners);
    let mut tree = RenderTree::new(clear_color);
    let mut pending_surface: Option<SurfaceState> = None;
    let mut dirty = false;
    let mut has_viewport = false;
    let mut paused = false;
    loop {
        let animating = tree.is_animating();
        let mut messages = Vec::new();
        if paused || !(dirty || animating) {
            match receiver.recv() {
                Ok(message) => messages.push(message),
                Err(_) => break,
            }
        }
        loop {
            match receiver.try_recv() {
                Ok(message) => messages.push(message),
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => return,
            }
        }
        let now = epoch.elapsed();
        for message in messages {
            match message {
                RenderMessage::Shutdown => return,
                RenderMessage::Paused(pause) => {
                    tracing::debug!(pause, "render paused");
                    paused = pause;
                    dirty = true;
                }
                RenderMessage::Viewport {
                    width,
                    height,
                    scale,
                } => {
                    tracing::debug!(width, height, scale, "render viewport");
                    renderer.set_viewport(width, height, scale);
                    has_viewport = true;
                    dirty = true;
                }
                RenderMessage::Batch(batch) => {
                    let (ops, surface_state, releases) = batch.upload(&mut renderer);
                    if let Some(state) = surface_state {
                        merge_surface_state(&mut pending_surface, state);
                    }
                    tree.apply(ops, now, &mut renderer);
                    tree.release_when_unused(releases);
                    dirty = true;
                }
            }
        }
        if paused || !has_viewport || !(dirty || tree.is_animating()) {
            continue;
        }
        let refresh = match surface.wait_for_frame() {
            Ok(timing) => timing.refresh,
            Err(error) => {
                tracing::error!("CUI0403: surface lost: {error}");
                return;
            }
        };
        let now = epoch.elapsed() + Duration::from_millis(1);
        let record_started = Instant::now();
        let (composition, stats) = tree.prepare(now, &mut renderer);
        let record_us = crate::frame_stats::micros(record_started.elapsed());
        let frame = tree.frame_count();
        if frame <= 3 || frame.is_power_of_two() {
            tracing::debug!(
                frame,
                nodes = stats.nodes,
                recorded = stats.recorded,
                animating = stats.animating,
                "render frame"
            );
        }
        if let Some(state) = pending_surface.take() {
            surface.apply(&state);
        }
        surface.pre_present_notify();
        let render_started = Instant::now();
        if let Err(error) = renderer.render(&composition) {
            tracing::error!("CUI0404: frame failed: {error}");
        }
        let _ = reports.try_send(RenderReport {
            frame,
            record_us,
            render_us: crate::frame_stats::micros(render_started.elapsed()),
            refresh,
        });
        dirty = false;
    }
}

fn merge_surface_state(pending: &mut Option<SurfaceState>, next: SurfaceState) {
    match pending {
        None => *pending = Some(next),
        Some(current) => {
            current.damage.extend(next.damage);
            if next.material.is_some() {
                current.material = next.material;
            }
            if next.material_shapes.is_some() {
                current.material_shapes = next.material_shapes;
            }
            if next.input_region.is_some() {
                current.input_region = next.input_region;
            }
            if next.opaque_region.is_some() {
                current.opaque_region = next.opaque_region;
            }
        }
    }
}
