use std::time::Duration;

use geometry::{CornerRadii, Rect};

/// A rounded rectangle the platform material should cover, in surface-local logical pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MaterialShape {
    /// Area covered.
    pub rect: Rect,
    /// Corner rounding.
    pub radii: CornerRadii,
}

/// Per-frame surface state the render thread commits together with the frame's buffer,
/// so shapes and regions never lag the pixels they describe.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SurfaceState {
    /// Areas that changed since the previous frame; empty means the whole surface.
    pub damage: Vec<Rect>,
    /// Replaces the platform material; `Material::default()` turns every effect off.
    pub material: Option<crate::Material>,
    /// Shapes the platform material (blur) covers; `None` keeps the previous shapes.
    pub material_shapes: Option<Vec<MaterialShape>>,
    /// Areas that accept input; `None` keeps the previous region.
    pub input_region: Option<Vec<Rect>>,
    /// Areas drawn fully opaque, letting the compositor skip what is behind them.
    pub opaque_region: Option<Vec<Rect>>,
    /// Replaces a layer surface's keyboard focus behavior; `None` keeps the current one.
    pub keyboard: Option<crate::KeyboardInteractivity>,
}

/// When a frame callback fired, as reported by the platform.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FrameTiming {
    /// Platform timestamp of the callback, if it reports one.
    pub callback_time: Option<Duration>,
    /// When the latest presented frame turned into light, on the platform's presentation
    /// clock (`CLOCK_MONOTONIC` on CrownOS).
    pub presented_at: Option<Duration>,
    /// The display's refresh interval at that presentation, when it has a fixed one.
    pub refresh: Option<Duration>,
}

/// Why a surface operation failed.
#[derive(Debug, thiserror::Error)]
pub enum SurfaceError {
    /// The connection to the platform broke.
    #[error("surface connection lost: {0}")]
    Disconnected(String),
}

/// A window surface handed to the render thread.
pub trait PlatformSurface: Send + 'static {
    /// Call right before presenting so [`wait_for_frame`](Self::wait_for_frame) returns when the
    /// platform wants the next frame.
    fn pre_present_notify(&mut self);

    /// Blocks until the requested frame callback fires; returns at once when none is pending.
    fn wait_for_frame(&mut self) -> Result<FrameTiming, SurfaceError>;

    /// Queues `state` to be committed with the next presented frame.
    fn apply(&mut self, state: &SurfaceState);
}
