use reactive::Runtime;
use view::Prop;

/// What `prop` holds now; a dynamic prop subscribes the running observer.
pub(crate) fn current<T: Copy>(prop: &mut Prop<T>, runtime: &mut Runtime) -> T {
    match prop {
        Prop::Static(value) => *value,
        Prop::Dynamic(compute) => compute(runtime),
    }
}
