mod collapse;
mod crumb;
mod handlers;
mod kind;
mod options;
mod segment;

pub use kind::PathBarKind;
pub use options::PathBarOptions;
pub use segment::PathSegment;

use reactive::Signal;
use view::{Cx, Element, Semantic, Styled};

use crate::kit::{INTER_FAMILY, TEXT_SIZE};

use handlers::SegmentHandlers;

/// A breadcrumb of `segments`, the last one the current folder: hoverable pills between muted
/// chevrons, the first one optionally led by an icon (home). Clicking segment `index` runs
/// `on_select(cx, index)`. It takes the free width of its row; when the path does not fit, the
/// leading segments fold into an ellipsis that leads to the deepest folded one.
pub fn path_bar(
    segments: Signal<Vec<PathSegment>>,
    on_select: impl FnMut(&mut Cx, usize) + 'static,
) -> Element<PathBarKind> {
    Element::new(PathBarKind {
        segments,
        handlers: SegmentHandlers {
            select: Some(Box::new(on_select)),
            ..SegmentHandlers::default()
        },
    })
    .label("Path")
    .flex_row()
    .items_center()
    .grow(1.0)
    .shrink(1.0)
    .basis(0.0)
    .min_w(0.0)
    .overflow_hidden()
    .text_size(TEXT_SIZE)
    .font_family(INTER_FAMILY)
}
