use gpui::{Rgba, rgb};

pub const BACKGROUND: u32 = 0xf8fafc;
pub const FOREGROUND: u32 = 0x0f172a;
pub const MUTED: u32 = 0x64748b;
pub const ACCENT: u32 = 0x3b82f6;

const SWATCHES: [u32; 8] = [
    0x3b82f6, 0xef4444, 0x10b981, 0xf59e0b, 0x8b5cf6, 0xec4899, 0x14b8a6, 0xf97316,
];

pub fn swatch(index: usize) -> Rgba {
    rgb(SWATCHES[index % SWATCHES.len()])
}
