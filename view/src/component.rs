use crate::{Cx, NodeId, View};

/// A view with its own scope: signals, effects, tasks and listeners created while building it
/// live as long as the component stays mounted.
pub struct Component<F>(F);

/// Declares a component with local state:
/// `component(move |cx| { let count = cx.signal(0); button(...) })`.
pub fn component<V: View, F: FnOnce(&mut Cx) -> V + 'static>(build: F) -> Component<F> {
    Component(build)
}

impl<V: View, F: FnOnce(&mut Cx) -> V + 'static> View for Component<F> {
    fn build(self, cx: &mut Cx) -> NodeId {
        let scope = cx.create_scope();
        let previous = cx.set_current_scope(scope);
        let view = (self.0)(cx);
        let node = view.build(cx);
        cx.set_current_scope(previous);
        node
    }
}
