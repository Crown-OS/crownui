use std::thread::{self, JoinHandle};

use crossbeam_channel::{Receiver, Sender, TrySendError, select};
use media::VideoFrame;
use tokio::sync::watch;

use crate::{
    CameraError, CameraState,
    capture::{CaptureEvent, EncodedFrame},
    decode::Decoder,
};

const THREAD_NAME: &str = "crownui-camera-decode";

/// Asks the decoding thread for something.
pub(crate) enum Command {
    /// Sends every frame from now on to this tap too, until its receiver is dropped.
    Tap(Sender<VideoFrame>),
}

/// The ends the decoding thread works between.
pub(crate) struct ProcessingLinks {
    pub(crate) events: Receiver<CaptureEvent>,
    pub(crate) recycle: Sender<Vec<u8>>,
    pub(crate) commands: Receiver<Command>,
    pub(crate) state: watch::Sender<CameraState>,
}

/// Decodes captured frames off the PipeWire thread and hands them to the preview and every tap.
/// Ends when the capture thread does.
pub(crate) fn spawn(
    links: ProcessingLinks,
    preview: impl FnMut(VideoFrame) + Send + 'static,
) -> Result<JoinHandle<()>, CameraError> {
    let decoder = Decoder::new().map_err(|error| CameraError::PipeWire(error.to_string()))?;
    thread::Builder::new()
        .name(THREAD_NAME.into())
        .spawn(move || Processing::new(decoder, links).run(preview))
        .map_err(CameraError::Thread)
}

struct Processing {
    decoder: Decoder,
    links: ProcessingLinks,
    taps: Vec<Sender<VideoFrame>>,
    warned: bool,
}

impl Processing {
    fn new(decoder: Decoder, links: ProcessingLinks) -> Self {
        Self {
            decoder,
            links,
            taps: Vec::new(),
            warned: false,
        }
    }

    fn run(mut self, mut preview: impl FnMut(VideoFrame)) {
        loop {
            select! {
                recv(self.links.events) -> event => match event {
                    Ok(CaptureEvent::Frame(frame)) => self.process(frame, &mut preview),
                    Ok(CaptureEvent::Failed(reason)) => {
                        self.links.state.send_replace(CameraState::Failed(reason));
                    }
                    Err(_) => return,
                },
                recv(self.links.commands) -> command => match command {
                    Ok(Command::Tap(tap)) => self.taps.push(tap),
                    Err(_) => self.links.commands = crossbeam_channel::never(),
                },
            }
        }
    }

    fn process(&mut self, encoded: EncodedFrame, preview: &mut impl FnMut(VideoFrame)) {
        let decoded = self.decoder.decode(&encoded);
        let format = encoded.format;
        let _ = self.links.recycle.try_send(encoded.bytes);
        let frame = match decoded {
            Ok(frame) => frame,
            Err(error) => {
                if !std::mem::replace(&mut self.warned, true) {
                    tracing::warn!("skipping camera frames that do not decode: {error}");
                }
                return;
            }
        };
        self.links.state.send_if_modified(|state| {
            let live = CameraState::Live(format);
            let changed = *state != live;
            *state = live;
            changed
        });
        self.taps.retain(|tap| {
            !matches!(
                tap.try_send(frame.clone()),
                Err(TrySendError::Disconnected(_))
            )
        });
        preview(frame);
    }
}
