use geometry::Color;
use reactive::{Memo, Signal};
use view::{AnyView, Cx, Element, IntoProp, Prop, Styled};

use super::metrics::LINE_HEIGHT;
use crate::{
    kit::Theme,
    text::{TextKind, text},
};

/// Builds a row's leading glyph in the color the row wants it, which follows the theme and, on a
/// selectable row, its selection.
pub type PanelGlyph = Box<dyn FnOnce(Memo<Color>) -> AnyView>;

pub(super) fn role_text(
    content: impl IntoProp<String>,
    (size, weight): (f32, u16),
) -> Element<TextKind> {
    text(content)
        .text_size(size)
        .font_weight(weight)
        .leading(LINE_HEIGHT)
}

pub(super) fn memo_of<T: Clone + PartialEq + 'static>(cx: &mut Cx, prop: Prop<T>) -> Memo<T> {
    match prop {
        Prop::Static(value) => cx.memo(move |_| value.clone()),
        Prop::Dynamic(mut compute) => cx.memo(move |runtime| compute(runtime)),
    }
}

pub(super) fn theme_color(
    cx: &mut Cx,
    theme: Signal<Theme>,
    pick: fn(&Theme) -> Color,
) -> Memo<Color> {
    cx.memo(move |runtime| theme.with(runtime, pick))
}
