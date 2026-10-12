use std::borrow::Cow;

use graphics::{GraphicsError, TextStyle, TextSystem};
use impellers::{Paragraph, ParagraphBuilder, ParagraphStyle, TypographyContext};

use super::{
    layout::ImpellerTextLayout,
    style::{depends_on_layout_width, paragraph_style},
};

/// Shapes text with one Impeller typography context (creating it scans the system fonts, so
/// keep a single instance per UI thread).
#[derive(Debug, Default)]
pub struct ImpellerTextSystem {
    typography: TypographyContext,
}

impl ImpellerTextSystem {
    /// Creates the typography context with the system fonts.
    pub fn new() -> Self {
        Self::default()
    }

    fn build(&self, text: &str, style: &ParagraphStyle, width: f32) -> Option<Paragraph> {
        let mut builder = ParagraphBuilder::new(&self.typography)?;
        builder.push_style(style).add_text(text);
        builder.build(width)
    }

    fn shape(
        &self,
        text: &str,
        style: &TextStyle,
        max_width: Option<f32>,
    ) -> Option<ImpellerTextLayout> {
        let converted = paragraph_style(style);
        let width = max_width.unwrap_or(f32::INFINITY);
        let paragraph = self.build(text, &converted, width)?;
        let width_is_layout_width = depends_on_layout_width(style);
        let fitted_width = paragraph.get_max_intrinsic_width().ceil();
        let paragraph = if width_is_layout_width && fitted_width < width {
            self.build(text, &converted, fitted_width)?
        } else {
            paragraph
        };
        Some(ImpellerTextLayout::new(
            paragraph,
            text.len(),
            width_is_layout_width,
        ))
    }
}

impl TextSystem for ImpellerTextSystem {
    type Layout = ImpellerTextLayout;

    fn register_font(&mut self, bytes: Vec<u8>, alias: Option<&str>) -> Result<(), GraphicsError> {
        self.typography
            .register_font(Cow::Owned(bytes), alias)
            .map_err(|message| GraphicsError::Font(message.to_owned()))
    }

    fn layout(
        &mut self,
        text: &str,
        style: &TextStyle,
        max_width: Option<f32>,
    ) -> ImpellerTextLayout {
        self.shape(text, style, max_width).unwrap_or_else(|| {
            tracing::warn!(text_len = text.len(), "Impeller failed to shape text");
            ImpellerTextLayout::empty()
        })
    }
}

#[cfg(test)]
mod tests {
    use graphics::{TextAlign, TextLayout};

    use super::*;

    const GREETING: &str = "Hello CrownUI";

    fn body_style() -> TextStyle {
        TextStyle {
            font_size: 17.0,
            ..TextStyle::default()
        }
    }

    #[test]
    fn text_metrics_are_sane_and_wrapping_and_truncation_work() {
        let mut text = ImpellerTextSystem::new();

        let single_line = text.layout(GREETING, &body_style(), None);
        let size = single_line.size();
        assert_eq!(single_line.line_count(), 1);
        assert!(
            size.width > 17.0 && size.width < 400.0,
            "width {}",
            size.width
        );
        assert!(
            size.height >= 17.0 && size.height < 60.0,
            "height {}",
            size.height
        );
        assert!((single_line.max_intrinsic_width() - size.width).abs() < 1.0);
        assert!(single_line.min_intrinsic_width() <= single_line.max_intrinsic_width());
        assert!(single_line.first_baseline() > 0.0 && single_line.first_baseline() < size.height);
        assert!(single_line.memory_bytes() > GREETING.len());

        let wrapped = text.layout(GREETING, &body_style(), Some(40.0));
        assert!(
            wrapped.line_count() > 1,
            "wrapped into {} lines",
            wrapped.line_count()
        );
        assert!(wrapped.size().height > size.height);

        let truncated_style = TextStyle {
            max_lines: Some(1),
            ellipsis: true,
            ..body_style()
        };
        let truncated = text.layout(GREETING, &truncated_style, Some(60.0));
        assert_eq!(truncated.line_count(), 1);
        assert!(truncated.size().width <= 60.0);
        assert!((truncated.size().height - size.height).abs() < 1.0);
    }

    #[test]
    fn styles_change_shaping() {
        let mut text = ImpellerTextSystem::new();
        let regular = text.layout(GREETING, &body_style(), None).size();
        let bold_style = TextStyle {
            font_weight: graphics::FontWeight::BOLD,
            ..body_style()
        };
        assert!(text.layout(GREETING, &bold_style, None).size().width > regular.width);
        let tall_style = TextStyle {
            line_height: Some(2.0),
            ..body_style()
        };
        assert!((text.layout(GREETING, &tall_style, None).size().height - 34.0).abs() < 1.0);
        let centered_style = TextStyle {
            align: TextAlign::Center,
            ..body_style()
        };
        let centered = text.layout(GREETING, &centered_style, None).size();
        assert!(
            (centered.width - regular.width.ceil()).abs() < 1.0,
            "centered {centered:?}"
        );
        assert!(
            text.register_font(b"not a font".to_vec(), Some("Broken"))
                .is_err()
        );
    }
}
