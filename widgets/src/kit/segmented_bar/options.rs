use reactive::Runtime;
use view::{Element, Prop};

use super::{kind::SegmentedBarKind, segment::Segment};

/// The options of a [`segmented_bar`](super::segmented_bar), chained before its modifiers.
pub trait SegmentedBarOptions: Sized {
    /// Takes the runs from `compute` instead, recomputed whenever the signals it reads change.
    fn segments_from(self, compute: impl FnMut(&mut Runtime) -> Vec<Segment> + 'static) -> Self;
}

impl SegmentedBarOptions for Element<SegmentedBarKind> {
    fn segments_from(self, compute: impl FnMut(&mut Runtime) -> Vec<Segment> + 'static) -> Self {
        self.configure(|kind| kind.segments = Prop::Dynamic(Box::new(compute)))
    }
}
