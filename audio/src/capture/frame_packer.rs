use crate::{AudioFrame, ClockNanos, FRAME_SAMPLES};

const SAMPLE_BYTES: usize = size_of::<f32>();

/// Cuts PipeWire buffers of any length into [`AudioFrame`]s, stamping each with the graph time
/// of its first sample. Holds one frame; never allocates.
pub(super) struct FramePacker {
    frame: AudioFrame,
    filled: usize,
}

impl FramePacker {
    pub(super) const fn new() -> Self {
        Self {
            frame: AudioFrame::SILENT,
            filled: 0,
        }
    }

    pub(super) fn discard_partial(&mut self) {
        self.filled = 0;
    }

    /// Appends little-endian `f32` samples captured from `start` on, calling `on_frame` for
    /// every frame they complete.
    pub(super) fn pack(
        &mut self,
        start: ClockNanos,
        bytes: &[u8],
        mut on_frame: impl FnMut(&AudioFrame),
    ) {
        let (mut samples, _) = bytes.as_chunks::<SAMPLE_BYTES>();
        let mut consumed = 0;
        while !samples.is_empty() {
            if self.filled == 0 {
                self.frame.pts = start.after_samples(consumed);
            }
            let open = &mut self.frame.samples[self.filled..];
            let (taken, rest) = samples.split_at(open.len().min(samples.len()));
            open.iter_mut()
                .zip(taken)
                .for_each(|(sample, raw)| *sample = f32::from_le_bytes(*raw));
            self.filled += taken.len();
            consumed += taken.len();
            samples = rest;
            if self.filled == FRAME_SAMPLES {
                on_frame(&self.frame);
                self.filled = 0;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bytes_of(samples: impl IntoIterator<Item = f32>) -> Vec<u8> {
        samples.into_iter().flat_map(f32::to_le_bytes).collect()
    }

    fn packed(packer: &mut FramePacker, start: u64, samples: &[f32]) -> Vec<AudioFrame> {
        let mut frames = Vec::new();
        packer.pack(
            ClockNanos(start),
            &bytes_of(samples.iter().copied()),
            |frame| {
                frames.push(*frame);
            },
        );
        frames
    }

    #[test]
    fn an_exact_quantum_makes_one_frame_at_the_buffer_time() {
        let samples: Vec<f32> = (0..480).map(|index| index as f32).collect();
        let frames = packed(&mut FramePacker::new(), 1_000, &samples);
        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0].pts, ClockNanos(1_000));
        assert_eq!(frames[0].samples[479], 479.0);
    }

    #[test]
    fn frames_span_buffers_and_keep_the_time_of_their_first_sample() {
        let mut packer = FramePacker::new();
        assert!(packed(&mut packer, 0, &[0.5; 300]).is_empty());
        let frames = packed(&mut packer, 6_250_000, &[0.25; 1024]);
        assert_eq!(frames.len(), 2);
        assert_eq!(frames[0].pts, ClockNanos(0));
        assert_eq!(frames[0].samples[299], 0.5);
        assert_eq!(frames[0].samples[300], 0.25);
        assert_eq!(frames[1].pts, ClockNanos(6_250_000).after_samples(180));
    }

    #[test]
    fn a_discarded_partial_frame_restarts_at_the_next_buffer() {
        let mut packer = FramePacker::new();
        assert!(packed(&mut packer, 0, &[1.0; 100]).is_empty());
        packer.discard_partial();
        let frames = packed(&mut packer, 42, &[0.0; 480]);
        assert_eq!(frames[0].pts, ClockNanos(42));
        assert_eq!(frames[0].samples, [0.0; FRAME_SAMPLES]);
    }

    #[test]
    fn a_torn_trailing_sample_is_ignored() {
        let mut bytes = bytes_of([0.5; 480]);
        bytes.push(0xff);
        let mut count = 0;
        FramePacker::new().pack(ClockNanos(0), &bytes, |_| count += 1);
        assert_eq!(count, 1);
    }
}
