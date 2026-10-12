#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

mod color;
mod corner_radii;
mod corner_shape;
mod format;
mod insets;
mod point;
mod rect;
mod size;
mod transform;

pub use color::{BLACK, Color, TRANSPARENT, WHITE, rgb, rgba};
pub use corner_radii::CornerRadii;
pub use corner_shape::CornerShape;
pub use format::Length;
pub use insets::Insets;
pub use point::{Point, Vector};
pub use rect::Rect;
pub use size::Size;
pub use transform::Transform;
