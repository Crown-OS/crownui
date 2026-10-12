use crownos_protocols::background_effects::v1::client::{
    crownos_background_effect_surface_v1::CrownosBackgroundEffectSurfaceV1,
    crownos_shape_manager_v1::CrownosShapeManagerV1, crownos_shape_v1::CrownosShapeV1,
};
use sctk::reexports::client::QueueHandle;

use super::material::{BackgroundEffect, Shape, ShapePrimitive};
use crate::wayland::{event_loop::WaylandPlatform, util::no_events::NoEvents};

/// A shape object that lives only as long as the request it is passed to;
/// the compositor copies its primitives when the request arrives.
struct TransientShape(CrownosShapeV1);

impl Drop for TransientShape {
    fn drop(&mut self) {
        self.0.destroy();
    }
}

/// The protocol objects behind a window's material; cloneable so the render thread can
/// update shapes together with its frames.
#[derive(Debug, Clone)]
pub(crate) struct EffectProxies {
    pub(super) effect: CrownosBackgroundEffectSurfaceV1,
    pub(super) shapes: CrownosShapeManagerV1,
    pub(super) queue_handle: QueueHandle<WaylandPlatform>,
}

/// Owns the effect object and destroys it with the window.
pub(crate) struct BackgroundEffectSurface(pub(crate) EffectProxies);

impl BackgroundEffectSurface {
    pub(crate) fn apply(&self, effect: &BackgroundEffect) {
        self.0.apply(effect);
    }

    pub(crate) fn proxies(&self) -> EffectProxies {
        self.0.clone()
    }
}

impl EffectProxies {
    pub(crate) fn apply(&self, effect: &BackgroundEffect) {
        self.effect.set_corner_radius(effect.corner_radius);
        self.effect.set_border(effect.border_width);

        match &effect.blur {
            Some(blur) => {
                let shape = blur.shape.as_ref().map(|shape| self.build(shape));
                self.effect.set_blur(
                    shape.as_ref().map(|shape| &shape.0),
                    blur.radius,
                    blur.tint.0,
                    blur.saturation.max(0.0),
                );
            }
            None => self.effect.set_blur(None, 0, 0, 1.0),
        }

        match &effect.shadow {
            Some(shadow) => {
                let shape = shadow.shape.as_ref().map(|shape| self.build(shape));
                self.effect.set_shadow(
                    shape.as_ref().map(|shape| &shape.0),
                    shadow.radius,
                    shadow.offset_x,
                    shadow.offset_y,
                    shadow.color.0,
                );
            }
            None => self.effect.set_shadow(None, 0, 0, 0, 0),
        }
    }

    fn build(&self, shape: &Shape) -> TransientShape {
        let proxy = self.shapes.create_shape(&self.queue_handle, NoEvents);
        let length = |value: u32| i32::try_from(value).unwrap_or(i32::MAX);
        for primitive in &shape.0 {
            match *primitive {
                ShapePrimitive::RoundedRect {
                    x,
                    y,
                    width,
                    height,
                    radius,
                } => proxy.add_rounded_rect(x, y, length(width), length(height), radius),
                ShapePrimitive::Circle {
                    center_x,
                    center_y,
                    radius,
                } => proxy.add_circle(center_x, center_y, radius),
            }
        }
        TransientShape(proxy)
    }
}

impl Drop for BackgroundEffectSurface {
    fn drop(&mut self) {
        self.0.effect.destroy();
    }
}
