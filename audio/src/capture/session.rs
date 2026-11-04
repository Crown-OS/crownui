use std::{ops::ControlFlow, sync::mpsc, time::Instant};

use pipewire::{core::Core, loop_::Loop};
use tokio::sync::watch;

use crate::{Enhancement, MicrophoneOptions, SourceId, StreamState, UnenhancedReason};

use super::{
    event::{Command, Event},
    frame_sink::{FrameSink, SinkSlot},
    pipewire_capture::{Capture, StatusReporter},
    status::{CaptureStatus, Route, stream_state},
    voice_link::VoiceLink,
};

/// The capture thread's state: which node it captures, the crown-voiced link enhancing it, and
/// the [`StreamState`] that follows from both.
pub(super) struct Session<'l> {
    loop_: &'l Loop,
    core: &'l Core,
    format: &'l [u8],
    events: mpsc::Sender<Event>,
    state: &'l watch::Sender<StreamState>,
    spare_sink: mpsc::Receiver<FrameSink>,
    sink_home: mpsc::Sender<FrameSink>,
    source: Option<SourceId>,
    enhancement: Enhancement,
    capture: Option<(Option<String>, Capture<'l>)>,
    generation: u32,
    status: CaptureStatus,
    voice: Option<VoiceLink<'l>>,
    route: Route,
}

impl<'l> Session<'l> {
    pub(super) fn new(
        loop_: &'l Loop,
        core: &'l Core,
        format: &'l [u8],
        events: mpsc::Sender<Event>,
        state: &'l watch::Sender<StreamState>,
        options: MicrophoneOptions,
        sink: FrameSink,
    ) -> Self {
        let (sink_home, spare_sink) = mpsc::channel();
        let _ = sink_home.send(sink);
        Self {
            loop_,
            core,
            format,
            events,
            state,
            spare_sink,
            sink_home,
            source: options.source,
            enhancement: options.enhancement,
            capture: None,
            generation: 0,
            status: CaptureStatus::Waiting,
            voice: None,
            route: Route::Raw,
        }
    }

    pub(super) fn start(&mut self) {
        if self.enhancement.is_active() {
            self.request_enhancement();
        } else {
            self.capture_raw();
        }
        self.publish();
    }

    pub(super) fn handle(&mut self, event: Event) -> ControlFlow<()> {
        match event {
            Event::Command(Command::Stop) => return ControlFlow::Break(()),
            Event::Command(Command::SetEnhancement(enhancement)) => {
                self.set_enhancement(enhancement);
            }
            Event::Capture { generation, status } if generation == self.generation => {
                self.status = status;
            }
            Event::Capture { .. } => {}
            Event::VoiceReadable => self.pump_voice(),
        }
        self.publish();
        ControlFlow::Continue(())
    }

    pub(super) fn deadline(&self) -> Option<Instant> {
        self.voice.as_ref().and_then(VoiceLink::deadline)
    }

    pub(super) fn check_deadline(&mut self) {
        if self
            .deadline()
            .is_some_and(|deadline| deadline <= Instant::now())
        {
            self.fall_back(UnenhancedReason::TimedOut);
            self.publish();
        }
    }

    fn set_enhancement(&mut self, enhancement: Enhancement) {
        self.enhancement = enhancement;
        if !enhancement.is_active() {
            self.voice = None;
            self.route = Route::Raw;
            self.capture_raw();
            return;
        }
        match self
            .voice
            .as_mut()
            .map(|link| link.set_enhancement(enhancement))
        {
            Some(Ok(())) => {}
            Some(Err(reason)) => self.fall_back(reason),
            None => self.request_enhancement(),
        }
    }

    fn request_enhancement(&mut self) {
        match VoiceLink::open(
            self.loop_,
            &self.events,
            self.source.as_ref(),
            self.enhancement,
        ) {
            Ok(link) => self.voice = Some(link),
            Err(reason) => self.fall_back(reason),
        }
    }

    fn pump_voice(&mut self) {
        match self.voice.as_mut().map(VoiceLink::pump) {
            Some(Ok(Some(node))) => {
                self.route = Route::Enhanced {
                    backend: node.backend,
                };
                self.connect(Some(node.node_name));
            }
            Some(Err(reason)) => self.fall_back(reason),
            Some(Ok(None)) | None => {}
        }
    }

    fn fall_back(&mut self, reason: UnenhancedReason) {
        tracing::info!("capturing the raw microphone: {reason}");
        self.voice = None;
        self.route = Route::Fallback(reason);
        self.capture_raw();
    }

    fn capture_raw(&mut self) {
        let target = self
            .source
            .as_ref()
            .map(|source| source.as_str().to_owned());
        self.connect(target);
    }

    fn connect(&mut self, target: Option<String>) {
        if self
            .capture
            .as_ref()
            .is_some_and(|(current, _)| *current == target)
        {
            return;
        }
        self.capture = None;
        self.generation = self.generation.wrapping_add(1);
        self.status = CaptureStatus::Waiting;
        let Ok(sink) = self.spare_sink.try_recv() else {
            self.status = CaptureStatus::Failed("the frame ring was lost".into());
            return;
        };
        let reporter = StatusReporter {
            events: self.events.clone(),
            generation: self.generation,
        };
        match Capture::connect(
            self.core,
            target.as_deref(),
            self.format,
            SinkSlot::new(sink, self.sink_home.clone()),
            reporter,
        ) {
            Ok(capture) => self.capture = Some((target, capture)),
            Err(error) => self.status = CaptureStatus::Failed(error.to_string()),
        }
    }

    fn publish(&self) {
        let next = stream_state(&self.status, &self.route);
        self.state.send_if_modified(|current| {
            let changed = *current != next;
            if changed {
                *current = next;
            }
            changed
        });
    }
}
