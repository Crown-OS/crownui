use std::fmt;

use geometry::{Color, CornerRadii, Length, Point, Rect, Transform, Vector};

use crate::{Brush, Paint, Path, Stroke, TextLayoutId};

/// Identifies an image uploaded to the renderer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ImageId(pub u64);

/// One drawing operation, in the coordinates of the enclosing repaint boundary.
#[derive(Debug, Clone, PartialEq)]
pub enum DrawCommand {
    /// Saves transform and clip.
    Save,
    /// Restores the last saved transform and clip (and composites a layer).
    Restore,
    /// Multiplies the current transform.
    Transform(Transform),
    /// Intersects the clip with a rectangle.
    ClipRect(Rect),
    /// Intersects the clip with a rounded rectangle.
    ClipRoundedRect(Rect, CornerRadii),
    /// Fills a rectangle.
    FillRect(Rect, Paint),
    /// Fills a rounded rectangle.
    FillRoundedRect(Rect, CornerRadii, Paint),
    /// Outlines a rounded rectangle, centered on its edge.
    StrokeRoundedRect(Rect, CornerRadii, Stroke, Paint),
    /// Fills a path.
    FillPath(Path, Paint),
    /// Outlines a path.
    StrokePath(Path, Stroke, Paint),
    /// A blurred drop shadow of a rounded rectangle (CSS `box-shadow`).
    Shadow {
        /// Casting shape.
        rect: Rect,
        /// Casting shape's corners.
        radii: CornerRadii,
        /// Shadow color.
        color: Color,
        /// Blur radius.
        blur: f32,
        /// Offset from the casting shape.
        offset: Vector,
    },
    /// Draws a shaped text layout with its top-left corner at `origin`.
    Text(TextLayoutId, Point),
    /// Draws an image scaled into `dest`.
    Image(ImageId, Rect),
    /// Starts a layer composited at `opacity`, optionally blurring what is behind it first.
    /// Ends at the matching [`DrawCommand::Restore`].
    SaveLayer {
        /// Area the layer covers; `None` is unbounded.
        bounds: Option<Rect>,
        /// Layer opacity.
        opacity: f32,
        /// Backdrop blur radius (CSS `backdrop-filter: blur`).
        backdrop_blur: Option<f32>,
    },
}

/// A recorded list of draw commands.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct DrawList {
    commands: Vec<DrawCommand>,
}

impl DrawList {
    /// An empty list.
    pub fn new() -> Self {
        Self::default()
    }

    /// The recorded commands.
    pub fn commands(&self) -> &[DrawCommand] {
        &self.commands
    }

    /// Whether nothing was recorded.
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    /// Removes every command, keeping the allocation.
    pub fn clear(&mut self) {
        self.commands.clear();
    }

    /// Appends a command.
    pub fn push(&mut self, command: DrawCommand) {
        self.commands.push(command);
    }

    /// Appends every command of `other`.
    pub fn extend_from(&mut self, other: &DrawList) {
        self.commands.extend_from_slice(&other.commands);
    }

    /// Saves transform and clip.
    pub fn save(&mut self) {
        self.push(DrawCommand::Save);
    }

    /// Restores transform and clip.
    pub fn restore(&mut self) {
        self.push(DrawCommand::Restore);
    }

    /// Moves the origin.
    pub fn translate(&mut self, offset: Vector) {
        if offset != Vector::ZERO {
            self.push(DrawCommand::Transform(Transform::translate(offset)));
        }
    }

    /// Multiplies the current transform.
    pub fn transform(&mut self, transform: Transform) {
        if !transform.is_identity() {
            self.push(DrawCommand::Transform(transform));
        }
    }

    /// Clips to a rounded rectangle (or a plain one when the radii are zero).
    pub fn clip(&mut self, rect: Rect, radii: CornerRadii) {
        if radii.is_zero() {
            self.push(DrawCommand::ClipRect(rect));
        } else {
            self.push(DrawCommand::ClipRoundedRect(rect, radii));
        }
    }

    /// Fills a rounded rectangle (or a plain one when the radii are zero).
    pub fn fill(&mut self, rect: Rect, radii: CornerRadii, paint: impl Into<Paint>) {
        let paint = paint.into();
        if radii.is_zero() {
            self.push(DrawCommand::FillRect(rect, paint));
        } else {
            self.push(DrawCommand::FillRoundedRect(rect, radii, paint));
        }
    }

    /// Outlines a rounded rectangle.
    pub fn stroke(&mut self, rect: Rect, radii: CornerRadii, width: f32, paint: impl Into<Paint>) {
        self.push(DrawCommand::StrokeRoundedRect(
            rect,
            radii,
            Stroke::new(width),
            paint.into(),
        ));
    }

    /// Adds a drop shadow.
    pub fn shadow(
        &mut self,
        rect: Rect,
        radii: CornerRadii,
        color: Color,
        blur: f32,
        offset: Vector,
    ) {
        self.push(DrawCommand::Shadow {
            rect,
            radii,
            color,
            blur,
            offset,
        });
    }

    /// Draws text.
    pub fn text(&mut self, layout: TextLayoutId, origin: Point) {
        self.push(DrawCommand::Text(layout, origin));
    }

    /// Draws `image` scaled into `dest`.
    pub fn image(&mut self, image: ImageId, dest: Rect) {
        self.push(DrawCommand::Image(image, dest));
    }

    /// Text layouts this list references.
    pub fn text_layouts(&self) -> impl Iterator<Item = TextLayoutId> + '_ {
        self.commands.iter().filter_map(|command| match command {
            DrawCommand::Text(layout, _) => Some(*layout),
            _ => None,
        })
    }

    /// The images this list draws, directly or as a fill.
    pub fn images(&self) -> impl Iterator<Item = ImageId> + '_ {
        let filled = |paint: &Paint| match paint.brush {
            Brush::Image { image, .. } => Some(image),
            _ => None,
        };
        self.commands
            .iter()
            .filter_map(move |command| match command {
                DrawCommand::Image(image, _) => Some(*image),
                DrawCommand::FillRect(_, paint)
                | DrawCommand::FillRoundedRect(_, _, paint)
                | DrawCommand::StrokeRoundedRect(_, _, _, paint)
                | DrawCommand::FillPath(_, paint)
                | DrawCommand::StrokePath(_, _, paint) => filled(paint),
                _ => None,
            })
    }
}

fn write_rect(formatter: &mut fmt::Formatter<'_>, rect: &Rect) -> fmt::Result {
    write!(
        formatter,
        "{},{} {}x{}",
        Length(rect.min_x()),
        Length(rect.min_y()),
        Length(rect.size.width),
        Length(rect.size.height)
    )
}

fn write_radii(formatter: &mut fmt::Formatter<'_>, radii: &CornerRadii) -> fmt::Result {
    if radii.top_left == radii.top_right
        && radii.top_left == radii.bottom_right
        && radii.top_left == radii.bottom_left
    {
        write!(formatter, " r{}", Length(radii.top_left))
    } else {
        write!(
            formatter,
            " r{}/{}/{}/{}",
            Length(radii.top_left),
            Length(radii.top_right),
            Length(radii.bottom_right),
            Length(radii.bottom_left)
        )
    }
}

fn write_color(formatter: &mut fmt::Formatter<'_>, color: &Color) -> fmt::Result {
    let [red, green, blue, alpha] = color.to_rgba8();
    write!(formatter, "#{red:02x}{green:02x}{blue:02x}{alpha:02x}")
}

fn write_paint(formatter: &mut fmt::Formatter<'_>, paint: &Paint) -> fmt::Result {
    formatter.write_str(" ")?;
    match &paint.brush {
        Brush::Solid(color) => write_color(formatter, color)?,
        Brush::LinearGradient { stops, .. } => {
            formatter.write_str("gradient(")?;
            for (index, stop) in stops.iter().enumerate() {
                if index > 0 {
                    formatter.write_str(",")?;
                }
                write_color(formatter, &stop.color)?;
                write!(formatter, "@{}", stop.offset)?;
            }
            formatter.write_str(")")?;
        }
        Brush::Image { image, rect } => {
            write!(formatter, "image({})", image.0)?;
            write_rect(formatter, rect)?;
        }
    }
    match paint.blur {
        Some(blur) => write!(formatter, " blur{}", Length(blur)),
        None => Ok(()),
    }
}

impl fmt::Display for DrawCommand {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Save => formatter.write_str("save"),
            Self::Restore => formatter.write_str("restore"),
            Self::Transform(transform) => write!(formatter, "transform {:?}", transform.matrix),
            Self::ClipRect(rect) => {
                formatter.write_str("clip ")?;
                write_rect(formatter, rect)
            }
            Self::ClipRoundedRect(rect, radii) => {
                formatter.write_str("clip ")?;
                write_rect(formatter, rect)?;
                write_radii(formatter, radii)
            }
            Self::FillRect(rect, paint) => {
                formatter.write_str("fill ")?;
                write_rect(formatter, rect)?;
                write_paint(formatter, paint)
            }
            Self::FillRoundedRect(rect, radii, paint) => {
                formatter.write_str("fill ")?;
                write_rect(formatter, rect)?;
                write_radii(formatter, radii)?;
                write_paint(formatter, paint)
            }
            Self::StrokeRoundedRect(rect, radii, stroke, paint) => {
                write!(formatter, "stroke{} ", Length(stroke.width))?;
                write_rect(formatter, rect)?;
                write_radii(formatter, radii)?;
                write_paint(formatter, paint)
            }
            Self::FillPath(path, paint) => {
                write!(formatter, "fill path[{}] ", path.commands.len())?;
                write_rect(formatter, &path.bounds())?;
                write_paint(formatter, paint)
            }
            Self::StrokePath(path, stroke, paint) => {
                write!(
                    formatter,
                    "stroke{} path[{}] ",
                    Length(stroke.width),
                    path.commands.len()
                )?;
                write_rect(formatter, &path.bounds())?;
                write_paint(formatter, paint)
            }
            Self::Shadow {
                rect,
                radii,
                color,
                blur,
                offset,
            } => {
                formatter.write_str("shadow ")?;
                write_rect(formatter, rect)?;
                write_radii(formatter, radii)?;
                formatter.write_str(" ")?;
                write_color(formatter, color)?;
                write!(
                    formatter,
                    " blur{} offset{},{}",
                    Length(*blur),
                    Length(offset.x),
                    Length(offset.y)
                )
            }
            Self::Text(layout, origin) => {
                write!(
                    formatter,
                    "text #{} at {},{}",
                    layout.0,
                    Length(origin.x),
                    Length(origin.y)
                )
            }
            Self::Image(image, rect) => {
                write!(formatter, "image #{} ", image.0)?;
                write_rect(formatter, rect)
            }
            Self::SaveLayer {
                bounds,
                opacity,
                backdrop_blur,
            } => {
                write!(formatter, "layer opacity{}", Length(*opacity))?;
                if let Some(bounds) = bounds {
                    formatter.write_str(" ")?;
                    write_rect(formatter, bounds)?;
                }
                match backdrop_blur {
                    Some(radius) => write!(formatter, " backdrop-blur{}", Length(*radius)),
                    None => Ok(()),
                }
            }
        }
    }
}

impl fmt::Display for DrawList {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut depth = 0_usize;
        for command in &self.commands {
            if matches!(command, DrawCommand::Restore) {
                depth = depth.saturating_sub(1);
            }
            writeln!(formatter, "{:indent$}{command}", "", indent = depth * 2)?;
            if matches!(command, DrawCommand::Save | DrawCommand::SaveLayer { .. }) {
                depth += 1;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use geometry::rgb;

    use super::*;

    #[test]
    fn draw_lists_print_as_indented_text() {
        let mut list = DrawList::new();
        list.save();
        list.clip(
            Rect::from_xywh(0.0, 0.0, 10.0, 10.0),
            CornerRadii::uniform(2.0),
        );
        list.fill(
            Rect::from_xywh(1.0, 2.0, 3.0, 4.0),
            CornerRadii::ZERO,
            rgb(0x3b82f6),
        );
        list.restore();
        assert_eq!(
            list.to_string(),
            "save\n  clip 0,0 10x10 r2\n  fill 1,2 3x4 #3b82f6ff\nrestore\n"
        );
    }

    #[test]
    fn identity_transforms_and_zero_offsets_are_not_recorded() {
        let mut list = DrawList::new();
        list.translate(Vector::ZERO);
        list.transform(Transform::IDENTITY);
        assert!(list.is_empty());
    }
}
