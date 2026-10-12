mod click;
mod drag;
mod drag_axis;
mod long_press;
mod pinch;

use std::time::Duration;

use input::{PointerEvent, TouchpadGesture};

use crate::{GestureOutput, Vote};

pub use click::{ClickConfig, ClickRecognizer};
pub use drag::{DragConfig, DragRecognizer};
pub use drag_axis::DragAxis;
pub use long_press::{LongPressConfig, LongPressRecognizer};
pub use pinch::PinchRecognizer;

/// A state machine that turns input into [`GestureOutput`]s while competing for its pointer in a
/// [`GestureArena`](crate::GestureArena).
///
/// A recognizer follows one pointer at a time. The caller adds it to that pointer's arena only when
/// [`allows_pointer`](Self::allows_pointer) is true, feeds it the pointer's events, forwards every
/// [`Vote`] to the arena and reports the arena's decision through [`accepted`](Self::accepted) or
/// [`rejected`](Self::rejected). Outputs produced before winning are held, with consecutive updates
/// coalesced, released by `accepted` and discarded by `rejected`.
pub trait Recognizer {
    /// Whether the recognizer wants to join the arena opened by the pointer down `event`.
    fn allows_pointer(&self, event: &PointerEvent) -> bool {
        let _ = event;
        true
    }

    /// A pointer went down; starts following it.
    fn pointer_down(&mut self, event: &PointerEvent) -> Vote;

    /// The pointer moved.
    fn pointer_move(&mut self, event: &PointerEvent) -> Vote;

    /// The pointer went up.
    fn pointer_up(&mut self, event: &PointerEvent) -> Vote;

    /// The platform cancelled the pointer; a started gesture emits [`GestureOutput::Cancel`].
    fn pointer_cancel(&mut self);

    /// A touchpad gesture arrived; recognizers that ignore touchpads reject it.
    fn touchpad(&mut self, gesture: &TouchpadGesture) -> Vote {
        let _ = gesture;
        Vote::Reject
    }

    /// Lets time-based recognizers react to the monotonic clock reaching `now`.
    fn poll(&mut self, now: Duration) -> Vote {
        let _ = now;
        Vote::Pending
    }

    /// The arena picked this recognizer; held outputs are released.
    fn accepted(&mut self);

    /// The arena picked another recognizer, or this one rejected; held outputs are discarded.
    fn rejected(&mut self);

    /// Moves released outputs into `out`.
    fn drain_outputs(&mut self, out: &mut Vec<GestureOutput>);
}
