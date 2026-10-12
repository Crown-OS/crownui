use crate::util::pixels::straight_top_down;

/// A straight-alpha RGBA8 image, top row first.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pixels {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// `width * height` pixels, four bytes each.
    pub rgba: Vec<u8>,
}

impl Pixels {
    pub(crate) fn from_gl_readback(
        width: u32,
        height: u32,
        bottom_up_premultiplied: &[u8],
    ) -> Self {
        Self {
            width,
            height,
            rgba: straight_top_down(width, bottom_up_premultiplied),
        }
    }

    /// The pixel at column `x`, row `y` (from the top), if inside the image.
    pub fn pixel(&self, x: u32, y: u32) -> Option<[u8; 4]> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let start = (y as usize * self.width as usize + x as usize) * 4;
        self.rgba.get(start..start + 4)?.try_into().ok()
    }
}
