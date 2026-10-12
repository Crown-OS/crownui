mod kind;
mod options;
mod segment;

pub use kind::SegmentedBarKind;
pub use options::SegmentedBarOptions;
pub use segment::Segment;

use view::{Element, Prop, Styled};

const DEFAULT_HEIGHT: f32 = 14.0;

/// One pill shared by proportional `segments`, filling its column's width; resize its thickness with `.h(..)`.
pub fn segmented_bar(segments: impl IntoIterator<Item = Segment>) -> Element<SegmentedBarKind> {
    Element::new(SegmentedBarKind {
        segments: Prop::Static(segments.into_iter().collect()),
    })
    .self_stretch()
    .shrink(0.0)
    .h(DEFAULT_HEIGHT)
}
