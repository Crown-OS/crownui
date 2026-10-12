use std::time::Duration;

use geometry::Point;
use input::{PointerButton, PointerEvent};

use crate::util::{OutputQueue, Press};
use crate::{GestureOutput, LONG_PRESS_DURATION, Recognizer, TOUCH_SLOP, Vote};

/// Tunables for [`LongPressRecognizer`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LongPressConfig {
    /// The button that must be held.
    pub button: PointerButton,
    /// Logical pixels the press may move before it is no longer a long press.
    pub slop: f32,
    /// How long the press must be held.
    pub duration: Duration,
}

impl Default for LongPressConfig {
    fn default() -> Self {
        Self {
            button: PointerButton::Primary,
            slop: TOUCH_SLOP,
            duration: LONG_PRESS_DURATION,
        }
    }
}

/// Recognizes a press held within the slop for [`LongPressConfig::duration`]. Accepts from
/// [`poll`](Recognizer::poll) (or a later pointer event) once the time is up; releasing earlier
/// rejects.
#[derive(Debug, Default)]
pub struct LongPressRecognizer {
    config: LongPressConfig,
    press: Option<Press>,
    position: Point,
    fired: bool,
    queue: OutputQueue,
}

impl LongPressRecognizer {
    /// A recognizer with `config`.
    pub fn new(config: LongPressConfig) -> Self {
        Self {
            config,
            ..Self::default()
        }
    }

    fn fire_if_due(&mut self, now: Duration) -> Vote {
        match self.press {
            Some(press) if !self.fired && press.held_for(now) >= self.config.duration => {
                self.fired = true;
                self.queue.push(GestureOutput::LongPress {
                    position: self.position,
                });
                Vote::Accept
            }
            _ => Vote::Pending,
        }
    }
}

impl Recognizer for LongPressRecognizer {
    fn allows_pointer(&self, event: &PointerEvent) -> bool {
        self.press.is_none() && event.button == Some(self.config.button)
    }

    fn pointer_down(&mut self, event: &PointerEvent) -> Vote {
        if !self.allows_pointer(event) {
            return Vote::Reject;
        }
        self.queue.begin();
        self.press = Some(Press::new(event, self.config.button));
        self.position = event.position;
        self.fired = false;
        Vote::Pending
    }

    fn pointer_move(&mut self, event: &PointerEvent) -> Vote {
        let Some(press) = self.press.filter(|press| press.follows(event)) else {
            return Vote::Pending;
        };
        if self.fire_if_due(event.timestamp) == Vote::Accept || self.fired {
            return Vote::Accept;
        }
        if press.exceeds_slop(event.position, self.config.slop) {
            self.press = None;
            return Vote::Reject;
        }
        self.position = event.position;
        Vote::Pending
    }

    fn pointer_up(&mut self, event: &PointerEvent) -> Vote {
        if !self.press.is_some_and(|press| press.is_released_by(event)) {
            return Vote::Pending;
        }
        self.fire_if_due(event.timestamp);
        self.press = None;
        if self.fired {
            Vote::Accept
        } else {
            Vote::Reject
        }
    }

    fn pointer_cancel(&mut self) {
        self.rejected();
    }

    fn poll(&mut self, now: Duration) -> Vote {
        self.fire_if_due(now)
    }

    fn accepted(&mut self) {
        self.queue.accept();
    }

    fn rejected(&mut self) {
        self.press = None;
        self.queue.reject();
    }

    fn drain_outputs(&mut self, out: &mut Vec<GestureOutput>) {
        self.queue.drain_into(out);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{at, down, drain};

    #[test]
    fn holding_still_accepts_via_poll() {
        let mut recognizer = LongPressRecognizer::default();
        recognizer.pointer_down(&down(5.0, 5.0, 100));
        assert_eq!(recognizer.pointer_move(&at(8.0, 5.0, 200)), Vote::Pending);
        assert_eq!(recognizer.poll(Duration::from_millis(599)), Vote::Pending);
        assert_eq!(recognizer.poll(Duration::from_millis(600)), Vote::Accept);
        recognizer.accepted();
        assert_eq!(
            drain(&mut recognizer),
            [GestureOutput::LongPress {
                position: Point::new(8.0, 5.0)
            }]
        );
        assert_eq!(recognizer.poll(Duration::from_millis(700)), Vote::Pending);
        assert_eq!(recognizer.pointer_up(&down(8.0, 5.0, 800)), Vote::Accept);
        assert!(drain(&mut recognizer).is_empty());
    }

    #[test]
    fn early_release_rejects() {
        let mut recognizer = LongPressRecognizer::default();
        recognizer.pointer_down(&down(0.0, 0.0, 0));
        assert_eq!(recognizer.pointer_up(&down(0.0, 0.0, 300)), Vote::Reject);
        assert_eq!(recognizer.poll(Duration::from_millis(600)), Vote::Pending);
    }

    #[test]
    fn moving_beyond_slop_rejects() {
        let mut recognizer = LongPressRecognizer::default();
        recognizer.pointer_down(&down(0.0, 0.0, 0));
        assert_eq!(recognizer.pointer_move(&at(0.0, 9.0, 100)), Vote::Reject);
        assert_eq!(recognizer.poll(Duration::from_millis(600)), Vote::Pending);
    }

    #[test]
    fn late_event_without_poll_still_fires() {
        let mut recognizer = LongPressRecognizer::default();
        recognizer.pointer_down(&down(0.0, 0.0, 0));
        assert_eq!(recognizer.pointer_up(&down(0.0, 0.0, 700)), Vote::Accept);
        recognizer.accepted();
        assert_eq!(drain(&mut recognizer).len(), 1);
    }

    #[test]
    fn losing_discards_the_long_press() {
        let mut recognizer = LongPressRecognizer::default();
        recognizer.pointer_down(&down(0.0, 0.0, 0));
        recognizer.poll(Duration::from_millis(500));
        recognizer.rejected();
        assert!(drain(&mut recognizer).is_empty());
    }
}
