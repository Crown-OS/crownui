use std::time::Duration;

use geometry::Point;
use input::{GesturePhase, PointerEvent, TouchpadGesture};

use crate::util::OutputQueue;
use crate::{GestureOutput, Recognizer, VelocityTracker, Vote};

/// Recognizes touchpad pinches: `Begin` accepts at once, `Update` emits `Pinch`, `End` emits
/// `PinchEnd` and `Cancel` emits `Cancel`.
///
/// Touchpad gestures carry no timestamp, so the scale velocity is measured against the clock last
/// passed to [`poll`](Recognizer::poll); poll every frame for a meaningful `velocity_scale`.
#[derive(Debug, Default)]
pub struct PinchRecognizer {
    active: bool,
    clock: Duration,
    scale_history: VelocityTracker,
    queue: OutputQueue,
}

impl PinchRecognizer {
    /// An idle recognizer.
    pub fn new() -> Self {
        Self::default()
    }

    fn record_scale(&mut self, scale: f32) {
        self.scale_history
            .add_sample(self.clock, Point::new(scale, 0.0));
    }
}

impl Recognizer for PinchRecognizer {
    fn allows_pointer(&self, _event: &PointerEvent) -> bool {
        false
    }

    fn pointer_down(&mut self, _event: &PointerEvent) -> Vote {
        Vote::Reject
    }

    fn pointer_move(&mut self, _event: &PointerEvent) -> Vote {
        Vote::Pending
    }

    fn pointer_up(&mut self, _event: &PointerEvent) -> Vote {
        Vote::Pending
    }

    fn pointer_cancel(&mut self) {}

    fn touchpad(&mut self, gesture: &TouchpadGesture) -> Vote {
        let TouchpadGesture::Pinch {
            phase,
            scale,
            rotation,
            delta,
        } = *gesture
        else {
            return Vote::Reject;
        };
        match (phase, self.active) {
            (GesturePhase::Begin, _) => {
                self.queue.begin();
                self.active = true;
                self.scale_history.reset();
                self.record_scale(scale);
                self.queue.push(GestureOutput::PinchStart);
                Vote::Accept
            }
            (GesturePhase::Update, true) => {
                self.record_scale(scale);
                self.queue.push(GestureOutput::Pinch {
                    scale,
                    rotation,
                    center_delta: delta,
                });
                Vote::Accept
            }
            (GesturePhase::End, true) => {
                self.record_scale(scale);
                self.active = false;
                self.queue.push(GestureOutput::PinchEnd {
                    velocity_scale: self.scale_history.velocity().x,
                });
                Vote::Accept
            }
            (GesturePhase::Cancel, true) => {
                self.active = false;
                self.queue.push(GestureOutput::Cancel);
                Vote::Reject
            }
            _ => Vote::Reject,
        }
    }

    fn poll(&mut self, now: Duration) -> Vote {
        self.clock = now;
        Vote::Pending
    }

    fn accepted(&mut self) {
        self.queue.accept();
    }

    fn rejected(&mut self) {
        self.active = false;
        self.queue.reject();
    }

    fn drain_outputs(&mut self, out: &mut Vec<GestureOutput>) {
        self.queue.drain_into(out);
    }
}

#[cfg(test)]
mod tests {
    use geometry::Vector;

    use super::*;
    use crate::test_support::{down, drain};

    fn pinch(phase: GesturePhase, scale: f32) -> TouchpadGesture {
        TouchpadGesture::Pinch {
            phase,
            scale,
            rotation: 1.0,
            delta: Vector::new(1.0, 0.0),
        }
    }

    #[test]
    fn pinch_lifecycle_emits_start_updates_and_end_velocity() {
        let mut recognizer = PinchRecognizer::new();
        assert_eq!(
            recognizer.touchpad(&pinch(GesturePhase::Begin, 1.0)),
            Vote::Accept
        );
        recognizer.accepted();
        for frame in 1..=5u32 {
            recognizer.poll(Duration::from_millis(u64::from(frame) * 10));
            recognizer.touchpad(&pinch(GesturePhase::Update, 1.0 + frame as f32 * 0.1));
        }
        recognizer.poll(Duration::from_millis(60));
        assert_eq!(
            recognizer.touchpad(&pinch(GesturePhase::End, 1.6)),
            Vote::Accept
        );
        let outputs = drain(&mut recognizer);
        assert_eq!(outputs[0], GestureOutput::PinchStart);
        assert_eq!(outputs.len(), 7);
        let GestureOutput::PinchEnd { velocity_scale } = outputs[6] else {
            panic!("pinch did not end");
        };
        assert!((velocity_scale - 10.0).abs() < 0.2, "{velocity_scale}");
    }

    #[test]
    fn held_updates_coalesce_until_accepted() {
        let mut recognizer = PinchRecognizer::new();
        recognizer.touchpad(&pinch(GesturePhase::Begin, 1.0));
        recognizer.touchpad(&pinch(GesturePhase::Update, 1.2));
        recognizer.touchpad(&pinch(GesturePhase::Update, 1.5));
        recognizer.accepted();
        assert_eq!(
            drain(&mut recognizer),
            [
                GestureOutput::PinchStart,
                GestureOutput::Pinch {
                    scale: 1.5,
                    rotation: 2.0,
                    center_delta: Vector::new(2.0, 0.0),
                },
            ]
        );
    }

    #[test]
    fn cancel_emits_cancel_and_stray_updates_reject() {
        let mut recognizer = PinchRecognizer::new();
        assert_eq!(
            recognizer.touchpad(&pinch(GesturePhase::Update, 1.0)),
            Vote::Reject
        );
        recognizer.touchpad(&pinch(GesturePhase::Begin, 1.0));
        recognizer.accepted();
        assert_eq!(
            recognizer.touchpad(&pinch(GesturePhase::Cancel, 1.0)),
            Vote::Reject
        );
        assert_eq!(drain(&mut recognizer).last(), Some(&GestureOutput::Cancel));
        assert_eq!(
            recognizer.touchpad(&pinch(GesturePhase::End, 1.0)),
            Vote::Reject
        );
    }

    #[test]
    fn pointer_presses_and_other_touchpad_gestures_reject() {
        let mut recognizer = PinchRecognizer::new();
        assert_eq!(recognizer.pointer_down(&down(0.0, 0.0, 0)), Vote::Reject);
        let swipe = TouchpadGesture::Swipe {
            phase: GesturePhase::Begin,
            fingers: 3,
            delta: Vector::ZERO,
        };
        assert_eq!(recognizer.touchpad(&swipe), Vote::Reject);
    }
}
