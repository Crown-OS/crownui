use reactive::Memo;
use view::{Cx, Prop};

/// A prop several nodes of one widget read from: evaluated once per change, readable many times.
pub(crate) fn shared_prop<T: PartialEq + Clone + 'static>(cx: &mut Cx, prop: Prop<T>) -> Memo<T> {
    match prop {
        Prop::Static(value) => cx.memo(move |_| value.clone()),
        Prop::Dynamic(mut compute) => cx.memo(move |runtime| compute(runtime)),
    }
}
