#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

mod backend;
mod composition;
mod draw_list;
mod paint;
mod path;
mod recording;
mod text;

pub use backend::{CreateRenderer, GraphicsBackend, GraphicsError, Renderer};
pub use composition::{BoundaryId, CompositeOp, Composition, LayerProps};
pub use draw_list::{DrawCommand, DrawList, ImageId};
pub use paint::{Brush, GradientStop, Paint, Stroke, StrokeCap, StrokeJoin};
pub use path::{Path, PathCommand};
pub use recording::{
    RecordedFrame, RecordingBackend, RecordingRenderer, RecordingTextLayout, RecordingTextSystem,
};
pub use text::{FontWeight, TextAlign, TextLayout, TextLayoutId, TextStyle, TextSystem};
