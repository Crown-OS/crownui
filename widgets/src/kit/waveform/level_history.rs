use super::metrics::{ATTACK_PER_UPDATE, HISTORY_CAPACITY, RELEASE_PER_UPDATE};
use crate::util::unit_interval::{byte_to_unit, clamp_unit, unit_to_byte};

/// The recent input levels a [`live_waveform`](super::live_waveform) scrolls through: a fixed ring
/// of [`CAPACITY`](Self::CAPACITY) bytes that never reallocates. Each pushed level is smoothed with
/// a fast attack and a slow release, so peaks pop without flicker.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LevelHistory {
    levels: [u8; HISTORY_CAPACITY],
    next: usize,
    len: usize,
    envelope: f32,
}

impl LevelHistory {
    /// How many levels are kept; older ones fall off the left edge.
    pub const CAPACITY: usize = HISTORY_CAPACITY;

    /// An empty history.
    pub const fn new() -> Self {
        Self {
            levels: [0; HISTORY_CAPACITY],
            next: 0,
            len: 0,
            envelope: 0.0,
        }
    }

    /// Appends a `0..=1` level, typically once per audio update (about 60 Hz).
    pub fn push(&mut self, level: f32) {
        let level = clamp_unit(level);
        let rate = if level > self.envelope {
            ATTACK_PER_UPDATE
        } else {
            RELEASE_PER_UPDATE
        };
        self.envelope += (level - self.envelope) * rate;
        self.levels[self.next] = unit_to_byte(self.envelope);
        self.next = (self.next + 1) % HISTORY_CAPACITY;
        self.len = (self.len + 1).min(HISTORY_CAPACITY);
    }

    /// Forgets every level, e.g. when a new take starts.
    pub fn clear(&mut self) {
        *self = Self::new();
    }

    /// How many levels are held.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Whether nothing was pushed since the last clear.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// The smoothed levels, newest first.
    pub fn newest_first(&self) -> impl Iterator<Item = f32> + '_ {
        (1..=self.len).map(move |age| {
            byte_to_unit(self.levels[(self.next + HISTORY_CAPACITY - age) % HISTORY_CAPACITY])
        })
    }
}

impl Default for LevelHistory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_sudden_level_is_reached_in_a_few_updates_and_left_slowly() {
        let mut history = LevelHistory::new();
        history.push(1.0);
        let first = history.newest_first().next().unwrap_or_default();
        assert!((first - 0.4).abs() < 0.01, "{first}");
        for _ in 0..5 {
            history.push(1.0);
        }
        assert!(history.newest_first().next().unwrap_or_default() > 0.9);
        history.push(0.0);
        assert!(history.newest_first().next().unwrap_or_default() > 0.75);
    }

    #[test]
    fn the_ring_keeps_the_newest_capacity_levels_without_growing() {
        let mut history = LevelHistory::new();
        for _ in 0..LevelHistory::CAPACITY + 10 {
            history.push(0.5);
        }
        assert_eq!(history.len(), LevelHistory::CAPACITY);
        assert_eq!(history.newest_first().count(), LevelHistory::CAPACITY);
        history.clear();
        assert!(history.is_empty());
    }
}
