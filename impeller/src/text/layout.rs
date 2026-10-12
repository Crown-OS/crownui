use geometry::Size;
use graphics::TextLayout;
use impellers::Paragraph;

const ESTIMATED_PARAGRAPH_OVERHEAD_BYTES: usize = 1024;
const ESTIMATED_BYTES_PER_TEXT_BYTE: usize = 24;
const ESTIMATED_BYTES_PER_LINE: usize = 64;

/// A shaped Impeller paragraph with its metrics read once at layout time.
#[derive(Debug)]
pub struct ImpellerTextLayout {
    paragraph: Option<Paragraph>,
    size: Size,
    min_intrinsic_width: f32,
    max_intrinsic_width: f32,
    line_count: usize,
    first_baseline: f32,
    memory_bytes: usize,
}

impl ImpellerTextLayout {
    /// `width_is_layout_width` reports the paragraph width instead of the widest line, for text
    /// whose lines are positioned relative to it (centered, trailing, justified or RTL).
    pub(crate) fn new(paragraph: Paragraph, text_len: usize, width_is_layout_width: bool) -> Self {
        let line_count = paragraph.get_line_count() as usize;
        let layout_width = paragraph.get_max_width();
        let width = if width_is_layout_width && layout_width.is_finite() {
            layout_width
        } else {
            widest_line(&paragraph, line_count)
        };
        Self {
            size: Size::new(width, paragraph.get_height()),
            min_intrinsic_width: paragraph.get_min_intrinsic_width(),
            max_intrinsic_width: paragraph.get_max_intrinsic_width(),
            line_count,
            first_baseline: paragraph.get_alphabetic_baseline(),
            memory_bytes: ESTIMATED_PARAGRAPH_OVERHEAD_BYTES
                + text_len * ESTIMATED_BYTES_PER_TEXT_BYTE
                + line_count * ESTIMATED_BYTES_PER_LINE,
            paragraph: Some(paragraph),
        }
    }

    /// A layout that draws nothing, for text Impeller failed to shape.
    pub(crate) fn empty() -> Self {
        Self {
            paragraph: None,
            size: Size::ZERO,
            min_intrinsic_width: 0.0,
            max_intrinsic_width: 0.0,
            line_count: 0,
            first_baseline: 0.0,
            memory_bytes: 0,
        }
    }

    /// The shaped paragraph, unless shaping failed.
    pub fn paragraph(&self) -> Option<&Paragraph> {
        self.paragraph.as_ref()
    }

    pub(crate) fn into_paragraph(self) -> Option<Paragraph> {
        self.paragraph
    }
}

fn widest_line(paragraph: &Paragraph, line_count: usize) -> f32 {
    paragraph.get_line_metrics().map_or_else(
        || paragraph.get_longest_line_width(),
        |metrics| {
            (0..line_count)
                .map(|line| metrics.get_width(line) as f32)
                .fold(0.0, f32::max)
        },
    )
}

impl TextLayout for ImpellerTextLayout {
    fn size(&self) -> Size {
        self.size
    }

    fn min_intrinsic_width(&self) -> f32 {
        self.min_intrinsic_width
    }

    fn max_intrinsic_width(&self) -> f32 {
        self.max_intrinsic_width
    }

    fn line_count(&self) -> usize {
        self.line_count
    }

    fn first_baseline(&self) -> f32 {
        self.first_baseline
    }

    fn memory_bytes(&self) -> usize {
        self.memory_bytes
    }
}
