use geometry::Color;
use reactive::Signal;
use view::{Cx, IntoProp, NodeId, StyleProp, bind};

use crate::kit::Theme;

/// A color property that follows `theme`.
pub(crate) fn themed(
    theme: Signal<Theme>,
    pick: impl Fn(&Theme) -> Color + 'static,
) -> impl IntoProp<Color> {
    bind(move |runtime| theme.with(runtime, &pick))
}

/// Binds a style property of a kind's own node to `theme`.
pub(crate) fn bind_themed_style(
    cx: &mut Cx,
    node: NodeId,
    theme: Signal<Theme>,
    style: impl Fn(&Theme) -> StyleProp + 'static,
) {
    cx.bind_style(node, move |runtime| theme.with(runtime, &style));
}
