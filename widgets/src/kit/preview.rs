use view::{AnyView, View, ViewSeq};

use crate::stack::vstack;
use view::Styled;

/// One entry of the component catalog that `cargo xtask preview` and `crowndev` render.
#[derive(Debug, Clone, Copy)]
pub struct Preview {
    /// Component name, as its constructor is called.
    pub name: &'static str,
    /// Builds the component in a representative state.
    pub build: fn() -> AnyView,
}

impl Preview {
    /// Builds the preview's view.
    pub fn view(&self) -> AnyView {
        (self.build)()
    }
}

/// The padded, left-aligned column every preview sits in.
pub(crate) fn stage(content: impl ViewSeq) -> AnyView {
    vstack(content).p(24.0).gap(12.0).items_start().boxed()
}
