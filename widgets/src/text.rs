use view::{Cx, Element, IntoProp, Kind, NodeId, Prop};

/// The kind behind [`text`].
pub struct TextKind {
    content: Prop<String>,
}

impl Kind for TextKind {
    const NAME: &'static str = "text";

    fn mount(self, cx: &mut Cx, node: NodeId) {
        cx.set_text(node, self.content);
    }
}

/// A run of text. Style it with `.text_size()`, `.text_color()`, `.font_semibold()` …; unset text
/// properties inherit from ancestors like CSS.
pub fn text(content: impl IntoProp<String>) -> Element<TextKind> {
    Element::new(TextKind {
        content: content.into_prop(),
    })
}
