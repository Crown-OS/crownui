use std::{collections::HashMap, fmt::Write};

use geometry::Size;

use crate::{
    BoundaryId, CompositeOp, Composition, CreateRenderer, DrawCommand, DrawList, GraphicsBackend,
    GraphicsError, ImageId, Renderer, TextLayout, TextLayoutId, TextStyle, TextSystem,
};

/// A backend with no GPU: frames are kept as text for snapshots and previews.
#[derive(Debug, Clone, Copy, Default)]
pub struct RecordingBackend;

/// Deterministic text measurement: every character is `0.6 × font size` wide.
#[derive(Debug, Default)]
pub struct RecordingTextSystem;

/// A layout produced by [`RecordingTextSystem`].
#[derive(Debug, Clone, PartialEq)]
pub struct RecordingTextLayout {
    text: String,
    lines: Vec<String>,
    size: Size,
    min_width: f32,
    max_width: f32,
    line_height: f32,
}

/// Keeps every boundary list and the last composition.
#[derive(Debug, Default)]
pub struct RecordingRenderer {
    boundaries: HashMap<BoundaryId, DrawList>,
    texts: HashMap<TextLayoutId, RecordingTextLayout>,
    images: HashMap<ImageId, (u32, u32)>,
    viewport: (u32, u32, f32),
    last_frame: Option<RecordedFrame>,
    frames_rendered: u64,
}

/// A rendered frame flattened to text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordedFrame {
    /// The composition with every boundary's commands inlined, and text contents resolved.
    pub text: String,
}

const CHARACTER_WIDTH_FACTOR: f32 = 0.6;
const DEFAULT_LINE_HEIGHT: f32 = 1.2;

impl RecordingTextLayout {
    /// The shaped text.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// The text of each line.
    pub fn lines(&self) -> &[String] {
        &self.lines
    }
}

impl TextLayout for RecordingTextLayout {
    fn size(&self) -> Size {
        self.size
    }

    fn min_intrinsic_width(&self) -> f32 {
        self.min_width
    }

    fn max_intrinsic_width(&self) -> f32 {
        self.max_width
    }

    fn line_count(&self) -> usize {
        self.lines.len()
    }

    fn first_baseline(&self) -> f32 {
        self.line_height * 0.8
    }

    fn memory_bytes(&self) -> usize {
        self.text.len() * 2 + self.lines.len() * std::mem::size_of::<String>()
    }
}

fn wrap(text: &str, character_width: f32, max_width: Option<f32>) -> Vec<String> {
    let Some(max_width) = max_width else {
        return text.lines().map(str::to_owned).collect();
    };
    let max_characters = ((max_width / character_width).floor() as usize).max(1);
    let mut lines = Vec::new();
    for paragraph in text.split('\n') {
        let mut line = String::new();
        for word in paragraph.split(' ') {
            let needed = if line.is_empty() { word.chars().count() } else { line.chars().count() + 1 + word.chars().count() };
            if needed > max_characters && !line.is_empty() {
                lines.push(std::mem::take(&mut line));
            }
            if !line.is_empty() {
                line.push(' ');
            }
            line.push_str(word);
        }
        lines.push(line);
    }
    lines
}

impl TextSystem for RecordingTextSystem {
    type Layout = RecordingTextLayout;

    fn register_font(&mut self, _bytes: Vec<u8>, _alias: Option<&str>) -> Result<(), GraphicsError> {
        Ok(())
    }

    fn layout(&mut self, text: &str, style: &TextStyle, max_width: Option<f32>) -> RecordingTextLayout {
        let character_width = style.font_size * CHARACTER_WIDTH_FACTOR + style.letter_spacing;
        let line_height = style.font_size * style.line_height.unwrap_or(DEFAULT_LINE_HEIGHT);
        let mut lines = wrap(text, character_width, max_width);
        if lines.is_empty() {
            lines.push(String::new());
        }
        if let Some(max_lines) = style.max_lines {
            lines.truncate(max_lines.max(1) as usize);
        }
        let width_of = |line: &str| line.chars().count() as f32 * character_width;
        let widest = lines.iter().map(|line| width_of(line)).fold(0.0, f32::max);
        let max_width_unbroken = text.lines().map(width_of).fold(0.0, f32::max);
        let longest_word = text.split_whitespace().map(width_of).fold(0.0, f32::max);
        RecordingTextLayout {
            text: text.to_owned(),
            size: Size::new(widest, line_height * lines.len() as f32),
            lines,
            min_width: longest_word,
            max_width: max_width_unbroken,
            line_height,
        }
    }
}

impl RecordingRenderer {
    /// The last rendered frame.
    pub fn last_frame(&self) -> Option<&RecordedFrame> {
        self.last_frame.as_ref()
    }

    /// Frames rendered so far.
    pub fn frames_rendered(&self) -> u64 {
        self.frames_rendered
    }

    /// The current viewport in device pixels and its scale.
    pub fn viewport(&self) -> (u32, u32, f32) {
        self.viewport
    }

    /// Images currently uploaded.
    pub fn image_count(&self) -> usize {
        self.images.len()
    }

    fn flatten(&self, composition: &Composition) -> String {
        let mut out = String::new();
        let mut depth = 0_usize;
        let [red, green, blue, alpha] = composition.clear_color.to_rgba8();
        let _ = writeln!(out, "clear #{red:02x}{green:02x}{blue:02x}{alpha:02x}");
        for op in &composition.ops {
            match op {
                CompositeOp::PushLayer(props) => {
                    let layer = Composition {
                        clear_color: composition.clear_color,
                        ops: vec![CompositeOp::PushLayer(*props)],
                    }
                    .to_string();
                    let description = layer.lines().nth(1).unwrap_or("layer").trim();
                    let _ = writeln!(out, "{:indent$}{description}", "", indent = depth * 2);
                    depth += 1;
                }
                CompositeOp::PopLayer => depth = depth.saturating_sub(1),
                CompositeOp::Draw(boundary) => {
                    let Some(list) = self.boundaries.get(boundary) else {
                        let _ = writeln!(out, "{:indent$}missing #{}", "", boundary.0, indent = depth * 2);
                        continue;
                    };
                    for command in list.commands() {
                        let line = match command {
                            DrawCommand::Text(id, origin) => match self.texts.get(id) {
                                Some(layout) => format!(
                                    "text {:?} at {},{}",
                                    layout.text,
                                    geometry::Length(origin.x),
                                    geometry::Length(origin.y)
                                ),
                                None => command.to_string(),
                            },
                            other => other.to_string(),
                        };
                        let _ = writeln!(out, "{:indent$}{line}", "", indent = depth * 2);
                    }
                }
            }
        }
        out
    }
}

impl Renderer for RecordingRenderer {
    type TextLayout = RecordingTextLayout;

    fn set_viewport(&mut self, width: u32, height: u32, scale: f32) {
        self.viewport = (width, height, scale);
    }

    fn upload_text(&mut self, id: TextLayoutId, layout: RecordingTextLayout) {
        self.texts.insert(id, layout);
    }

    fn release_text(&mut self, id: TextLayoutId) {
        self.texts.remove(&id);
    }

    fn upload_image(&mut self, id: ImageId, width: u32, height: u32, rgba: &[u8]) -> Result<(), GraphicsError> {
        let expected = width as usize * height as usize * 4;
        if rgba.len() != expected {
            return Err(GraphicsError::Image(format!(
                "expected {expected} bytes for {width}x{height}, got {}",
                rgba.len()
            )));
        }
        self.images.insert(id, (width, height));
        Ok(())
    }

    fn release_image(&mut self, id: ImageId) {
        self.images.remove(&id);
    }

    fn record_boundary(&mut self, id: BoundaryId, list: &DrawList) {
        self.boundaries.insert(id, list.clone());
    }

    fn release_boundary(&mut self, id: BoundaryId) {
        self.boundaries.remove(&id);
    }

    fn render(&mut self, composition: &Composition) -> Result<(), GraphicsError> {
        self.last_frame = Some(RecordedFrame {
            text: self.flatten(composition),
        });
        self.frames_rendered += 1;
        Ok(())
    }
}

impl GraphicsBackend for RecordingBackend {
    type TextLayout = RecordingTextLayout;
    type TextSystem = RecordingTextSystem;
    type Renderer = RecordingRenderer;

    fn create_text_system() -> Result<RecordingTextSystem, GraphicsError> {
        Ok(RecordingTextSystem)
    }
}

impl<S> CreateRenderer<S> for RecordingBackend {
    fn create_renderer(_surface: &S) -> Result<RecordingRenderer, GraphicsError> {
        Ok(RecordingRenderer::default())
    }
}

#[cfg(test)]
mod tests {
    use geometry::{CornerRadii, Point, Rect, rgb};

    use super::*;
    use crate::LayerProps;

    #[test]
    fn text_wraps_at_word_boundaries() {
        let mut text = RecordingTextSystem;
        let style = TextStyle {
            font_size: 10.0,
            ..TextStyle::default()
        };
        let layout = text.layout("hello brave new world", &style, Some(40.0));
        assert_eq!(layout.lines(), ["hello", "brave", "new", "world"]);
        assert_eq!(layout.size(), Size::new(30.0, 48.0));
        assert_eq!(layout.max_intrinsic_width(), 126.0);
    }

    #[test]
    fn frames_flatten_boundaries_and_resolve_text() {
        let mut renderer = RecordingRenderer::default();
        let mut text = RecordingTextSystem;
        renderer.upload_text(TextLayoutId(1), text.layout("Hi", &TextStyle::default(), None));
        let mut list = DrawList::new();
        list.fill(Rect::from_xywh(0.0, 0.0, 20.0, 10.0), CornerRadii::uniform(4.0), rgb(0x111111));
        list.text(TextLayoutId(1), Point::new(2.0, 1.0));
        renderer.record_boundary(BoundaryId(7), &list);
        let composition = Composition {
            clear_color: geometry::WHITE,
            ops: vec![
                CompositeOp::PushLayer(LayerProps {
                    opacity: 0.5,
                    ..LayerProps::default()
                }),
                CompositeOp::Draw(BoundaryId(7)),
                CompositeOp::PopLayer,
            ],
        };
        assert!(renderer.render(&composition).is_ok());
        assert_eq!(
            renderer.last_frame().map(|frame| frame.text.as_str()),
            Some("clear #ffffffff\nlayer opacity0.5\n  fill 0,0 20x10 r4 #111111ff\n  text \"Hi\" at 2,1\n")
        );
    }
}
