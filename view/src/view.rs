use crate::{Cx, NodeId};

/// Something that can be mounted into the element tree. Views are built once; reactivity comes
/// from signals bound to their properties.
pub trait View: 'static {
    /// Mounts the view and returns its root node.
    fn build(self, cx: &mut Cx) -> NodeId;

    /// Erases the view's type, e.g. to return different views from branches.
    fn boxed(self) -> AnyView
    where
        Self: Sized,
    {
        AnyView(Box::new(move |cx| self.build(cx)))
    }
}

/// A type-erased view.
pub struct AnyView(Box<dyn FnOnce(&mut Cx) -> NodeId>);

impl View for AnyView {
    fn build(self, cx: &mut Cx) -> NodeId {
        (self.0)(cx)
    }

    fn boxed(self) -> AnyView {
        self
    }
}

/// An ordered group of child views: a single view, a tuple, a `Vec` or an `Option`.
pub trait ViewSeq: 'static {
    /// Mounts every view and appends the roots to `children`.
    fn build_into(self, cx: &mut Cx, children: &mut Vec<NodeId>);
}

impl<V: View> ViewSeq for V {
    fn build_into(self, cx: &mut Cx, children: &mut Vec<NodeId>) {
        children.push(self.build(cx));
    }
}

impl<V: View> ViewSeq for Vec<V> {
    fn build_into(self, cx: &mut Cx, children: &mut Vec<NodeId>) {
        children.reserve(self.len());
        for view in self {
            children.push(view.build(cx));
        }
    }
}

impl<V: View> ViewSeq for Option<V> {
    fn build_into(self, cx: &mut Cx, children: &mut Vec<NodeId>) {
        if let Some(view) = self {
            children.push(view.build(cx));
        }
    }
}

impl ViewSeq for () {
    fn build_into(self, _cx: &mut Cx, _children: &mut Vec<NodeId>) {}
}

macro_rules! tuple_view_seq {
    ($($name:ident),+) => {
        impl<$($name: ViewSeq),+> ViewSeq for ($($name,)+) {
            #[allow(non_snake_case)]
            fn build_into(self, cx: &mut Cx, children: &mut Vec<NodeId>) {
                let ($($name,)+) = self;
                $($name.build_into(cx, children);)+
            }
        }
    };
}

tuple_view_seq!(A);
tuple_view_seq!(A, B);
tuple_view_seq!(A, B, C);
tuple_view_seq!(A, B, C, D);
tuple_view_seq!(A, B, C, D, E);
tuple_view_seq!(A, B, C, D, E, F);
tuple_view_seq!(A, B, C, D, E, F, G);
tuple_view_seq!(A, B, C, D, E, F, G, H);
tuple_view_seq!(A, B, C, D, E, F, G, H, I);
tuple_view_seq!(A, B, C, D, E, F, G, H, I, J);
tuple_view_seq!(A, B, C, D, E, F, G, H, I, J, K);
tuple_view_seq!(A, B, C, D, E, F, G, H, I, J, K, L);
tuple_view_seq!(A, B, C, D, E, F, G, H, I, J, K, L, M);
tuple_view_seq!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
tuple_view_seq!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
tuple_view_seq!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);
