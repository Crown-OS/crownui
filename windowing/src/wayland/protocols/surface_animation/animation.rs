use std::{f32::consts::TAU, time::Duration};

use crownos_protocols::surface_animation::v1::client::crownos_animated_surface_v1::{
    CrownosAnimatedSurfaceV1, Property,
};
use motion::Spring;
use sctk::reexports::client::{Proxy, backend::ObjectId};

use crate::wayland::util::u64_halves::split_u64;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SurfaceAnimationId(pub(super) ObjectId);

/// Compositor-driven springs on one subsurface's translation, scale, rotation and
/// opacity. Requests take effect on the subsurface's next commit (and its parent's,
/// while the subsurface is synchronized); dropping the handle returns every property
/// to rest on that commit.
#[derive(Debug)]
pub struct SurfaceAnimation {
    pub(super) proxy: CrownosAnimatedSurfaceV1,
}

impl SurfaceAnimation {
    pub fn id(&self) -> SurfaceAnimationId {
        SurfaceAnimationId(self.proxy.id())
    }

    /// Springs `property` toward `target` from `start`, a CLOCK_MONOTONIC instant
    /// (see [`monotonic_now`](super::monotonic_now)); [`Duration::ZERO`] starts it
    /// when the compositor applies the commit. Sampling the same
    /// [`motion::Animation`] from `start` predicts every frame the compositor draws.
    pub fn animate(&self, property: Property, target: f32, spring: Spring, start: Duration) {
        let micros = u64::try_from(start.as_micros()).unwrap_or(u64::MAX);
        let (start_high, start_low) = split_u64(micros);
        self.proxy.animate(
            property,
            f64::from(target),
            f64::from(response_seconds(spring)),
            f64::from(spring.damping_ratio()),
            start_high,
            start_low,
        );
    }

    /// Jumps `property` to `value`, cancelling its animation.
    pub fn set(&self, property: Property, value: f32) {
        self.proxy.set(property, f64::from(value));
    }
}

impl Drop for SurfaceAnimation {
    fn drop(&mut self) {
        self.proxy.destroy();
    }
}

fn response_seconds(spring: Spring) -> f32 {
    TAU / spring.stiffness().sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_response_survives_the_round_trip_through_stiffness() {
        for response in [0.15, 0.25, 0.5, 1.0] {
            let spring = Spring::new(response, 0.8);
            assert!((response_seconds(spring) - response).abs() < 1e-4);
        }
    }
}
