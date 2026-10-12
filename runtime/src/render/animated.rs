use std::time::Duration;

use geometry::{Color, CornerRadii, TRANSPARENT, Vector};
use motion::{Animatable, Animation, Transaction};
use view::{Shadow, Visual};

/// A value that jumps or animates toward targets, retargeting smoothly.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Animated<T: Animatable> {
    current: T,
    animation: Option<Animation<T>>,
}

impl<T: Animatable> Animated<T> {
    pub(crate) fn new(value: T) -> Self {
        Self {
            current: value,
            animation: None,
        }
    }

    pub(crate) fn current(&self) -> T {
        self.current
    }

    pub(crate) fn target(&self) -> T {
        self.animation
            .as_ref()
            .map_or(self.current, Animation::target)
    }

    pub(crate) fn set(&mut self, target: T, transaction: Option<Transaction>, now: Duration) {
        match transaction {
            None => {
                self.current = target;
                self.animation = None;
            }
            Some(transaction) if target != self.target() => {
                self.animation = Some(match &self.animation {
                    Some(running) => running.retarget(now, target, transaction.curve),
                    None => Animation::new(
                        self.current,
                        target,
                        self.current.sub(self.current),
                        now,
                        transaction.curve,
                    ),
                });
            }
            Some(_) => {}
        }
    }

    /// Advances to `now`; returns whether the value is still moving.
    pub(crate) fn tick(&mut self, now: Duration) -> bool {
        let Some(animation) = &self.animation else {
            return false;
        };
        let sample = animation.sample(now);
        self.current = sample.value;
        if sample.finished {
            self.animation = None;
        }
        !sample.finished
    }

    pub(crate) fn is_animating(&self) -> bool {
        self.animation.is_some()
    }
}

/// Every animatable number of a [`Visual`], flattened so one spring moves them together.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct VisualValues {
    pub(crate) background: Color,
    pub(crate) border_width: f32,
    pub(crate) border_color: Color,
    pub(crate) ring_width: f32,
    pub(crate) ring_color: Color,
    pub(crate) radii: CornerRadii,
    pub(crate) shadow_color: Color,
    pub(crate) shadow_blur: f32,
    pub(crate) shadow_offset: Vector,
    pub(crate) opacity: f32,
    pub(crate) translate: Vector,
    pub(crate) scale: Vector,
    pub(crate) rotate: f32,
    pub(crate) backdrop_blur: f32,
    pub(crate) content_offset: Vector,
}

impl VisualValues {
    pub(crate) fn from_visual(visual: &Visual) -> Self {
        let shadow = visual
            .shadow
            .unwrap_or(Shadow::new(TRANSPARENT, 0.0, Vector::ZERO));
        let (ring_width, ring_color) = visual.ring.unwrap_or((0.0, TRANSPARENT));
        Self {
            background: visual.background.unwrap_or(TRANSPARENT),
            border_width: visual.border_width,
            border_color: visual.border_color,
            ring_width,
            ring_color,
            radii: visual.radii,
            shadow_color: shadow.color,
            shadow_blur: shadow.blur,
            shadow_offset: shadow.offset,
            opacity: visual.opacity,
            translate: visual.translate,
            scale: visual.scale,
            rotate: visual.rotate,
            backdrop_blur: visual.backdrop_blur,
            content_offset: visual.content_offset,
        }
    }

    /// Whether only composite values (transform, opacity, scroll) differ from `other`.
    pub(crate) fn paint_equals(&self, other: &Self) -> bool {
        self.background == other.background
            && self.border_width == other.border_width
            && self.border_color == other.border_color
            && self.ring_width == other.ring_width
            && self.ring_color == other.ring_color
            && self.radii == other.radii
            && self.shadow_color == other.shadow_color
            && self.shadow_blur == other.shadow_blur
            && self.shadow_offset == other.shadow_offset
    }
}

macro_rules! fieldwise {
    ($self:ident, $other:ident, $op:ident) => {
        Self {
            background: $self.background.$op($other.background),
            border_width: $self.border_width.$op($other.border_width),
            border_color: $self.border_color.$op($other.border_color),
            ring_width: $self.ring_width.$op($other.ring_width),
            ring_color: $self.ring_color.$op($other.ring_color),
            radii: $self.radii.$op($other.radii),
            shadow_color: $self.shadow_color.$op($other.shadow_color),
            shadow_blur: $self.shadow_blur.$op($other.shadow_blur),
            shadow_offset: $self.shadow_offset.$op($other.shadow_offset),
            opacity: $self.opacity.$op($other.opacity),
            translate: $self.translate.$op($other.translate),
            scale: $self.scale.$op($other.scale),
            rotate: $self.rotate.$op($other.rotate),
            backdrop_blur: $self.backdrop_blur.$op($other.backdrop_blur),
            content_offset: $self.content_offset.$op($other.content_offset),
        }
    };
}

impl Animatable for VisualValues {
    fn add(self, other: Self) -> Self {
        fieldwise!(self, other, add)
    }

    fn sub(self, other: Self) -> Self {
        fieldwise!(self, other, sub)
    }

    fn scale(self, factor: f32) -> Self {
        Self {
            background: self.background.scale(factor),
            border_width: self.border_width * factor,
            border_color: self.border_color.scale(factor),
            ring_width: self.ring_width * factor,
            ring_color: self.ring_color.scale(factor),
            radii: self.radii.scale(factor),
            shadow_color: self.shadow_color.scale(factor),
            shadow_blur: self.shadow_blur * factor,
            shadow_offset: self.shadow_offset.scale(factor),
            opacity: self.opacity * factor,
            translate: self.translate.scale(factor),
            scale: self.scale.scale(factor),
            rotate: self.rotate * factor,
            backdrop_blur: self.backdrop_blur * factor,
            content_offset: self.content_offset.scale(factor),
        }
    }

    fn magnitude(self) -> f32 {
        [
            self.background.magnitude(),
            self.border_width.abs(),
            self.border_color.magnitude(),
            self.ring_width.abs(),
            self.ring_color.magnitude(),
            self.radii.magnitude(),
            self.shadow_color.magnitude(),
            self.shadow_blur.abs(),
            self.shadow_offset.magnitude(),
            self.opacity.abs(),
            self.translate.magnitude(),
            self.scale.magnitude(),
            self.rotate.abs(),
            self.backdrop_blur.abs(),
            self.content_offset.magnitude(),
        ]
        .into_iter()
        .fold(0.0, f32::max)
    }
}

#[cfg(test)]
mod tests {
    use motion::Spring;

    use super::*;

    #[test]
    fn retargeting_mid_flight_keeps_moving_from_the_current_value() {
        let mut value = Animated::new(0.0_f32);
        value.set(
            100.0,
            Some(Transaction::new(Spring::snappy())),
            Duration::ZERO,
        );
        value.tick(Duration::from_millis(50));
        let midway = value.current();
        value.set(
            0.0,
            Some(Transaction::new(Spring::snappy())),
            Duration::from_millis(50),
        );
        value.tick(Duration::from_millis(50));
        assert_eq!(value.current(), midway);
        assert!(value.tick(Duration::from_millis(60)));
        assert!(!value.tick(Duration::from_secs(5)));
        assert_eq!(value.current(), 0.0);
    }

    #[test]
    fn untransacted_changes_jump() {
        let mut value = Animated::new(1.0_f32);
        value.set(5.0, None, Duration::ZERO);
        assert_eq!(value.current(), 5.0);
        assert!(!value.is_animating());
    }
}
