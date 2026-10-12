use geometry::Point;
use input::{PointerButton, PointerEvent};

use crate::util::{OutputQueue, Press};
use crate::{DragAxis, GestureOutput, Recognizer, TOUCH_SLOP, VelocityTracker, Vote};

/// Tunables for [`DragRecognizer`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DragConfig {
    /// The button that drags.
    pub button: PointerButton,
    /// Logical pixels the pointer must move before the drag starts.
    pub slop: f32,
    /// The direction the drag is locked to.
    pub axis: DragAxis,
}

impl Default for DragConfig {
    fn default() -> Self {
        Self {
            button: PointerButton::Primary,
            slop: TOUCH_SLOP,
            axis: DragAxis::Free,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
enum DragState {
    #[default]
    Idle,
    Pressed(Press),
    Dragging {
        press: Press,
        last_position: Point,
    },
}

/// Recognizes a drag: accepts once the pointer leaves the slop along its [`DragAxis`] (rejecting
/// when the movement runs across the axis), then emits `DragStart`, `Drag` and `DragEnd` with the
/// release velocity.
#[derive(Debug, Default)]
pub struct DragRecognizer {
    config: DragConfig,
    state: DragState,
    velocity: VelocityTracker,
    queue: OutputQueue,
}

impl DragRecognizer {
    /// A recognizer with `config`.
    pub fn new(config: DragConfig) -> Self {
        Self {
            config,
            ..Self::default()
        }
    }

    fn start_if_beyond_slop(&mut self, press: Press, position: Point) -> Vote {
        if !press.exceeds_slop(position, self.config.slop) {
            return Vote::Pending;
        }
        let offset = position - press.origin;
        if !self.config.axis.is_along(offset) {
            self.state = DragState::Idle;
            return Vote::Reject;
        }
        let total = self.config.axis.project(offset);
        self.queue.push(GestureOutput::DragStart {
            position: press.origin,
        });
        self.queue.push(GestureOutput::Drag {
            position,
            delta: total,
            total,
        });
        self.state = DragState::Dragging {
            press,
            last_position: position,
        };
        Vote::Accept
    }
}

impl Recognizer for DragRecognizer {
    fn allows_pointer(&self, event: &PointerEvent) -> bool {
        matches!(self.state, DragState::Idle) && event.button == Some(self.config.button)
    }

    fn pointer_down(&mut self, event: &PointerEvent) -> Vote {
        if !self.allows_pointer(event) {
            return Vote::Reject;
        }
        self.queue.begin();
        self.velocity.reset();
        self.velocity.add_sample(event.timestamp, event.position);
        self.state = DragState::Pressed(Press::new(event, self.config.button));
        Vote::Pending
    }

    fn pointer_move(&mut self, event: &PointerEvent) -> Vote {
        match self.state {
            DragState::Pressed(press) if press.follows(event) => {
                self.velocity.add_sample(event.timestamp, event.position);
                self.start_if_beyond_slop(press, event.position)
            }
            DragState::Dragging {
                press,
                last_position,
            } if press.follows(event) => {
                self.velocity.add_sample(event.timestamp, event.position);
                let axis = self.config.axis;
                self.queue.push(GestureOutput::Drag {
                    position: event.position,
                    delta: axis.project(event.position - last_position),
                    total: axis.project(event.position - press.origin),
                });
                self.state = DragState::Dragging {
                    press,
                    last_position: event.position,
                };
                Vote::Accept
            }
            _ => Vote::Pending,
        }
    }

    fn pointer_up(&mut self, event: &PointerEvent) -> Vote {
        match self.state {
            DragState::Pressed(press) if press.is_released_by(event) => {
                self.state = DragState::Idle;
                Vote::Reject
            }
            DragState::Dragging { press, .. } if press.is_released_by(event) => {
                self.velocity.add_sample(event.timestamp, event.position);
                self.queue.push(GestureOutput::DragEnd {
                    position: event.position,
                    velocity: self.config.axis.project(self.velocity.velocity()),
                });
                self.state = DragState::Idle;
                Vote::Accept
            }
            _ => Vote::Pending,
        }
    }

    fn pointer_cancel(&mut self) {
        if matches!(self.state, DragState::Dragging { .. }) && self.queue.is_won() {
            self.queue.push(GestureOutput::Cancel);
        }
        self.rejected();
    }

    fn accepted(&mut self) {
        self.queue.accept();
    }

    fn rejected(&mut self) {
        self.state = DragState::Idle;
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
    use crate::test_support::{at, down, drain};

    fn locked(axis: DragAxis) -> DragRecognizer {
        DragRecognizer::new(DragConfig {
            axis,
            ..DragConfig::default()
        })
    }

    #[test]
    fn movement_within_slop_stays_pending_and_release_rejects() {
        let mut recognizer = DragRecognizer::default();
        recognizer.pointer_down(&down(0.0, 0.0, 0));
        assert_eq!(recognizer.pointer_move(&at(4.0, 4.0, 10)), Vote::Pending);
        assert_eq!(recognizer.pointer_up(&down(4.0, 4.0, 20)), Vote::Reject);
    }

    #[test]
    fn leaving_slop_accepts_and_flushes_start_plus_one_drag() {
        let mut recognizer = DragRecognizer::default();
        recognizer.pointer_down(&down(0.0, 0.0, 0));
        assert_eq!(recognizer.pointer_move(&at(7.0, 0.0, 10)), Vote::Accept);
        recognizer.pointer_move(&at(10.0, 2.0, 20));
        recognizer.accepted();
        recognizer.pointer_move(&at(12.0, 2.0, 30));
        assert_eq!(recognizer.pointer_up(&down(12.0, 2.0, 40)), Vote::Accept);
        let outputs = drain(&mut recognizer);
        assert_eq!(
            outputs[..3],
            [
                GestureOutput::DragStart {
                    position: Point::ZERO
                },
                GestureOutput::Drag {
                    position: Point::new(10.0, 2.0),
                    delta: Vector::new(10.0, 2.0),
                    total: Vector::new(10.0, 2.0),
                },
                GestureOutput::Drag {
                    position: Point::new(12.0, 2.0),
                    delta: Vector::new(2.0, 0.0),
                    total: Vector::new(12.0, 2.0),
                },
            ]
        );
        assert!(matches!(outputs[3], GestureOutput::DragEnd { velocity, .. } if velocity.x > 0.0));
    }

    #[test]
    fn vertical_drag_rejects_horizontal_movement() {
        let mut recognizer = locked(DragAxis::Vertical);
        recognizer.pointer_down(&down(0.0, 0.0, 0));
        assert_eq!(recognizer.pointer_move(&at(8.0, 3.0, 10)), Vote::Reject);
    }

    #[test]
    fn horizontal_drag_projects_onto_its_axis() {
        let mut recognizer = locked(DragAxis::Horizontal);
        recognizer.pointer_down(&down(0.0, 0.0, 0));
        assert_eq!(recognizer.pointer_move(&at(8.0, 3.0, 10)), Vote::Accept);
        recognizer.accepted();
        assert_eq!(
            drain(&mut recognizer)[1],
            GestureOutput::Drag {
                position: Point::new(8.0, 3.0),
                delta: Vector::new(8.0, 0.0),
                total: Vector::new(8.0, 0.0),
            }
        );
    }

    #[test]
    fn release_velocity_matches_constant_motion() {
        let mut recognizer = DragRecognizer::default();
        recognizer.pointer_down(&down(0.0, 0.0, 0));
        recognizer.accepted();
        for step in 1..=10u16 {
            let x = f32::from(step) * 4.0;
            recognizer.pointer_move(&at(x, 0.0, u64::from(step) * 8));
        }
        recognizer.pointer_up(&down(44.0, 0.0, 88));
        let Some(GestureOutput::DragEnd { velocity, .. }) = drain(&mut recognizer).pop() else {
            panic!("drag did not end");
        };
        assert!((velocity.x - 500.0).abs() < 10.0, "{velocity:?}");
    }

    #[test]
    fn cancelling_a_won_drag_emits_cancel() {
        let mut recognizer = DragRecognizer::default();
        recognizer.pointer_down(&down(0.0, 0.0, 0));
        recognizer.pointer_move(&at(10.0, 0.0, 10));
        recognizer.accepted();
        recognizer.pointer_cancel();
        assert_eq!(drain(&mut recognizer).last(), Some(&GestureOutput::Cancel));
    }
}
