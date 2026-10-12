use geometry::Color;
use layouting::{Align, Justify};
use semantics::Role;
use view::{Corners, Cx, Edges, Element, IntoProp, Kind, NodeId, Prop, StyleProp};
use windowing::CursorIcon;

use crate::text::text;

const BUTTON_BACKGROUND: Color = Color::new(0.5, 0.5, 0.5, 0.16);

/// The kind behind [`button`].
pub struct ButtonKind {
    label: Prop<String>,
}

impl Kind for ButtonKind {
    const NAME: &'static str = "button";

    fn mount(self, cx: &mut Cx, node: NodeId) {
        cx.set_role(node, Role::Button);
        for prop in [
            StyleProp::FlexDirection(layouting::FlexDirection::Row),
            StyleProp::AlignItems(Align::Center),
            StyleProp::Justify(Justify::Center),
            StyleProp::Padding(Edges::Horizontal, 12.0),
            StyleProp::Padding(Edges::Vertical, 6.0),
            StyleProp::Radius(Corners::All, 8.0),
            StyleProp::Background(Some(BUTTON_BACKGROUND)),
            StyleProp::Cursor(CursorIcon::Pointer),
        ] {
            cx.set_default_style(node, prop);
        }
        cx.set_children(node, text(self.label));
    }
}

/// A push button showing `label`; attach behavior with `.on_click(..)`. Keyboard users activate
/// it with Enter or Space once focused.
pub fn button(label: impl IntoProp<String>) -> Element<ButtonKind> {
    Element::new(ButtonKind {
        label: label.into_prop(),
    })
}
