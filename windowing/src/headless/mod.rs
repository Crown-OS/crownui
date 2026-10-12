//! In-memory backend for tests and previews: windows never reach a screen, time only moves
//! through [`HeadlessPlatform::advance`] and input arrives through [`HeadlessPlatform::inject`].

mod platform;
mod surface;
mod window;

pub use platform::HeadlessPlatform;
pub use surface::HeadlessSurface;
pub use window::HeadlessWindow;
