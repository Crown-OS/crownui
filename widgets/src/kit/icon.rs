pub use blinc_icons::icons;
use geometry::Color;
use view::{Cx, Element, IntoProp, Kind, NodeId, Styled};

use super::theme::{Theme, use_theme};
use crate::{
    svg::{SvgKind, icon as svg_icon, mount_svg},
    util::themed::themed,
};

const DEFAULT_SIZE: f32 = 16.0;
const DEFAULT_STROKE_WIDTH: f32 = 2.0;
const VIEW_BOX_SIZE: f32 = 24.0;

/// The kind behind [`icon`]: a stroked Lucide glyph painted in a theme color.
pub struct IconKind {
    body: &'static str,
    stroke_width: f32,
    color: Box<dyn Fn(&Theme) -> Color>,
}

fn lucide_source(body: &str, stroke_width: f32) -> String {
    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{VIEW_BOX_SIZE}" height="{VIEW_BOX_SIZE}" viewBox="0 0 {VIEW_BOX_SIZE} {VIEW_BOX_SIZE}" fill="none" stroke="#000000" stroke-width="{stroke_width}" stroke-linecap="round" stroke-linejoin="round">{body}</svg>"##
    )
}

impl Kind for IconKind {
    const NAME: &'static str = "kit_icon";

    fn mount(self, cx: &mut Cx, node: NodeId) {
        let theme = use_theme(cx);
        let source = lucide_source(self.body, self.stroke_width);
        mount_svg(
            cx,
            node,
            &source,
            Some(themed(theme, self.color).into_prop()),
        );
    }
}

/// A Lucide glyph tinted by a caller-driven color, for widgets whose icon color is their own state.
pub(crate) fn tinted_icon(
    body: &str,
    stroke_width: f32,
    tint: impl IntoProp<Color>,
) -> Element<SvgKind> {
    svg_icon(lucide_source(body, stroke_width), tint).size(DEFAULT_SIZE)
}

/// A Lucide glyph from [`icons`] in the theme's icon color, 16px; resize with `.size(..)`.
pub fn icon(body: &'static str) -> Element<IconKind> {
    icon_colored(body, DEFAULT_STROKE_WIDTH, |theme| theme.text.icon)
}

/// A Lucide glyph in the theme's muted icon color.
pub fn icon_muted(body: &'static str) -> Element<IconKind> {
    icon_colored(body, DEFAULT_STROKE_WIDTH, |theme| theme.text.icon_muted)
}

/// A Lucide glyph with an explicit `stroke_width` (in the 24-unit view box), colored by `color`.
pub fn icon_colored(
    body: &'static str,
    stroke_width: f32,
    color: impl Fn(&Theme) -> Color + 'static,
) -> Element<IconKind> {
    Element::new(IconKind {
        body,
        stroke_width,
        color: Box::new(color),
    })
    .size(DEFAULT_SIZE)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_bundled_icon_set_is_available() {
        assert!(icons::CHECK.contains("<path"));
    }
}
