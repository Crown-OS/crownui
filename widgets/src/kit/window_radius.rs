use reactive::Signal;
use view::Cx;

/// The corner radius the compositor rounds this window with, in logical pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WindowRadius(pub f32);

impl WindowRadius {
    /// The radius of a box inset `inset` from the window's edges that keeps its corners
    /// concentric with the window's.
    pub fn concentric(self, inset: f32) -> f32 {
        (self.0 - inset).max(0.0)
    }
}

/// Provides the window's corner radius to the current scope; kit layouts inset from the window
/// edge round concentrically with it, and setting the returned signal re-rounds them.
pub fn provide_window_radius(cx: &mut Cx, radius: WindowRadius) -> Signal<WindowRadius> {
    cx.provide(radius)
}
