use std::time::Duration;

use geometry::Point;
use input::{PointerButton, PointerEvent};

use crate::util::{OutputQueue, Press};
use crate::{
    GestureOutput, MAX_CLICK_COUNT, MAX_CLICK_DURATION, MULTI_CLICK_DISTANCE, MULTI_CLICK_INTERVAL,
    Recognizer, TOUCH_SLOP, Vote,
};

/// Tunables for [`ClickRecognizer`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClickConfig {
    /// The button that clicks; `Secondary` recognizes context-menu clicks.
    pub button: PointerButton,
    /// Logical pixels the press may move before it is no longer a click.
    pub slop: f32,
    /// A press held this long or longer is not a click.
    pub max_press_duration: Duration,
    /// Longest gap from a click's release to the next press for the click count to grow.
    pub multi_click_interval: Duration,
    /// Farthest a press may land from the previous click for the click count to grow.
    pub multi_click_distance: f32,
}

impl Default for ClickConfig {
    fn default() -> Self {
        Self {
            button: PointerButton::Primary,
            slop: TOUCH_SLOP,
            max_press_duration: MAX_CLICK_DURATION,
            multi_click_interval: MULTI_CLICK_INTERVAL,
            multi_click_distance: MULTI_CLICK_DISTANCE,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct ClickRecord {
    released: Duration,
    position: Point,
    count: u8,
}

/// Recognizes `click`, `dblclick` and triple clicks: a press and release within the slop and
/// shorter than [`ClickConfig::max_press_duration`]. Accepts on release.
#[derive(Debug, Default)]
pub struct ClickRecognizer {
    config: ClickConfig,
    press: Option<Press>,
    last_click: Option<ClickRecord>,
    unconfirmed_click: Option<ClickRecord>,
    queue: OutputQueue,
}

impl ClickRecognizer {
    /// A recognizer with `config`.
    pub fn new(config: ClickConfig) -> Self {
        Self {
            config,
            ..Self::default()
        }
    }

    fn click_count(&self, press: &Press) -> u8 {
        self.last_click
            .filter(|last| {
                press.started.saturating_sub(last.released) <= self.config.multi_click_interval
                    && (press.origin - last.position).length() <= self.config.multi_click_distance
            })
            .map_or(1, |last| last.count % MAX_CLICK_COUNT + 1)
    }

    fn fail(&mut self) -> Vote {
        self.press = None;
        Vote::Reject
    }
}

impl Recognizer for ClickRecognizer {
    fn allows_pointer(&self, event: &PointerEvent) -> bool {
        self.press.is_none() && event.button == Some(self.config.button)
    }

    fn pointer_down(&mut self, event: &PointerEvent) -> Vote {
        if !self.allows_pointer(event) {
            return Vote::Reject;
        }
        self.queue.begin();
        self.unconfirmed_click = None;
        self.press = Some(Press::new(event, self.config.button));
        Vote::Pending
    }

    fn pointer_move(&mut self, event: &PointerEvent) -> Vote {
        match self.press {
            Some(press)
                if press.follows(event) && press.exceeds_slop(event.position, self.config.slop) =>
            {
                self.fail()
            }
            _ => Vote::Pending,
        }
    }

    fn pointer_up(&mut self, event: &PointerEvent) -> Vote {
        let Some(press) = self.press.filter(|press| press.is_released_by(event)) else {
            return Vote::Pending;
        };
        self.press = None;
        if press.exceeds_slop(event.position, self.config.slop)
            || press.held_for(event.timestamp) >= self.config.max_press_duration
        {
            return Vote::Reject;
        }
        let click = ClickRecord {
            released: event.timestamp,
            position: event.position,
            count: self.click_count(&press),
        };
        self.queue.push(GestureOutput::Click {
            position: click.position,
            count: click.count,
        });
        if self.queue.is_won() {
            self.last_click = Some(click);
        } else {
            self.unconfirmed_click = Some(click);
        }
        Vote::Accept
    }

    fn pointer_cancel(&mut self) {
        self.rejected();
    }

    fn poll(&mut self, now: Duration) -> Vote {
        match self.press {
            Some(press) if press.held_for(now) >= self.config.max_press_duration => self.fail(),
            _ => Vote::Pending,
        }
    }

    fn accepted(&mut self) {
        self.queue.accept();
        if let Some(click) = self.unconfirmed_click.take() {
            self.last_click = Some(click);
        }
    }

    fn rejected(&mut self) {
        self.press = None;
        self.unconfirmed_click = None;
        self.queue.reject();
    }

    fn drain_outputs(&mut self, out: &mut Vec<GestureOutput>) {
        self.queue.drain_into(out);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{at, down, drain, pressed};

    fn click(recognizer: &mut ClickRecognizer, x: f32, down_millis: u64, up_millis: u64) -> Vote {
        assert_eq!(
            recognizer.pointer_down(&down(x, 0.0, down_millis)),
            Vote::Pending
        );
        let vote = recognizer.pointer_up(&down(x, 0.0, up_millis));
        if vote == Vote::Accept {
            recognizer.accepted();
        }
        vote
    }

    fn counts(recognizer: &mut ClickRecognizer) -> Vec<u8> {
        drain(recognizer)
            .into_iter()
            .filter_map(|output| match output {
                GestureOutput::Click { count, .. } => Some(count),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn release_within_slop_accepts_and_clicks_after_winning() {
        let mut recognizer = ClickRecognizer::default();
        recognizer.pointer_down(&down(10.0, 10.0, 0));
        assert_eq!(recognizer.pointer_move(&at(13.0, 12.0, 30)), Vote::Pending);
        assert_eq!(recognizer.pointer_up(&down(13.0, 12.0, 60)), Vote::Accept);
        assert!(drain(&mut recognizer).is_empty());
        recognizer.accepted();
        assert_eq!(
            drain(&mut recognizer),
            [GestureOutput::Click {
                position: Point::new(13.0, 12.0),
                count: 1
            }]
        );
    }

    #[test]
    fn moving_beyond_slop_rejects() {
        let mut recognizer = ClickRecognizer::default();
        recognizer.pointer_down(&down(0.0, 0.0, 0));
        assert_eq!(recognizer.pointer_move(&at(7.0, 0.0, 10)), Vote::Reject);
        assert_eq!(recognizer.pointer_up(&down(0.0, 0.0, 20)), Vote::Pending);
    }

    #[test]
    fn long_press_is_not_a_click() {
        let mut recognizer = ClickRecognizer::default();
        recognizer.pointer_down(&down(0.0, 0.0, 0));
        assert_eq!(recognizer.poll(Duration::from_millis(499)), Vote::Pending);
        assert_eq!(recognizer.poll(Duration::from_millis(500)), Vote::Reject);

        recognizer.pointer_down(&down(0.0, 0.0, 1000));
        assert_eq!(recognizer.pointer_up(&down(0.0, 0.0, 1600)), Vote::Reject);
    }

    #[test]
    fn rejected_click_is_discarded_and_not_counted() {
        let mut recognizer = ClickRecognizer::default();
        recognizer.pointer_down(&down(0.0, 0.0, 0));
        recognizer.pointer_up(&down(0.0, 0.0, 50));
        recognizer.rejected();
        assert!(drain(&mut recognizer).is_empty());
        click(&mut recognizer, 0.0, 100, 150);
        assert_eq!(counts(&mut recognizer), [1]);
    }

    #[test]
    fn quick_clicks_count_to_three_then_wrap() {
        let mut recognizer = ClickRecognizer::default();
        for (index, start) in [0, 200, 400, 600].into_iter().enumerate() {
            assert_eq!(
                click(&mut recognizer, index as f32, start, start + 50),
                Vote::Accept
            );
        }
        assert_eq!(counts(&mut recognizer), [1, 2, 3, 1]);
    }

    #[test]
    fn slow_or_distant_clicks_restart_the_count() {
        let mut recognizer = ClickRecognizer::default();
        click(&mut recognizer, 0.0, 0, 50);
        click(&mut recognizer, 0.0, 500, 550);
        click(&mut recognizer, 20.0, 600, 650);
        assert_eq!(counts(&mut recognizer), [1, 1, 1]);
    }

    #[test]
    fn secondary_button_configuration_ignores_primary() {
        let mut recognizer = ClickRecognizer::new(ClickConfig {
            button: PointerButton::Secondary,
            ..ClickConfig::default()
        });
        assert!(!recognizer.allows_pointer(&down(0.0, 0.0, 0)));
        assert_eq!(recognizer.pointer_down(&down(0.0, 0.0, 0)), Vote::Reject);
        let right = |millis| pressed(0.0, 0.0, millis, PointerButton::Secondary);
        assert_eq!(recognizer.pointer_down(&right(10)), Vote::Pending);
        assert_eq!(recognizer.pointer_up(&right(40)), Vote::Accept);
    }

    #[test]
    fn winning_before_release_clicks_on_release() {
        let mut recognizer = ClickRecognizer::default();
        recognizer.pointer_down(&down(0.0, 0.0, 0));
        recognizer.accepted();
        recognizer.pointer_up(&down(1.0, 0.0, 40));
        assert_eq!(counts(&mut recognizer), [1]);
    }
}
