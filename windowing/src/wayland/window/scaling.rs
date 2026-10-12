use sctk::{
    dispatch2::Dispatch2,
    reexports::{
        client::{Connection, QueueHandle, globals::GlobalList, protocol::wl_surface::WlSurface},
        protocols::wp::{
            fractional_scale::v1::client::{
                wp_fractional_scale_manager_v1::WpFractionalScaleManagerV1,
                wp_fractional_scale_v1::{self, WpFractionalScaleV1},
            },
            viewporter::client::{wp_viewport::WpViewport, wp_viewporter::WpViewporter},
        },
    },
};

use crate::wayland::{event_loop::WaylandPlatform, util::no_events::NoEvents};
use crate::{
    WindowId,
    dpi::{LogicalSize, ScaleFactor},
};

/// Fractional scaling needs both globals: the scale arrives through one and the
/// oversized buffer is mapped back onto the logical size through the other.
pub(crate) struct ScalingGlobals {
    fractional_scale_manager: WpFractionalScaleManagerV1,
    viewporter: WpViewporter,
}

impl ScalingGlobals {
    pub(crate) fn bind(globals: &GlobalList, qh: &QueueHandle<WaylandPlatform>) -> Option<Self> {
        Some(Self {
            fractional_scale_manager: globals.bind(qh, 1..=1, NoEvents).ok()?,
            viewporter: globals.bind(qh, 1..=1, NoEvents).ok()?,
        })
    }

    /// A viewport that maps `surface`'s buffer onto a logical size of its own.
    pub(crate) fn viewport(
        &self,
        surface: &WlSurface,
        qh: &QueueHandle<WaylandPlatform>,
    ) -> WpViewport {
        self.viewporter.get_viewport(surface, qh, NoEvents)
    }
}

pub(crate) enum SurfaceScaling {
    Integer,
    Fractional {
        fractional_scale: WpFractionalScaleV1,
        viewport: WpViewport,
    },
}

impl SurfaceScaling {
    pub(crate) fn new(
        globals: Option<&ScalingGlobals>,
        surface: &WlSurface,
        window_id: WindowId,
        qh: &QueueHandle<WaylandPlatform>,
    ) -> Self {
        let Some(globals) = globals else {
            return Self::Integer;
        };
        Self::Fractional {
            fractional_scale: globals.fractional_scale_manager.get_fractional_scale(
                surface,
                qh,
                FractionalScaleData(window_id),
            ),
            viewport: globals.viewporter.get_viewport(surface, qh, NoEvents),
        }
    }

    pub(crate) fn is_fractional(&self) -> bool {
        matches!(self, Self::Fractional { .. })
    }

    pub(crate) fn apply(&self, surface: &WlSurface, size: LogicalSize, scale: ScaleFactor) {
        match self {
            Self::Integer => {
                surface.set_buffer_scale((scale.as_120ths() / ScaleFactor::DENOMINATOR) as i32)
            }
            Self::Fractional { viewport, .. } => {
                viewport.set_destination(size.width as i32, size.height as i32)
            }
        }
    }
}

impl Drop for SurfaceScaling {
    fn drop(&mut self) {
        if let Self::Fractional {
            fractional_scale,
            viewport,
        } = self
        {
            fractional_scale.destroy();
            viewport.destroy();
        }
    }
}

pub(crate) struct FractionalScaleData(WindowId);

impl Dispatch2<WpFractionalScaleV1, WaylandPlatform> for FractionalScaleData {
    fn event(
        &self,
        state: &mut WaylandPlatform,
        _: &WpFractionalScaleV1,
        event: wp_fractional_scale_v1::Event,
        _: &Connection,
        _: &QueueHandle<WaylandPlatform>,
    ) {
        if let wp_fractional_scale_v1::Event::PreferredScale { scale } = event {
            state.apply_window_scale(self.0, ScaleFactor::from_120ths(scale));
        }
    }
}
