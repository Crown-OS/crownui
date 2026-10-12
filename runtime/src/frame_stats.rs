use std::collections::VecDeque;

use inspect::FrameStats;

const FRAME_HISTORY: usize = 120;

/// The most recent frames' timings, oldest first.
#[derive(Debug, Default)]
pub(crate) struct FrameHistory {
    frames: VecDeque<FrameStats>,
}

impl FrameHistory {
    pub(crate) fn push(&mut self, stats: FrameStats) {
        if self.frames.len() == FRAME_HISTORY {
            self.frames.pop_front();
        }
        self.frames.push_back(stats);
    }

    pub(crate) fn snapshot(&self) -> Vec<FrameStats> {
        self.frames.iter().copied().collect()
    }
}

pub(crate) fn micros(duration: std::time::Duration) -> u32 {
    u32::try_from(duration.as_micros()).unwrap_or(u32::MAX)
}
