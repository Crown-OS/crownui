use sctk::{
    reexports::{
        client::{
            Proxy,
            protocol::{wl_shm::Format, wl_surface::WlSurface},
        },
        protocols::wp::viewporter::client::wp_viewport::WpViewport,
    },
    shm::slot::{Buffer, SlotPool},
};

use crate::{DragIcon, wayland::event_loop::WaylandPlatform};

const BYTES_PER_PIXEL: u32 = 4;
/// `wl_surface.offset` replaced attach offsets in this version.
const OFFSET_SINCE: u32 = 5;

/// The surface the compositor moves with the pointer during a drag: one shared-memory buffer,
/// shown at the icon's logical size and held by its hotspot.
pub(crate) struct DragIconSurface {
    surface: WlSurface,
    viewport: Option<WpViewport>,
    buffer: Buffer,
    _pool: SlotPool,
    logical_size: (i32, i32),
    pixel_size: (i32, i32),
    hotspot: (i32, i32),
}

impl DragIconSurface {
    pub(crate) fn new(platform: &WaylandPlatform, icon: &DragIcon) -> Option<Self> {
        let (width, height) = (
            i32::try_from(icon.width).ok()?,
            i32::try_from(icon.height).ok()?,
        );
        let stride = width.checked_mul(BYTES_PER_PIXEL as i32)?;
        let length = usize::try_from(stride.checked_mul(height)?).ok()?;
        if width == 0 || height == 0 || icon.rgba.len() != length {
            return None;
        }
        let mut pool = SlotPool::new(length, &platform.shm).ok()?;
        let (buffer, canvas) = pool
            .create_buffer(width, height, stride, Format::Argb8888)
            .ok()?;
        write_premultiplied_bgra(&icon.rgba, canvas);
        let qh = &platform.queue_handle;
        let surface = platform.compositor_state.create_surface(qh);
        let viewport = platform
            .scaling
            .as_ref()
            .map(|scaling| scaling.viewport(&surface, qh));
        Some(Self {
            surface,
            viewport,
            buffer,
            _pool: pool,
            logical_size: (
                icon.logical_width.max(1) as i32,
                icon.logical_height.max(1) as i32,
            ),
            pixel_size: (width, height),
            hotspot: (icon.hotspot.x.round() as i32, icon.hotspot.y.round() as i32),
        })
    }

    pub(crate) fn surface(&self) -> &WlSurface {
        &self.surface
    }

    /// Attaches the pixels; call it after `start_drag` gave the surface its role.
    pub(crate) fn present(&self) {
        let (offset_x, offset_y) = (-self.hotspot.0, -self.hotspot.1);
        let buffer = Some(self.buffer.wl_buffer());
        if self.surface.version() >= OFFSET_SINCE {
            self.surface.attach(buffer, 0, 0);
            self.surface.offset(offset_x, offset_y);
        } else {
            self.surface.attach(buffer, offset_x, offset_y);
        }
        match &self.viewport {
            Some(viewport) => viewport.set_destination(self.logical_size.0, self.logical_size.1),
            None => self.surface.set_buffer_scale(self.integer_scale()),
        }
        self.surface
            .damage_buffer(0, 0, self.pixel_size.0, self.pixel_size.1);
        self.surface.commit();
    }

    /// The buffer scale that maps the pixels onto the logical size, or 1 when none does exactly.
    fn integer_scale(&self) -> i32 {
        let scale = (self.pixel_size.0 / self.logical_size.0).max(1);
        let exact = self.pixel_size.0 % scale == 0 && self.pixel_size.1 % scale == 0;
        if exact { scale } else { 1 }
    }
}

impl Drop for DragIconSurface {
    fn drop(&mut self) {
        if let Some(viewport) = &self.viewport {
            viewport.destroy();
        }
        self.surface.destroy();
    }
}

fn write_premultiplied_bgra(rgba: &[u8], canvas: &mut [u8]) {
    for (source, target) in rgba.chunks_exact(4).zip(canvas.chunks_exact_mut(4)) {
        let alpha = u16::from(source[3]);
        let premultiply = |channel: u8| ((u16::from(channel) * alpha + 127) / 255) as u8;
        target.copy_from_slice(&[
            premultiply(source[2]),
            premultiply(source[1]),
            premultiply(source[0]),
            source[3],
        ]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pixels_become_premultiplied_little_endian_argb() {
        let mut canvas = [0; 8];
        write_premultiplied_bgra(&[255, 0, 0, 255, 0, 255, 0, 128], &mut canvas);
        assert_eq!(canvas, [0, 0, 255, 255, 0, 128, 0, 128]);
    }
}
