mod fill;
mod kind;
mod options;
mod radii;

pub use fill::PaneFill;
pub use kind::PaneKind;
pub use options::PaneOptions;
pub use radii::Radii;

use view::{Element, ViewSeq};

/// A panel behind `content`, unfilled and square until [`PaneOptions`] make it a surface; content is clipped to its corners.
pub fn pane<C: ViewSeq>(content: C) -> Element<PaneKind<C>> {
    Element::new(PaneKind {
        content,
        fill: Box::new(|_| PaneFill::None),
        radii: Radii::default(),
        border: None,
    })
}
