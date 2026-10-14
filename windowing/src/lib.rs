#![doc = include_str!("../README.md")]

mod app_icon;
mod clock;
pub mod dpi;
mod drag;
mod event;
mod layer;
mod material;
mod platform;
mod surface;
mod text_input;
mod visibility;
mod window;

#[cfg(feature = "headless")]
pub mod headless;
#[cfg(feature = "wayland")]
pub mod wayland;

pub use app_icon::AppIconPlacement;
pub use clock::{Clock, ManualClock, SystemClock};
pub use drag::{DragIcon, DragOffer, DragOutcome, DragSource, DroppedData};
pub use event::WindowEvent;
pub use layer::{Anchor, KeyboardInteractivity, Layer, LayerExtent, LayerMargin, LayerOptions};
pub use material::{Material, MaterialBlur, MaterialShadow};
pub use platform::{ApplicationHandler, Capabilities, ControlFlow, Platform, Waker, WindowControl};
pub use raw_window_handle;
pub use surface::{FrameTiming, MaterialShape, PlatformSurface, SurfaceError, SurfaceState};
pub use text_input::{TextInputPurpose, TextInputState};
pub use visibility::{AgentAccess, Visibility};
pub use window::{CursorIcon, Decorations, WindowAttributes, WindowId};
