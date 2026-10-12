#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

mod arena;
mod defaults;
mod focus;
mod output;
mod recognizer;
#[cfg(test)]
mod test_support;
mod util;
mod velocity_tracker;
mod vote;

pub use arena::{ArenaDecision, ArenaDecisions, GestureArena};
pub use defaults::{
    CROSS_AXIS_WEIGHT, LONG_PRESS_DURATION, MAX_CLICK_COUNT, MAX_CLICK_DURATION,
    MULTI_CLICK_DISTANCE, MULTI_CLICK_INTERVAL, TOUCH_SLOP, VELOCITY_SAMPLE_CAPACITY,
    VELOCITY_WINDOW,
};
pub use focus::{Direction, FocusChange, FocusManager};
pub use output::GestureOutput;
pub use recognizer::{
    ClickConfig, ClickRecognizer, DragAxis, DragConfig, DragRecognizer, LongPressConfig,
    LongPressRecognizer, PinchRecognizer, Recognizer,
};
pub use velocity_tracker::VelocityTracker;
pub use vote::Vote;
