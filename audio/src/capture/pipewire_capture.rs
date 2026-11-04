use std::sync::mpsc;

use pipewire::{
    self as pw,
    core::Core,
    properties::properties,
    spa::{pod::Pod, utils::Direction},
    stream::{Stream, StreamBox, StreamFlags, StreamListener},
};

use crate::{ClockNanos, util::property_value::property_value};

use super::{event::Event, frame_sink::SinkSlot, status::CaptureStatus};

const STREAM_NAME: &str = "crownui-microphone";
const TEN_MS_QUANTUM: &str = "480/48000";

/// One PipeWire capture stream connected to `target` (the default source when `None`).
pub(super) struct Capture<'c> {
    _frames: StreamListener<SinkSlot>,
    _status: StreamListener<StatusReporter>,
    stream: StreamBox<'c>,
}

impl<'c> Capture<'c> {
    pub(super) fn connect(
        core: &'c Core,
        target: Option<&str>,
        format: &[u8],
        slot: SinkSlot,
        status: StatusReporter,
    ) -> Result<Self, pw::Error> {
        let mut properties = properties! {
            *pw::keys::MEDIA_TYPE => "Audio",
            *pw::keys::MEDIA_CATEGORY => "Capture",
            *pw::keys::MEDIA_ROLE => "Production",
            *pw::keys::NODE_NAME => STREAM_NAME,
            *pw::keys::NODE_LATENCY => TEN_MS_QUANTUM,
        };
        if let Some(target) = target {
            properties.insert(*pw::keys::TARGET_OBJECT, property_value(target).as_ref());
        }
        let stream = StreamBox::new(core, STREAM_NAME, properties)?;
        let frames = stream
            .add_local_listener_with_user_data(slot)
            .process(capture_buffer)
            .register()?;
        let status = stream
            .add_local_listener_with_user_data(status)
            .state_changed(|_, reporter, _, state| reporter.report(state))
            .register()?;
        let format = Pod::from_bytes(format).ok_or(pw::Error::CreationFailed)?;
        stream.connect(
            Direction::Input,
            None,
            StreamFlags::AUTOCONNECT | StreamFlags::MAP_BUFFERS | StreamFlags::RT_PROCESS,
            &mut [format],
        )?;
        Ok(Self {
            _frames: frames,
            _status: status,
            stream,
        })
    }
}

impl Drop for Capture<'_> {
    fn drop(&mut self) {
        let _ = self.stream.disconnect();
    }
}

/// Forwards a capture's state changes, tagged with the capture's generation so the session
/// can ignore a replaced capture's late reports.
pub(super) struct StatusReporter {
    pub(super) events: mpsc::Sender<Event>,
    pub(super) generation: u32,
}

impl StatusReporter {
    fn report(&self, state: pw::stream::StreamState) {
        if let Some(status) = CaptureStatus::from_pipewire(state) {
            let _ = self.events.send(Event::Capture {
                generation: self.generation,
                status,
            });
        }
    }
}

fn capture_buffer(stream: &Stream, slot: &mut SinkSlot) {
    let Some(sink) = slot.sink() else {
        return;
    };
    let start = stream.time().map_or(ClockNanos(0), |time| {
        ClockNanos(u64::try_from(time.now()).unwrap_or_default())
    });
    let Some(mut buffer) = stream.dequeue_buffer() else {
        return;
    };
    let Some(data) = buffer.datas_mut().first_mut() else {
        return;
    };
    let offset = data.chunk().offset() as usize;
    let size = data.chunk().size() as usize;
    if let Some(bytes) = data
        .data()
        .and_then(|bytes| bytes.get(offset..offset + size))
    {
        sink.capture(start, bytes);
    }
}
