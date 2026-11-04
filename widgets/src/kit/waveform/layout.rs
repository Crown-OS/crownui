use layouting::{Align, Dimension, Position};
use view::{Cx, NodeId, SizeProperty, StyleProp};

use super::metrics::{HEIGHT, MIN_WIDTH};

/// Both waveforms fill their column's width at a fixed height unless restyled.
pub(super) fn set_default_size(cx: &mut Cx, node: NodeId) {
    for prop in [
        StyleProp::Position(Position::Relative),
        StyleProp::AlignSelf(Align::Stretch),
        StyleProp::Size(SizeProperty::Width, Dimension::Percent(1.0)),
        StyleProp::Size(SizeProperty::MinWidth, Dimension::Length(MIN_WIDTH)),
        StyleProp::Size(SizeProperty::Height, Dimension::Length(HEIGHT)),
        StyleProp::Shrink(1.0),
    ] {
        cx.set_default_style(node, prop);
    }
}
