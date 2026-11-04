use std::{
    thread::JoinHandle,
    time::{Duration, Instant},
};

use crossbeam_channel::{Receiver, Sender};
use geometry::{Color, CornerShape};
use graphics::{CreateRenderer, GraphicsBackend, Renderer};
use windowing::{PlatformSurface, SurfaceState, Waker};

use crate::{
    batch::RenderBatch, frame_clock::FrameClock, render::RenderTree, render_inbox::RenderInbox,
};

/// Messages from the UI thread.
pub(crate) enum RenderMessage<T> {
    Batch(Box<RenderBatch<T>>),
    Viewport {
        width: u32,
        height: u32,
        scale: f32,
    },
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

/// The UI thread's handle to a running render thread.
pub(crate) struct RenderThread<T> {
    sender: Sender<RenderMessage<T>>,
    reports: Receiver<RenderReport>,
    thread: Option<JoinHandle<()>>,
}

impl<T: Send + 'static> RenderThread<T> {
    /// Moves `surface` to a new thread that creates a `G` renderer for it, drawing corners as
    /// `corners`, and renders whatever the UI sends, pacing frames by the surface and running
    /// animations on its own clock. After each frame it wakes the UI thread through `waker`
    /// while the UI waits for animation frames.
    pub(crate) fn spawn<G, S>(
        surface: S,
        clear_color: Color,
        corners: CornerShape,
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
                render_loop::<G, S>(
                    surface,
                    receiver,
                    report_sender,
                    clear_color,
                    corners,
                    epoch,
                    waker,
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
    renderer.set_corner_shape(corners);
    let mut tree = RenderTree::new(clear_color);
    let mut inbox = RenderInbox::default();
    let mut clock = FrameClock::default();
    loop {
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
        let timing = match surface.wait_for_frame() {
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
