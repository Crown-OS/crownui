/// A straight-alpha sRGB color with `f32` components in `0..=1`.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Color {
    /// Red.
    pub r: f32,
    /// Green.
    pub g: f32,
    /// Blue.
    pub b: f32,
    /// Alpha; `0` is fully transparent.
    pub a: f32,
}

/// Fully transparent black.
pub const TRANSPARENT: Color = Color::new(0.0, 0.0, 0.0, 0.0);
/// Opaque black.
pub const BLACK: Color = Color::new(0.0, 0.0, 0.0, 1.0);
/// Opaque white.
pub const WHITE: Color = Color::new(1.0, 1.0, 1.0, 1.0);

/// An opaque color from `0xRRGGBB`, as written in CSS.
pub const fn rgb(hex: u32) -> Color {
    Color::from_rgba8((hex >> 16) as u8, (hex >> 8) as u8, hex as u8, u8::MAX)
}

/// A color from `0xRRGGBBAA`, as written in CSS.
pub const fn rgba(hex: u32) -> Color {
    Color::from_rgba8(
        (hex >> 24) as u8,
        (hex >> 16) as u8,
        (hex >> 8) as u8,
        hex as u8,
    )
}

impl Color {
    /// Creates a color from `0..=1` components.
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Creates a color from 8-bit components.
    pub const fn from_rgba8(r: u8, g: u8, b: u8, a: u8) -> Self {
        const MAX: f32 = u8::MAX as f32;
        Self::new(
            r as f32 / MAX,
            g as f32 / MAX,
            b as f32 / MAX,
            a as f32 / MAX,
        )
    }

    /// The same color with alpha multiplied by `factor`.
    pub fn with_alpha_factor(self, factor: f32) -> Self {
        Self {
            a: self.a * factor,
            ..self
        }
    }

    /// 8-bit components, rounded.
    pub fn to_rgba8(self) -> [u8; 4] {
        let quantize = |channel: f32| (channel.clamp(0.0, 1.0) * 255.0).round() as u8;
        [
            quantize(self.r),
            quantize(self.g),
            quantize(self.b),
            quantize(self.a),
        ]
    }

    /// Linear interpolation towards `target`; `progress` 0 is `self`, 1 is `target`.
    pub fn lerp(self, target: Self, progress: f32) -> Self {
        let mix = |from: f32, to: f32| from + (to - from) * progress;
        Self::new(
            mix(self.r, target.r),
            mix(self.g, target.g),
            mix(self.b, target.b),
            mix(self.a, target.a),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_literals_round_trip_through_bytes() {
        assert_eq!(rgb(0x3b82f6).to_rgba8(), [0x3b, 0x82, 0xf6, 0xff]);
        assert_eq!(rgba(0x11223344).to_rgba8(), [0x11, 0x22, 0x33, 0x44]);
    }
}
