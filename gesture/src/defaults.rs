use std::time::Duration;

/// Logical pixels a press may wander before it stops being a click or long press and may become
/// a drag.
pub const TOUCH_SLOP: f32 = 6.0;

/// A press held this long or longer is not a click.
pub const MAX_CLICK_DURATION: Duration = Duration::from_millis(500);

/// Longest gap from a click's release to the next press for the click count to grow.
pub const MULTI_CLICK_INTERVAL: Duration = Duration::from_millis(400);

/// Farthest, in logical pixels, a press may land from the previous click for the count to grow.
pub const MULTI_CLICK_DISTANCE: f32 = 10.0;

/// Highest click count (triple click); the next click starts again at one.
pub const MAX_CLICK_COUNT: u8 = 3;

/// How long a press must be held within the slop to become a long press.
pub const LONG_PRESS_DURATION: Duration = Duration::from_millis(500);

/// Samples older than this, relative to the newest one, do not affect velocity.
pub const VELOCITY_WINDOW: Duration = Duration::from_millis(100);

/// Samples a [`VelocityTracker`](crate::VelocityTracker) keeps.
pub const VELOCITY_SAMPLE_CAPACITY: usize = 20;

/// How much cross-axis offset costs relative to distance along the arrow's axis when
/// [`FocusManager::focus_direction`](crate::FocusManager::focus_direction) picks a target.
pub const CROSS_AXIS_WEIGHT: f32 = 2.0;
