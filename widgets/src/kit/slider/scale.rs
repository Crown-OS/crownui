use input::NamedKey;

use super::metrics::{DEFAULT_STEPS, PAGE_MULTIPLIER, SHIFT_MULTIPLIER};

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct SliderScale {
    pub(super) min: f32,
    pub(super) max: f32,
    pub(super) step: Option<f32>,
}

impl SliderScale {
    pub(super) const DEFAULT: Self = Self {
        min: 0.0,
        max: 1.0,
        step: None,
    };

    fn span(self) -> f32 {
        (self.max - self.min).abs().max(f32::EPSILON)
    }

    pub(super) fn keyboard_step(self) -> f32 {
        self.step.unwrap_or(self.span() / DEFAULT_STEPS)
    }

    fn lowest(self) -> f32 {
        self.min.min(self.max)
    }

    fn highest(self) -> f32 {
        self.min.max(self.max)
    }

    pub(super) fn settle(self, value: f32) -> f32 {
        let snapped = match self.step {
            Some(step) if step > 0.0 => {
                self.lowest() + ((value - self.lowest()) / step).round() * step
            }
            _ => value,
        };
        snapped.clamp(self.lowest(), self.highest())
    }

    pub(super) fn progress(self, value: f32) -> f32 {
        if self.max == self.min {
            return 0.0;
        }
        ((value - self.min) / (self.max - self.min)).clamp(0.0, 1.0)
    }

    pub(super) fn value_at(self, progress: f32) -> f32 {
        self.settle(self.min + progress.clamp(0.0, 1.0) * (self.max - self.min))
    }

    pub(super) fn nudged(self, value: f32, steps: f32) -> f32 {
        self.settle(value + steps * self.keyboard_step())
    }

    pub(super) fn after_key(self, value: f32, key: NamedKey, shift: bool) -> Option<f32> {
        let arrow = if shift { SHIFT_MULTIPLIER } else { 1.0 };
        let next = match key {
            NamedKey::ArrowRight | NamedKey::ArrowUp => self.nudged(value, arrow),
            NamedKey::ArrowLeft | NamedKey::ArrowDown => self.nudged(value, -arrow),
            NamedKey::PageUp => self.nudged(value, PAGE_MULTIPLIER),
            NamedKey::PageDown => self.nudged(value, -PAGE_MULTIPLIER),
            NamedKey::Home => self.min,
            NamedKey::End => self.max,
            _ => return None,
        };
        Some(next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PERCENT: SliderScale = SliderScale {
        min: 0.0,
        max: 100.0,
        step: None,
    };

    #[test]
    fn progress_and_value_are_inverses_inside_the_range() {
        assert_eq!(PERCENT.progress(25.0), 0.25);
        assert_eq!(PERCENT.value_at(0.25), 25.0);
        assert_eq!(PERCENT.progress(-5.0), 0.0);
        assert_eq!(PERCENT.progress(500.0), 1.0);
    }

    #[test]
    fn a_degenerate_range_sits_at_zero_progress() {
        let flat = SliderScale {
            min: 3.0,
            max: 3.0,
            step: None,
        };
        assert_eq!(flat.progress(3.0), 0.0);
        assert_eq!(flat.value_at(0.7), 3.0);
    }

    #[test]
    fn arrows_move_a_hundredth_of_the_range_and_shift_moves_ten_times_that() {
        assert_eq!(
            PERCENT.after_key(50.0, NamedKey::ArrowRight, false),
            Some(51.0)
        );
        assert_eq!(
            PERCENT.after_key(50.0, NamedKey::ArrowDown, false),
            Some(49.0)
        );
        assert_eq!(PERCENT.after_key(50.0, NamedKey::ArrowUp, true), Some(60.0));
    }

    #[test]
    fn home_end_and_pages_jump_and_everything_clamps() {
        assert_eq!(PERCENT.after_key(50.0, NamedKey::Home, false), Some(0.0));
        assert_eq!(PERCENT.after_key(50.0, NamedKey::End, false), Some(100.0));
        assert_eq!(PERCENT.after_key(50.0, NamedKey::PageUp, false), Some(60.0));
        assert_eq!(PERCENT.nudged(99.5, 10.0), 100.0);
        assert_eq!(PERCENT.after_key(50.0, NamedKey::Tab, false), None);
    }

    #[test]
    fn a_step_snaps_pointer_values_and_sets_the_keyboard_increment() {
        let stepped = SliderScale {
            min: 0.0,
            max: 10.0,
            step: Some(2.0),
        };
        assert_eq!(stepped.value_at(0.33), 4.0);
        assert_eq!(
            stepped.after_key(4.0, NamedKey::ArrowRight, false),
            Some(6.0)
        );
        assert_eq!(stepped.after_key(4.0, NamedKey::PageUp, false), Some(10.0));
    }
}
