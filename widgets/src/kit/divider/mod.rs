mod kind;
mod options;

pub use kind::DividerKind;
pub use options::DividerOptions;

use view::Element;

use kind::LINE_THICKNESS;

/// A 1px hairline in the surface border color, centered in its height (1px unless [`DividerOptions::height`] says otherwise).
pub fn divider() -> Element<DividerKind> {
    Element::new(DividerKind {
        color: None,
        height: LINE_THICKNESS,
    })
}
