use std::sync::mpsc;

use rtrb::Producer;
use tokio::sync::watch;

use crate::{AudioFrame, ClockNanos, Level};

use super::frame_packer::FramePacker;

/// The real-time end of a stream: packs buffers into frames, pushes them into the ring and
/// publishes their level. Lives in whichever PipeWire capture currently feeds the stream.
pub(crate) struct FrameSink {
    packer: FramePacker,
    frames: Producer<AudioFrame>,
    level: watch::Sender<Level>,
}

impl FrameSink {
    pub(crate) const fn new(frames: Producer<AudioFrame>, level: watch::Sender<Level>) -> Self {
        Self {
            packer: FramePacker::new(),
            frames,
            level,
        }
    }

    pub(super) fn capture(&mut self, start: ClockNanos, bytes: &[u8]) {
        let Self {
            packer,
            frames,
            level,
        } = self;
        packer.pack(start, bytes, |frame| {
            level.send_replace(Level::measure(&frame.samples));
            let _ = frames.push(*frame);
        });
    }
}

/// Holds the [`FrameSink`] inside a PipeWire listener and hands it back through `home` when
/// the listener is dropped, so the next capture (after a retarget) feeds the same ring.
pub(super) struct SinkSlot {
    sink: Option<FrameSink>,
    home: mpsc::Sender<FrameSink>,
}

impl SinkSlot {
    pub(super) fn new(mut sink: FrameSink, home: mpsc::Sender<FrameSink>) -> Self {
        sink.packer.discard_partial();
        Self {
            sink: Some(sink),
            home,
        }
    }

    pub(super) fn sink(&mut self) -> Option<&mut FrameSink> {
        self.sink.as_mut()
    }
}

impl Drop for SinkSlot {
    fn drop(&mut self) {
        if let Some(sink) = self.sink.take() {
            let _ = self.home.send(sink);
        }
    }
}

#[cfg(test)]
mod tests {
    use rtrb::RingBuffer;

    use super::*;

    #[test]
    fn a_dropped_slot_returns_its_sink() {
        let (producer, _consumer) = RingBuffer::new(1);
        let (level, _) = watch::channel(Level::default());
        let (home, returned) = mpsc::channel();
        drop(SinkSlot::new(FrameSink::new(producer, level), home));
        assert!(returned.try_recv().is_ok());
    }

    #[test]
    fn a_full_ring_drops_frames_but_still_reports_the_level() {
        let (producer, mut consumer) = RingBuffer::new(1);
        let (level, levels) = watch::channel(Level::default());
        let mut sink = FrameSink::new(producer, level);
        let loud: Vec<u8> = [0.5_f32; 960]
            .into_iter()
            .flat_map(f32::to_le_bytes)
            .collect();
        sink.capture(ClockNanos(0), &loud);
        assert_eq!(levels.borrow().peak, 0.5);
        assert_eq!(consumer.pop().map(|frame| frame.pts), Ok(ClockNanos(0)));
        assert!(consumer.pop().is_err());
    }
}
