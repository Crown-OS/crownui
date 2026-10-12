use geometry::Vector;
use input::{GesturePhase, TouchpadGesture};

use crate::WindowId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GestureKind {
    Swipe,
    Pinch,
    Hold,
}

/// One protocol event of a touchpad gesture, already decoded.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum GestureInput {
    Begin {
        window: WindowId,
        fingers: u32,
    },
    Update {
        delta: Vector,
        scale: f32,
        rotation: f32,
    },
    End {
        cancelled: bool,
    },
}

impl GestureInput {
    pub(crate) fn swipe_update(dx: f64, dy: f64) -> Self {
        Self::pinch_update(dx, dy, 1.0, 0.0)
    }

    pub(crate) fn pinch_update(dx: f64, dy: f64, scale: f64, rotation: f64) -> Self {
        Self::Update {
            delta: Vector::new(dx as f32, dy as f32),
            scale: scale as f32,
            rotation: rotation as f32,
        }
    }

    pub(crate) fn end(cancelled: i32) -> Self {
        Self::End {
            cancelled: cancelled != 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct GestureTarget {
    window: WindowId,
    fingers: u32,
    scale: f32,
}

/// Follows one gesture object from begin to end: updates and ends carry no surface, so the
/// window under the pointer at begin keeps receiving them, and a pinch's end repeats the
/// last scale.
#[derive(Debug)]
pub(crate) struct GestureTracker {
    kind: GestureKind,
    target: Option<GestureTarget>,
}

impl GestureTracker {
    pub(crate) fn new(kind: GestureKind) -> Self {
        Self { kind, target: None }
    }

    pub(crate) fn track(&mut self, input: GestureInput) -> Option<(WindowId, TouchpadGesture)> {
        let (phase, delta, rotation) = match input {
            GestureInput::Begin { window, fingers } => {
                self.target = Some(GestureTarget {
                    window,
                    fingers,
                    scale: 1.0,
                });
                (GesturePhase::Begin, Vector::ZERO, 0.0)
            }
            GestureInput::Update {
                delta,
                scale,
                rotation,
            } => {
                self.target.as_mut()?.scale = scale;
                (GesturePhase::Update, delta, rotation)
            }
            GestureInput::End { cancelled } => {
                let phase = if cancelled {
                    GesturePhase::Cancel
                } else {
                    GesturePhase::End
                };
                (phase, Vector::ZERO, 0.0)
            }
        };
        let target = match phase {
            GesturePhase::End | GesturePhase::Cancel => self.target.take()?,
            GesturePhase::Begin | GesturePhase::Update => self.target?,
        };
        let gesture = match self.kind {
            GestureKind::Swipe => TouchpadGesture::Swipe {
                phase,
                fingers: target.fingers,
                delta,
            },
            GestureKind::Pinch => TouchpadGesture::Pinch {
                phase,
                scale: target.scale,
                rotation,
                delta,
            },
            GestureKind::Hold => TouchpadGesture::Hold {
                phase,
                fingers: target.fingers,
            },
        };
        Some((target.window, gesture))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const WINDOW: WindowId = WindowId::from_raw(7);

    #[test]
    fn a_pinch_reports_scale_rotation_and_ends_with_the_last_scale() {
        let mut tracker = GestureTracker::new(GestureKind::Pinch);
        let begin = tracker.track(GestureInput::Begin {
            window: WINDOW,
            fingers: 2,
        });
        assert_eq!(
            begin,
            Some((
                WINDOW,
                TouchpadGesture::Pinch {
                    phase: GesturePhase::Begin,
                    scale: 1.0,
                    rotation: 0.0,
                    delta: Vector::ZERO
                }
            ))
        );
        let update = tracker.track(GestureInput::pinch_update(1.0, -2.0, 1.5, 3.0));
        assert_eq!(
            update,
            Some((
                WINDOW,
                TouchpadGesture::Pinch {
                    phase: GesturePhase::Update,
                    scale: 1.5,
                    rotation: 3.0,
                    delta: Vector::new(1.0, -2.0)
                }
            ))
        );
        let end = tracker.track(GestureInput::end(0));
        assert_eq!(
            end,
            Some((
                WINDOW,
                TouchpadGesture::Pinch {
                    phase: GesturePhase::End,
                    scale: 1.5,
                    rotation: 0.0,
                    delta: Vector::ZERO
                }
            ))
        );
        assert_eq!(
            tracker.track(GestureInput::pinch_update(0.0, 0.0, 2.0, 0.0)),
            None
        );
    }

    #[test]
    fn a_cancelled_swipe_keeps_its_finger_count() {
        let mut tracker = GestureTracker::new(GestureKind::Swipe);
        tracker.track(GestureInput::Begin {
            window: WINDOW,
            fingers: 3,
        });
        assert_eq!(
            tracker.track(GestureInput::end(1)),
            Some((
                WINDOW,
                TouchpadGesture::Swipe {
                    phase: GesturePhase::Cancel,
                    fingers: 3,
                    delta: Vector::ZERO
                }
            ))
        );
    }

    #[test]
    fn events_without_a_begin_are_dropped() {
        let mut tracker = GestureTracker::new(GestureKind::Hold);
        assert_eq!(tracker.track(GestureInput::end(0)), None);
    }
}
