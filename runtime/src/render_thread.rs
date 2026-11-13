use std::{
    thread::JoinHandle,
    time::{Duration, Instant},
};

use crossbeam_channel::{Receiver, Sender};
use geometry::{Color, CornerShape};
use graphics::{CreateRenderer, GraphicsBackend, Renderer};
use windowing::{PlatformSurface, SurfaceGeometry, SurfaceState, Waker};

use crate::{
    FramePacing, batch::RenderBatch, frame_clock::FrameClock, render::RenderTree,
    render_inbox::RenderInbox,
};

/// Messages from the UI thread.
pub(crate) enum RenderMessage<T> {
    Batch(Box<RenderBatch<T>>),
    Viewport(SurfaceGeometry),
    Paused(bool),
    /// Whether the UI thread has animation-frame callbacks waiting, and so wants to be woken
    /// after each frame to run them in step with the display.
    AnimationFrames(bool),
    Shutdown,
}

/// Timings the render thread reports back for one frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RenderReport {
    pub(crate) frame: u64,
    pub(crate) record_us: u32,
    pub(crate) render_us: u32,
    /// The time the frame's animations were sampled at.
    pub(crate) sampled_at: Duration,
    /// The display's refresh interval, when the platform reports one.
    pub(crate) refresh: Option<Duration>,
}

/// How a window's render thread draws: the color it clears to, the curve of its rounded corners
/// and when it draws the next frame.
#[derive(Debug, Clone, Copy)]
pub(crate) struct RenderSettings {
    pub(crate) clear_color: Color,
    pub(crate) corners: CornerShape,
    pub(crate) pacing: FramePacing,
}

/// The UI thread's handle to a running render thread.
pub(crate) struct RenderThread<T> {
    sender: Sender<RenderMessage<T>>,
    reports: Receiver<RenderReport>,
    thread: Option<JoinHandle<()>>,
}

impl<T: Send + 'static> RenderThread<T> {
    /// Moves `surface` to a new thread that creates a `G` renderer for it and renders whatever the
    /// UI sends as `settings` say, running animations on its own clock. After each frame it wakes the UI thread through `waker`
    /// while the UI waits for animation frames.
    pub(crate) fn spawn<G, S>(
        surface: S,
        settings: RenderSettings,
        epoch: Instant,
        waker: Waker,
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
                render_loop::<G, S>(surface, receiver, report_sender, settings, epoch, waker)
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
    settings: RenderSettings,
    epoch: Instant,
    waker: Waker,
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
    renderer.set_corner_shape(settings.corners);
    let mut tree = RenderTree::new(settings.clear_color);
    let mut inbox = RenderInbox::default();
    let mut clock = FrameClock::default();
    loop {
        let animating_ui = inbox.ui_waits_for_frames
            && inbox.has_viewport
            && !inbox.paused
            && settings.pacing == FramePacing::Display;
        // While the UI animates, its next batch lands right after each frame: sleeping until the
        // display wants a frame takes one wakeup per frame instead of two.
        let mut waited_for_display = None;
        if animating_ui {
            match surface.wait_for_frame() {
                Ok(timing) => waited_for_display = Some(timing),
                Err(error) => {
                    tracing::error!("CUI0403: surface lost: {error}");
                    return;
                }
            }
        }
        if !inbox.wants_frame(&tree) {
            let Ok(message) = receiver.recv() else {
                return;
            };
            if inbox
                .receive(message, &mut tree, &mut renderer, epoch.elapsed())
                .is_break()
            {
                return;
            }
        }
        if inbox
            .drain(&receiver, &mut tree, &mut renderer, epoch.elapsed())
            .is_break()
        {
            return;
        }
        if !inbox.has_viewport || !inbox.wants_frame(&tree) {
            continue;
        }
        let timing = match (waited_for_display, settings.pacing) {
            (Some(timing), _) => Ok(timing),
            (None, FramePacing::Display) => surface.wait_for_frame(),
            (None, FramePacing::Unpaced) => surface.poll_frame(),
        };
        let timing = match timing {
            Ok(timing) => timing,
            Err(error) => {
                tracing::error!("CUI0403: surface lost: {error}");
                return;
            }
        };
        if inbox
            .drain(&receiver, &mut tree, &mut renderer, epoch.elapsed())
            .is_break()
        {
            return;
        }
        let now = clock.next(epoch.elapsed() + Duration::from_millis(1), timing);
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
        if let Some(state) = inbox.pending_surface.take() {
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
            sampled_at: now,
            refresh: timing.refresh,
        });
        if inbox.ui_waits_for_frames {
            waker();
        }
        inbox.dirty = false;
    }
}

pub(crate) fn merge_surface_state(pending: &mut Option<SurfaceState>, next: SurfaceState) {
    match pending {
        None => *pending = Some(next),
        Some(current) => {
            if next.geometry.is_some() {
                current.geometry = next.geometry;
            }
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
            if next.keyboard.is_some() {
                current.keyboard = next.keyboard;
            }
        }
    }
}
