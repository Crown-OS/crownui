use geometry::Color;
use layouting::Dimension;
use semantics::Role;
use view::{Cx, IntoProp, Kind, NodeId, Prop, SizeProperty, StyleProp, Styled};

use crate::{
    kit::use_theme,
    stack::vstack,
    util::{theme_fade::theme_fade, themed::themed},
};

pub(super) const LINE_THICKNESS: f32 = 1.0;

/// The kind behind [`divider`](super::divider).
pub struct DividerKind {
    pub(super) color: Option<Prop<Color>>,
    pub(super) height: f32,
}

impl Kind for DividerKind {
    const NAME: &'static str = "kit_divider";

    fn mount(self, cx: &mut Cx, node: NodeId) {
        let theme = use_theme(cx);
        cx.set_role(node, Role::Separator);
        cx.set_default_style(
            node,
            StyleProp::Size(SizeProperty::Height, Dimension::Length(self.height)),
        );
        cx.set_default_style(node, StyleProp::Shrink(0.0));
        let color = self
            .color
            .unwrap_or_else(|| themed(theme, |theme| theme.surface.border).into_prop());
        let line = vstack(())
            .absolute()
            .left(0.0)
            .right(0.0)
            .top((self.height - LINE_THICKNESS) / 2.0)
            .h(LINE_THICKNESS)
            .bg(color)
            .transition(theme_fade());
        cx.set_children(node, line);
    }
}
