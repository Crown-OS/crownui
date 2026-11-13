use std::collections::HashMap;

use geometry::{CornerRadii, Point, Rect, Transform};
use graphics::{
    Brush, DrawCommand, DrawList, ImageId, Paint, Path, PathCommand, Stroke, StrokeCap, StrokeJoin,
    TextAlign, TextLayoutId,
};

use crate::{
    renderer::DrawnText,
    scene::{Fill, Item, Shadow, StrokeSpec, TextItem},
    util::hex_color::hex_color,
};

/// What a boundary's commands refer to.
pub(crate) struct DrawSources<'a> {
    pub(crate) texts: &'a HashMap<TextLayoutId, DrawnText>,
    pub(crate) image_keys: &'a HashMap<ImageId, String>,
}

struct GroupHeader {
    name: &'static str,
    clip: Option<[f32; 4]>,
    radii: [f32; 4],
    opacity: f32,
    backdrop_blur: Option<f32>,
}

struct Sink {
    header: Option<GroupHeader>,
    items: Vec<Item>,
}

struct SavedState {
    transform: Transform,
    depth: usize,
}

/// Converts one boundary's list into scene items in the node's coordinates.
pub(crate) fn draw_items(list: &DrawList, sources: &DrawSources<'_>) -> Vec<Item> {
    let mut converter = Converter {
        sources,
        sinks: vec![Sink {
            header: None,
            items: Vec::new(),
        }],
        saved: Vec::new(),
        transform: Transform::IDENTITY,
    };
    for command in list.commands() {
        converter.apply(command);
    }
    converter.close_to(1);
    converter
        .sinks
        .pop()
        .map(|sink| sink.items)
        .unwrap_or_default()
}

struct Converter<'a, 'b> {
    sources: &'a DrawSources<'b>,
    sinks: Vec<Sink>,
    saved: Vec<SavedState>,
    transform: Transform,
}

impl Converter<'_, '_> {
    fn apply(&mut self, command: &DrawCommand) {
        match command {
            DrawCommand::Save => self.save(),
            DrawCommand::SaveLayer {
                bounds,
                opacity,
                backdrop_blur,
            } => {
                self.save();
                let clip = backdrop_blur.and(*bounds).map(|bounds| self.rect(bounds));
                self.open(GroupHeader {
                    name: "layer",
                    clip,
                    radii: [0.0; 4],
                    opacity: *opacity,
                    backdrop_blur: *backdrop_blur,
                });
            }
            DrawCommand::Restore => {
                if let Some(state) = self.saved.pop() {
                    self.close_to(state.depth);
                    self.transform = state.transform;
                }
            }
            DrawCommand::Transform(transform) => self.transform = transform.then(self.transform),
            DrawCommand::ClipRect(rect) => self.clip(*rect, CornerRadii::ZERO),
            DrawCommand::ClipRoundedRect(rect, radii) => self.clip(*rect, *radii),
            DrawCommand::FillRect(rect, paint) => self.fill_rect(*rect, CornerRadii::ZERO, paint),
            DrawCommand::FillRoundedRect(rect, radii, paint) => {
                self.fill_rect(*rect, *radii, paint)
            }
            DrawCommand::StrokeRoundedRect(rect, radii, stroke, paint) => {
                self.stroke_rect(*rect, *radii, stroke, paint);
            }
            DrawCommand::FillPath(path, paint) => {
                let fill = Some(self.fill(paint));
                self.push(Item::Path {
                    data: self.path_data(path),
                    fill,
                    stroke: None,
                });
            }
            DrawCommand::StrokePath(path, stroke, paint) => {
                let stroke = Some(self.stroke(stroke, paint));
                self.push(Item::Path {
                    data: self.path_data(path),
                    fill: None,
                    stroke,
                });
            }
            DrawCommand::Shadow {
                rect,
                radii,
                color,
                blur,
                offset,
            } => {
                let (x_scale, y_scale) = self.scales();
                self.push(Item::Rect {
                    rect: self.rect(*rect),
                    radii: scaled_radii(*radii, x_scale.min(y_scale)),
                    fill: None,
                    stroke: None,
                    shadows: vec![Shadow {
                        color: hex_color(*color),
                        blur: *blur,
                        offset: [offset.x * x_scale, offset.y * y_scale],
                    }],
                });
            }
            DrawCommand::Text(id, origin) => self.text(*id, *origin),
            DrawCommand::Image(id, dest) => {
                if let Some(image) = self.sources.image_keys.get(id) {
                    self.push(Item::Image {
                        rect: self.rect(*dest),
                        image: image.clone(),
                    });
                }
            }
        }
    }

    fn save(&mut self) {
        self.saved.push(SavedState {
            transform: self.transform,
            depth: self.sinks.len(),
        });
    }

    fn open(&mut self, header: GroupHeader) {
        self.sinks.push(Sink {
            header: Some(header),
            items: Vec::new(),
        });
    }

    fn close_to(&mut self, depth: usize) {
        while self.sinks.len() > depth.max(1) {
            let Some(Sink {
                header: Some(header),
                items,
            }) = self.sinks.pop()
            else {
                continue;
            };
            if items.is_empty() {
                continue;
            }
            self.push(Item::Group {
                name: header.name.to_owned(),
                matrix: Transform::IDENTITY.matrix,
                clip: header.clip,
                radii: header.radii,
                opacity: header.opacity,
                backdrop_blur: header.backdrop_blur,
                items,
            });
        }
    }

    fn push(&mut self, item: Item) {
        if let Some(sink) = self.sinks.last_mut() {
            sink.items.push(item);
        }
    }

    fn last_item(&mut self) -> Option<&mut Item> {
        self.sinks.last_mut().and_then(|sink| sink.items.last_mut())
    }

    fn clip(&mut self, rect: Rect, radii: CornerRadii) {
        let (x_scale, y_scale) = self.scales();
        self.open(GroupHeader {
            name: "clip",
            clip: Some(self.rect(rect)),
            radii: scaled_radii(radii, x_scale.min(y_scale)),
            opacity: 1.0,
            backdrop_blur: None,
        });
    }

    fn fill_rect(&mut self, rect: Rect, radii: CornerRadii, paint: &Paint) {
        let rect = self.rect(rect);
        let (x_scale, y_scale) = self.scales();
        let radii = scaled_radii(radii, x_scale.min(y_scale));
        let fill = self.fill(paint);
        if let Some(Item::Rect {
            rect: shadow_rect,
            radii: shadow_radii,
            fill: shadow_fill @ None,
            stroke: None,
            ..
        }) = self.last_item()
            && same_box(*shadow_rect, rect)
            && *shadow_radii == radii
        {
            *shadow_fill = Some(fill);
            return;
        }
        self.push(Item::Rect {
            rect,
            radii,
            fill: Some(fill),
            stroke: None,
            shadows: Vec::new(),
        });
    }

    fn stroke_rect(&mut self, rect: Rect, radii: CornerRadii, stroke: &Stroke, paint: &Paint) {
        let rect = self.rect(rect);
        let (x_scale, y_scale) = self.scales();
        let radii = scaled_radii(radii, x_scale.min(y_scale));
        let stroke = self.stroke(stroke, paint);
        if let Some(Item::Rect {
            rect: filled_rect,
            radii: filled_radii,
            stroke: previous @ None,
            ..
        }) = self.last_item()
            && same_box(*filled_rect, rect)
            && *filled_radii == radii
        {
            *previous = Some(stroke);
            return;
        }
        self.push(Item::Rect {
            rect,
            radii,
            fill: None,
            stroke: Some(stroke),
            shadows: Vec::new(),
        });
    }

    fn text(&mut self, id: TextLayoutId, origin: Point) {
        let Some(drawn) = self.sources.texts.get(&id) else {
            return;
        };
        let style = &drawn.source.style;
        let [a, b, c, d, ..] = self.transform.matrix;
        let translation_only = a == 1.0 && b == 0.0 && c == 0.0 && d == 1.0;
        let position = if translation_only {
            self.transform.apply(origin)
        } else {
            origin
        };
        self.push(Item::Text(TextItem {
            x: position.x,
            y: position.y,
            width: drawn.width,
            height: drawn.height,
            wrap_width: drawn.source.wrap_width,
            lines: drawn.lines,
            text: drawn.source.text.clone(),
            family: style.font_family.as_ref().map(|family| family.to_string()),
            size: style.font_size,
            weight: style.font_weight.0,
            italic: style.italic,
            color: hex_color(style.color),
            letter_spacing: style.letter_spacing,
            align: align_name(style.align).to_owned(),
            max_lines: style.max_lines.filter(|_| style.ellipsis),
            blur: style.blur,
            matrix: (!translation_only).then_some(self.transform.matrix),
        }));
    }

    fn scales(&self) -> (f32, f32) {
        let [a, b, c, d, ..] = self.transform.matrix;
        (a.hypot(b), c.hypot(d))
    }

    fn rect(&self, rect: Rect) -> [f32; 4] {
        let start = self.transform.apply(rect.origin);
        let end = self.transform.apply(Point::new(rect.max_x(), rect.max_y()));
        [
            start.x.min(end.x),
            start.y.min(end.y),
            (end.x - start.x).abs(),
            (end.y - start.y).abs(),
        ]
    }

    fn point(&self, point: Point) -> [f32; 2] {
        let moved = self.transform.apply(point);
        [moved.x, moved.y]
    }

    fn fill(&self, paint: &Paint) -> Fill {
        match &paint.brush {
            Brush::Solid(color) => Fill::Solid {
                color: hex_color(*color),
                blur: paint.blur,
            },
            Brush::LinearGradient { start, end, stops } => Fill::Linear {
                start: self.point(*start),
                end: self.point(*end),
                stops: stops
                    .iter()
                    .map(|stop| (stop.offset, hex_color(stop.color)))
                    .collect(),
                blur: paint.blur,
            },
            Brush::Image { image, rect } => Fill::Image {
                image: self
                    .sources
                    .image_keys
                    .get(image)
                    .cloned()
                    .unwrap_or_default(),
                rect: self.rect(*rect),
            },
        }
    }

    fn stroke(&self, stroke: &Stroke, paint: &Paint) -> StrokeSpec {
        let (x_scale, y_scale) = self.scales();
        StrokeSpec {
            fill: self.fill(paint),
            width: stroke.width * x_scale.min(y_scale),
            cap: match stroke.cap {
                StrokeCap::Butt => "butt",
                StrokeCap::Round => "round",
                StrokeCap::Square => "square",
            },
            join: match stroke.join {
                StrokeJoin::Miter => "miter",
                StrokeJoin::Round => "round",
                StrokeJoin::Bevel => "bevel",
            },
        }
    }

    fn path_data(&self, path: &Path) -> String {
        let mut data = String::with_capacity(path.commands.len() * 24);
        let point = |data: &mut String, value: Point| {
            let [x, y] = self.point(value);
            data.push_str(&format!(" {} {}", round2(x), round2(y)));
        };
        for command in &path.commands {
            match *command {
                PathCommand::MoveTo(to) => {
                    data.push_str(" M");
                    point(&mut data, to);
                }
                PathCommand::LineTo(to) => {
                    data.push_str(" L");
                    point(&mut data, to);
                }
                PathCommand::QuadTo(control, to) => {
                    data.push_str(" Q");
                    point(&mut data, control);
                    point(&mut data, to);
                }
                PathCommand::CubicTo(first, second, to) => {
                    data.push_str(" C");
                    point(&mut data, first);
                    point(&mut data, second);
                    point(&mut data, to);
                }
                PathCommand::Close => data.push_str(" Z"),
            }
        }
        data.trim_start().to_owned()
    }
}

fn round2(value: f32) -> f32 {
    (value * 100.0).round() / 100.0
}

fn scaled_radii(radii: CornerRadii, scale: f32) -> [f32; 4] {
    [
        radii.top_left * scale,
        radii.top_right * scale,
        radii.bottom_right * scale,
        radii.bottom_left * scale,
    ]
}

pub(crate) fn same_box(first: [f32; 4], second: [f32; 4]) -> bool {
    first
        .iter()
        .zip(second)
        .all(|(left, right)| (left - right).abs() < 0.5)
}

fn align_name(align: TextAlign) -> &'static str {
    match align {
        TextAlign::Start => "start",
        TextAlign::Center => "center",
        TextAlign::End => "end",
        TextAlign::Justify => "justify",
    }
}
