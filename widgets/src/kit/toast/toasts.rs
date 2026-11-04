use std::time::Duration;

use reactive::{ScopeId, Signal, Store};
use view::Cx;

use super::metrics::DEFAULT_DURATION;

/// One notification for [`Toasts::show_with`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Toast {
    /// What the toast says.
    pub message: String,
    /// A Lucide icon from [`icons`](crate::kit::icons) drawn before the message.
    pub icon: Option<&'static str>,
    /// How long it stays before it leaves.
    pub duration: Duration,
}

impl Toast {
    /// A toast saying `message` for three seconds.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            icon: None,
            duration: DEFAULT_DURATION,
        }
    }

    /// The same toast with a leading icon.
    pub fn icon(self, body: &'static str) -> Self {
        Self {
            icon: Some(body),
            ..self
        }
    }

    /// The same toast staying for `duration`.
    pub fn duration(self, duration: Duration) -> Self {
        Self { duration, ..self }
    }
}

/// A toast on screen, keyed by `id`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ShownToast {
    pub(super) id: u64,
    pub(super) message: String,
    pub(super) icon: Option<&'static str>,
}

#[derive(Debug, Clone, Copy)]
struct Timer {
    id: u64,
    duration: Duration,
    deadline: Option<Duration>,
}

/// The toasts of one window, shown by its [`toast_host`](super::toast_host): copy it into
/// handlers and call [`show`](Self::show).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Toasts {
    pub(super) shown: Signal<Vec<ShownToast>>,
    timers: Store<Vec<Timer>>,
    next_id: Store<u64>,
    ticking: Store<bool>,
    scope: ScopeId,
}

impl Toasts {
    fn new(cx: &mut Cx) -> Self {
        Self {
            shown: cx.signal(Vec::new()),
            timers: cx.store(Vec::new()),
            next_id: cx.store(0),
            ticking: cx.store(false),
            scope: cx.current_scope(),
        }
    }

    /// Shows `message` for three seconds.
    pub fn show(self, cx: &mut Cx, message: impl Into<String>) {
        self.show_with(cx, Toast::new(message));
    }

    /// Shows `toast` until its duration has passed.
    pub fn show_with(self, cx: &mut Cx, toast: Toast) {
        let id = self.next_id.update(cx, |next| {
            *next += 1;
            *next
        });
        self.timers.update(cx, |timers| {
            timers.push(Timer {
                id,
                duration: toast.duration,
                deadline: None,
            });
        });
        self.shown.update(cx, |shown| {
            shown.push(ShownToast {
                id,
                message: toast.message,
                icon: toast.icon,
            });
        });
        if !self
            .ticking
            .update(cx, |ticking| std::mem::replace(ticking, true))
        {
            self.schedule(cx);
        }
    }

    /// Takes every toast off the screen at once.
    pub fn dismiss_all(self, cx: &mut Cx) {
        self.timers.update(cx, Vec::clear);
        if self.shown.with_untracked(cx, |shown| !shown.is_empty()) {
            self.shown.set(cx, Vec::new());
        }
    }

    fn schedule(self, cx: &mut Cx) {
        let caller = cx.set_current_scope(self.scope);
        cx.request_animation_frame(move |cx, now| self.tick(cx, now));
        cx.set_current_scope(caller);
    }

    fn tick(self, cx: &mut Cx, now: Duration) {
        let expired: Vec<u64> = self.timers.update(cx, |timers| {
            for timer in timers.iter_mut() {
                timer.deadline.get_or_insert(now + timer.duration);
            }
            let expired = timers
                .iter()
                .filter(|timer| timer.deadline.is_some_and(|deadline| deadline <= now))
                .map(|timer| timer.id)
                .collect();
            timers.retain(|timer| timer.deadline.is_some_and(|deadline| deadline > now));
            expired
        });
        if !expired.is_empty() {
            self.shown.update(cx, |shown| {
                shown.retain(|toast| !expired.contains(&toast.id))
            });
        }
        if self.timers.with(cx, Vec::is_empty) {
            self.ticking.update(cx, |ticking| *ticking = false);
        } else {
            self.schedule(cx);
        }
    }
}

/// Provides a fresh [`Toasts`] to the current scope and everything built inside it, e.g. at the
/// root before building the [`toast_host`](super::toast_host).
pub fn provide_toasts(cx: &mut Cx) -> Toasts {
    let toasts = Toasts::new(cx);
    cx.provide(toasts);
    toasts
}

/// The nearest provided [`Toasts`], or a fresh one provided on the current scope.
pub fn use_toasts(cx: &mut Cx) -> Toasts {
    match cx.environment::<Toasts>() {
        Some(toasts) => toasts.with_untracked(cx, |toasts| *toasts),
        None => provide_toasts(cx),
    }
}
