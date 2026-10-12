#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

mod gesture;
mod key;
mod keybind;
mod keyboard;
mod modifiers;
mod pointer;
mod text_input;
mod wheel;

pub use gesture::{GesturePhase, TouchpadGesture};
pub use key::{Key, NamedKey, PhysicalKey};
pub use keybind::{Keybind, KeybindParseError};
pub use keyboard::KeyboardEvent;
pub use modifiers::Modifiers;
pub use pointer::{PointerButton, PointerEvent, PointerId, PointerType};
pub use text_input::TextInputEvent;
pub use wheel::{WheelDelta, WheelEvent};
