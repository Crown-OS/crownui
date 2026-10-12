use graphics::{FontWeight, TextAlign, TextStyle};
use impellers::{FontStyle, ParagraphStyle, TextAlignment, TextDirection};

use crate::util::geometry::to_color;

const ELLIPSIS: &str = "…";

const WEIGHTS_BY_HUNDRED: [impellers::FontWeight; 9] = [
    impellers::FontWeight::Thin,
    impellers::FontWeight::ExtraLight,
    impellers::FontWeight::Light,
    impellers::FontWeight::Regular,
    impellers::FontWeight::Medium,
    impellers::FontWeight::SemiBold,
    impellers::FontWeight::Bold,
    impellers::FontWeight::ExtraBold,
    impellers::FontWeight::Black,
];

/// The Impeller weight nearest to a CSS weight, clamped to 100–900.
pub(crate) fn nearest_weight(weight: FontWeight) -> impellers::FontWeight {
    let hundreds =
        usize::from(weight.0.saturating_add(50) / 100).clamp(1, WEIGHTS_BY_HUNDRED.len());
    WEIGHTS_BY_HUNDRED[hundreds - 1]
}

fn alignment(align: TextAlign) -> TextAlignment {
    match align {
        TextAlign::Start => TextAlignment::Start,
        TextAlign::Center => TextAlignment::Center,
        TextAlign::End => TextAlignment::End,
        TextAlign::Justify => TextAlignment::Justify,
    }
}

/// Whether lines are placed relative to the paragraph width rather than its leading edge.
pub(crate) fn depends_on_layout_width(style: &TextStyle) -> bool {
    style.rtl || style.align != TextAlign::Start
}

/// Every [`TextStyle`] field Impeller supports; `letter_spacing` has no Impeller equivalent.
pub(crate) fn paragraph_style(style: &TextStyle) -> ParagraphStyle {
    let mut foreground = impellers::Paint::default();
    foreground.set_color(to_color(style.color));
    let mut converted = ParagraphStyle::default();
    converted
        .set_foreground(&foreground)
        .set_font_size(style.font_size)
        .set_font_weight(nearest_weight(style.font_weight))
        .set_font_style(if style.italic {
            FontStyle::Italic
        } else {
            FontStyle::Normal
        })
        .set_text_alignment(alignment(style.align))
        .set_text_direction(if style.rtl {
            TextDirection::RTL
        } else {
            TextDirection::LTR
        });
    if let Some(family) = &style.font_family {
        converted.set_font_family(family);
    }
    if let Some(line_height) = style.line_height {
        converted.set_height(line_height);
    }
    if let Some(max_lines) = style.max_lines {
        converted.set_max_lines(max_lines.max(1));
    }
    if style.ellipsis {
        converted.set_ellipsis(Some(ELLIPSIS));
    }
    converted
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn css_weights_map_to_the_nearest_impeller_weight() {
        assert_eq!(nearest_weight(FontWeight(0)), impellers::FontWeight::Thin);
        assert_eq!(
            nearest_weight(FontWeight::NORMAL),
            impellers::FontWeight::Regular
        );
        assert_eq!(
            nearest_weight(FontWeight(549)),
            impellers::FontWeight::Medium
        );
        assert_eq!(
            nearest_weight(FontWeight(550)),
            impellers::FontWeight::SemiBold
        );
        assert_eq!(
            nearest_weight(FontWeight::BOLD),
            impellers::FontWeight::Bold
        );
        assert_eq!(
            nearest_weight(FontWeight(u16::MAX)),
            impellers::FontWeight::Black
        );
    }
}
