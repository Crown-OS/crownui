#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

mod animatable;
mod animation;
mod curve;
mod spring;
mod timing;

pub use animatable::Animatable;
pub use animation::{Animation, Sample};
pub use curve::{Curve, Transaction};
pub use spring::Spring;
pub use timing::{CubicBezier, TimingCurve};
