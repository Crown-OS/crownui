use reactive::{Memo, Runtime, Signal};
use view::{Bind, IntoProp, Prop};

/// A zero-based step: a literal, a signal, a memo or a [`bind`](view::bind) closure.
pub trait IntoStep {
    /// Performs the conversion.
    fn into_step(self) -> Prop<usize>;
}

impl IntoStep for usize {
    fn into_step(self) -> Prop<usize> {
        Prop::Static(self)
    }
}

impl IntoStep for Signal<usize> {
    fn into_step(self) -> Prop<usize> {
        IntoProp::<usize>::into_prop(self)
    }
}

impl IntoStep for Memo<usize> {
    fn into_step(self) -> Prop<usize> {
        IntoProp::<usize>::into_prop(self)
    }
}

impl<F: FnMut(&mut Runtime) -> usize + 'static> IntoStep for Bind<F> {
    fn into_step(self) -> Prop<usize> {
        IntoProp::<usize>::into_prop(self)
    }
}
