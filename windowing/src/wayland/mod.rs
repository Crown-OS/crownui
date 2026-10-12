//! CrownOS-first Wayland backend on smithay-client-toolkit. [`EventLoop`] drives an
//! [`ApplicationHandler`](crate::ApplicationHandler) with a [`WaylandPlatform`], which
//! implements [`Platform`](crate::Platform) and also exposes the CrownOS-only protocols.

mod clipboard;
mod color_management;
mod error;
mod event_loop;
mod globals;
mod output;
mod platform;
pub mod protocols;
mod seat;
mod text_input;
mod util;
mod window;

pub use error::Error;
pub use event_loop::{EventLoop, EventLoopProxy, WaylandPlatform};
pub use output::{Output, OutputEvent, OutputId};
pub use platform::WaylandEvent;
pub use protocols::CrownEvent;
pub use window::{RenderSurface, SurfaceHandle, Window};
