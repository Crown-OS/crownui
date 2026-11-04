use crate::ClockNanos;

/// Samples in one [`AudioFrame`]: 10 ms of mono audio at [`SAMPLE_RATE`](crate::SAMPLE_RATE).
pub const FRAME_SAMPLES: usize = 480;

/// 10 ms of mono 48 kHz audio, stamped with the graph time of its first sample.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AudioFrame {
    /// When the first sample was captured.
    pub pts: ClockNanos,
    /// Mono samples in `-1.0..=1.0`.
    pub samples: [f32; FRAME_SAMPLES],
}

impl AudioFrame {
    pub(crate) const SILENT: Self = Self {
        pts: ClockNanos(0),
        samples: [0.0; FRAME_SAMPLES],
    };
}

/// The lock-free ring a [`MicrophoneStream`](crate::MicrophoneStream) fills with frames; move it
/// to the thread that encodes or analyses them and `pop` from there.
pub type FrameConsumer = rtrb::Consumer<AudioFrame>;
