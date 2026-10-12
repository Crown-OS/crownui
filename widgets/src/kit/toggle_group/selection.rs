use reactive::{Runtime, Signal};
use view::Cx;

#[derive(Clone, Copy)]
pub(super) struct Selection {
    signal: Signal<usize>,
    last: usize,
}

impl Selection {
    pub(super) fn new(signal: Signal<usize>, count: usize) -> Self {
        Self {
            signal,
            last: count.saturating_sub(1),
        }
    }

    pub(super) fn get(self, runtime: &mut Runtime) -> usize {
        self.signal.get(runtime).min(self.last)
    }

    pub(super) fn choose(self, cx: &mut Cx, index: usize) {
        let index = index.min(self.last);
        if self.get(cx) != index {
            self.signal.set(cx, index);
        }
    }

    pub(super) fn previous(self, runtime: &mut Runtime) -> usize {
        self.get(runtime).saturating_sub(1)
    }

    pub(super) fn next(self, runtime: &mut Runtime) -> usize {
        (self.get(runtime) + 1).min(self.last)
    }

    pub(super) fn last(self) -> usize {
        self.last
    }
}
