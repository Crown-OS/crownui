use std::time::Duration;

use motion::{Animation, Curve};
use reactive::{Runtime, ScopeId, Signal};
use view::Cx;

/// A number that springs toward the targets it is given, one animation frame at a time, for
/// layout properties that `.transition(..)` snaps (it animates where nodes sit, not how big
/// they are).
#[derive(Clone, Copy)]
pub(crate) struct SpringValue {
    value: Signal<f32>,
    target: Signal<f32>,
    animation: Signal<Option<Animation<f32>>>,
    ticking: Signal<bool>,
    curve: Curve,
    scope: ScopeId,
}

impl SpringValue {
    pub(crate) fn new(cx: &mut Cx, initial: f32, curve: impl Into<Curve>) -> Self {
        Self {
            value: cx.signal(initial),
            target: cx.signal(initial),
            animation: cx.signal(None),
            ticking: cx.signal(false),
            curve: curve.into(),
            scope: cx.current_scope(),
        }
    }

    pub(crate) fn get(self, runtime: &mut Runtime) -> f32 {
        self.value.get(runtime)
    }

    pub(crate) fn animate_to(self, cx: &mut Cx, target: f32) {
        self.target.set(cx, target);
        if !self.ticking.get(cx) {
            self.ticking.set(cx, true);
            self.schedule(cx);
        }
    }

    fn schedule(self, cx: &mut Cx) {
        let caller = cx.set_current_scope(self.scope);
        cx.request_animation_frame(move |cx, now| self.tick(cx, now));
        cx.set_current_scope(caller);
    }

    fn tick(self, cx: &mut Cx, now: Duration) {
        let target = self.target.get(cx);
        let animation = match self.animation.get(cx) {
            Some(running) if running.target() == target => running,
            Some(running) => running.retarget(now, target, self.curve),
            None => Animation::new(self.value.get(cx), target, 0.0, now, self.curve),
        };
        let sample = animation.sample(now);
        self.value.set(cx, sample.value);
        if sample.finished {
            self.animation.set(cx, None);
            self.ticking.set(cx, false);
        } else {
            self.animation.set(cx, Some(animation));
            self.schedule(cx);
        }
    }
}
