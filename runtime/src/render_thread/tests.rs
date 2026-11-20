use std::{
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

use geometry::{CornerShape, TRANSPARENT};
use graphics::{GraphicsBackend, RecordingBackend};
use windowing::{
    FrameTiming, FrameWaitStopper, PlatformSurface, SurfaceError, SurfaceGeometry, SurfaceState,
    dpi::{LogicalSize, ScaleFactor},
};

use super::{RenderMessage, RenderSettings, RenderThread};
use crate::FramePacing;

const PROMPTLY: Duration = Duration::from_secs(2);

/// A surface on a gone output: after `frames_before_output_loss` frames its frame callbacks
/// never come, and only its stopper ends the wait.
struct SurfaceOnRemovedOutput {
    frames_before_output_loss: usize,
    waiting: mpsc::Sender<()>,
    stop: mpsc::Receiver<()>,
    stopper: mpsc::Sender<()>,
}

impl SurfaceOnRemovedOutput {
    fn new(frames_before_output_loss: usize) -> (Self, mpsc::Receiver<()>) {
        let (waiting, waits) = mpsc::channel();
        let (stopper, stop) = mpsc::channel();
        let surface = Self {
            frames_before_output_loss,
            waiting,
            stop,
            stopper,
        };
        (surface, waits)
    }
}

impl PlatformSurface for SurfaceOnRemovedOutput {
    fn pre_present_notify(&mut self) {}

    fn wait_for_frame(&mut self) -> Result<FrameTiming, SurfaceError> {
        if let Some(remaining) = self.frames_before_output_loss.checked_sub(1) {
            self.frames_before_output_loss = remaining;
            return Ok(FrameTiming::default());
        }
        let _ = self.waiting.send(());
        let _ = self.stop.recv();
        Err(SurfaceError::Stopped)
    }

    fn apply(&mut self, _: &SurfaceState) {}

    fn take_frame_wait_stopper(&mut self) -> FrameWaitStopper {
        let stopper = self.stopper.clone();
        Box::new(move || {
            let _ = stopper.send(());
        })
    }
}

type TextLayout = <RecordingBackend as GraphicsBackend>::TextLayout;

/// Starts a render thread on `surface`, sends `messages`, and drops the thread once it waits
/// for a frame that never comes. Returns how long the drop took, or `None` when the thread
/// never waited or the drop did not return in time.
fn drop_while_waiting_for_a_lost_frame(
    frames_before_output_loss: usize,
    messages: impl FnOnce() -> Vec<RenderMessage<TextLayout>> + Send + 'static,
) -> Option<Duration> {
    let (dropped_sender, dropped) = mpsc::channel();
    thread::spawn(move || {
        let (surface, waits) = SurfaceOnRemovedOutput::new(frames_before_output_loss);
        let settings = RenderSettings {
            clear_color: TRANSPARENT,
            corners: CornerShape::default(),
            pacing: FramePacing::Display,
        };
        let Ok(render) = RenderThread::spawn::<RecordingBackend, _>(
            surface,
            settings,
            Instant::now(),
            Box::new(|| {}),
        ) else {
            return;
        };
        for message in messages() {
            render.send(message);
        }
        if waits.recv_timeout(PROMPTLY).is_err() {
            return;
        }
        let drop_started = Instant::now();
        drop(render);
        let _ = dropped_sender.send(drop_started.elapsed());
    });
    dropped.recv_timeout(PROMPTLY).ok()
}

fn viewport() -> RenderMessage<TextLayout> {
    RenderMessage::Viewport(SurfaceGeometry {
        size: LogicalSize::new(64, 32),
        scale: ScaleFactor::IDENTITY,
    })
}

#[test]
fn dropping_a_render_thread_waiting_to_draw_a_frame_joins_promptly() {
    let took = drop_while_waiting_for_a_lost_frame(0, || vec![viewport()]);
    assert!(took.is_some_and(|took| took < PROMPTLY), "{took:?}");
}

#[test]
fn dropping_a_render_thread_pacing_ui_animation_joins_promptly() {
    let took = drop_while_waiting_for_a_lost_frame(1, || {
        vec![viewport(), RenderMessage::AnimationFrames(true)]
    });
    assert!(took.is_some_and(|took| took < PROMPTLY), "{took:?}");
}
