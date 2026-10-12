use std::fmt;

use geometry::{Color, CornerRadii, Length, Rect, Transform};

/// Identifies a repaint boundary whose recorded list the renderer caches.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BoundaryId(pub u64);

/// How a layer is composited; changing these never re-records any list.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LayerProps {
    /// Placement relative to the parent layer.
    pub transform: Transform,
    /// Opacity multiplied into everything inside.
    pub opacity: f32,
    /// Clip in the layer's own coordinates.
    pub clip: Option<(Rect, CornerRadii)>,
    /// Blur of what is behind the clip before drawing the layer (CSS `backdrop-filter`).
    pub backdrop_blur: Option<f32>,
}

impl Default for LayerProps {
    fn default() -> Self {
        Self {
            transform: Transform::IDENTITY,
            opacity: 1.0,
            clip: None,
            backdrop_blur: None,
        }
    }
}

/// One step of a frame's composition.
#[derive(Debug, Clone, PartialEq)]
pub enum CompositeOp {
    /// Enters a layer.
    PushLayer(LayerProps),
    /// Leaves the innermost layer.
    PopLayer,
    /// Draws a cached boundary in the current layer.
    Draw(BoundaryId),
}

/// Everything needed to put one frame on screen, given the cached boundaries.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Composition {
    /// Color the frame is cleared to.
    pub clear_color: Color,
    /// Steps in painting order.
    pub ops: Vec<CompositeOp>,
}

impl fmt::Display for Composition {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let [red, green, blue, alpha] = self.clear_color.to_rgba8();
        writeln!(formatter, "clear #{red:02x}{green:02x}{blue:02x}{alpha:02x}")?;
        let mut depth = 0_usize;
        for op in &self.ops {
            match op {
                CompositeOp::PushLayer(props) => {
                    write!(formatter, "{:indent$}layer", "", indent = depth * 2)?;
                    if !props.transform.is_identity() {
                        let [a, b, c, d, x, y] = props.transform.matrix;
                        if (a, b, c, d) == (1.0, 0.0, 0.0, 1.0) {
                            write!(formatter, " at {},{}", Length(x), Length(y))?;
                        } else {
                            write!(formatter, " transform {:?}", props.transform.matrix)?;
                        }
                    }
                    if props.opacity < 1.0 {
                        write!(formatter, " opacity{}", Length(props.opacity))?;
                    }
                    if let Some((rect, _)) = props.clip {
                        write!(
                            formatter,
                            " clip {},{} {}x{}",
                            Length(rect.min_x()),
                            Length(rect.min_y()),
                            Length(rect.size.width),
                            Length(rect.size.height)
                        )?;
                    }
                    if let Some(radius) = props.backdrop_blur {
                        write!(formatter, " backdrop-blur{}", Length(radius))?;
                    }
                    writeln!(formatter)?;
                    depth += 1;
                }
                CompositeOp::PopLayer => depth = depth.saturating_sub(1),
                CompositeOp::Draw(boundary) => {
                    writeln!(formatter, "{:indent$}draw #{}", "", boundary.0, indent = depth * 2)?;
                }
            }
        }
        Ok(())
    }
}
