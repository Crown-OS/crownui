use smallvec::SmallVec;

use crate::GestureOutput;

type Outputs = SmallVec<[GestureOutput; 4]>;

/// Holds a recognizer's outputs until it wins its arena, coalescing held updates so a long wait
/// never grows the queue.
#[derive(Debug, Default)]
pub(crate) struct OutputQueue {
    held: Outputs,
    released: Outputs,
    won: bool,
}

impl OutputQueue {
    pub fn begin(&mut self) {
        self.held.clear();
        self.won = false;
    }

    pub fn push(&mut self, output: GestureOutput) {
        if self.won {
            self.released.push(output);
        } else if !self
            .held
            .last_mut()
            .is_some_and(|last| coalesce(last, output))
        {
            self.held.push(output);
        }
    }

    pub fn accept(&mut self) {
        self.won = true;
        self.released.extend(self.held.drain(..));
    }

    pub fn reject(&mut self) {
        self.begin();
    }

    pub fn is_won(&self) -> bool {
        self.won
    }

    pub fn drain_into(&mut self, out: &mut Vec<GestureOutput>) {
        out.extend(self.released.drain(..));
    }
}

fn coalesce(held: &mut GestureOutput, next: GestureOutput) -> bool {
    match (held, next) {
        (
            GestureOutput::Drag {
                position,
                delta,
                total,
            },
            GestureOutput::Drag {
                position: next_position,
                delta: next_delta,
                total: next_total,
            },
        ) => {
            *position = next_position;
            *delta = *delta + next_delta;
            *total = next_total;
            true
        }
        (
            GestureOutput::Pinch {
                scale,
                rotation,
                center_delta,
            },
            GestureOutput::Pinch {
                scale: next_scale,
                rotation: next_rotation,
                center_delta: next_center_delta,
            },
        ) => {
            *scale = next_scale;
            *rotation += next_rotation;
            *center_delta = *center_delta + next_center_delta;
            true
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use geometry::{Point, Vector};

    use super::*;

    fn drag(x: f32, delta: f32) -> GestureOutput {
        GestureOutput::Drag {
            position: Point::new(x, 0.0),
            delta: Vector::new(delta, 0.0),
            total: Vector::new(x, 0.0),
        }
    }

    #[test]
    fn held_drags_coalesce_and_flush_on_accept() {
        let mut queue = OutputQueue::default();
        queue.push(GestureOutput::DragStart {
            position: Point::ZERO,
        });
        queue.push(drag(7.0, 7.0));
        queue.push(drag(10.0, 3.0));
        let mut out = Vec::new();
        queue.drain_into(&mut out);
        assert!(out.is_empty());

        queue.accept();
        queue.push(drag(12.0, 2.0));
        queue.drain_into(&mut out);
        assert_eq!(
            out,
            [
                GestureOutput::DragStart {
                    position: Point::ZERO
                },
                drag(10.0, 10.0),
                drag(12.0, 2.0),
            ]
        );
    }

    #[test]
    fn rejecting_discards_held_outputs() {
        let mut queue = OutputQueue::default();
        queue.push(GestureOutput::PinchStart);
        queue.reject();
        queue.accept();
        let mut out = Vec::new();
        queue.drain_into(&mut out);
        assert!(out.is_empty());
    }
}
