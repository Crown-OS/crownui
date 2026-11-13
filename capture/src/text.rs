use std::sync::Arc;

use geometry::Size;
use graphics::{GraphicsError, TextLayout, TextStyle, TextSystem};
use impeller::{ImpellerTextLayout, ImpellerTextSystem};

/// What a text layout was shaped from, kept so captures can rebuild it as live text.
#[derive(Debug)]
pub struct TextSource {
    /// The shaped string.
    pub text: String,
    /// Its style.
    pub style: TextStyle,
    /// The width lines were broken at, if any.
    pub wrap_width: Option<f32>,
}

/// An Impeller layout plus its source.
#[derive(Debug, Clone)]
pub struct CapturedText {
    pub(crate) layout: ImpellerTextLayout,
    pub(crate) source: Arc<TextSource>,
}

impl CapturedText {
    /// What the layout was shaped from.
    pub fn source(&self) -> &TextSource {
        &self.source
    }
}

impl TextLayout for CapturedText {
    fn size(&self) -> Size {
        self.layout.size()
    }

    fn min_intrinsic_width(&self) -> f32 {
        self.layout.min_intrinsic_width()
    }

    fn max_intrinsic_width(&self) -> f32 {
        self.layout.max_intrinsic_width()
    }

    fn line_count(&self) -> usize {
        self.layout.line_count()
    }

    fn first_baseline(&self) -> f32 {
        self.layout.first_baseline()
    }

    fn memory_bytes(&self) -> usize {
        self.layout.memory_bytes() + self.source.text.len()
    }
}

/// Impeller's real text shaping (system fonts, Inter), remembering each layout's source.
#[derive(Debug, Default)]
pub struct CaptureTextSystem {
    shaper: ImpellerTextSystem,
}

impl TextSystem for CaptureTextSystem {
    type Layout = CapturedText;

    fn register_font(&mut self, bytes: Vec<u8>, alias: Option<&str>) -> Result<(), GraphicsError> {
        self.shaper.register_font(bytes, alias)
    }

    fn layout(&mut self, text: &str, style: &TextStyle, max_width: Option<f32>) -> CapturedText {
        CapturedText {
            layout: self.shaper.layout(text, style, max_width),
            source: Arc::new(TextSource {
                text: text.to_owned(),
                style: style.clone(),
                wrap_width: max_width.filter(|width| width.is_finite()),
            }),
        }
    }
}
