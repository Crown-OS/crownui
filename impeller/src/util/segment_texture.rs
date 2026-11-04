use std::borrow::Cow;

use geometry::{Point, Rect};
use graphics::Path;
use impellers::{Context, Texture};

use super::flatten::{Flattening, Segment, flatten};

/// Texels `flattened_path.frag` reads per segment: its start, then its end.
const TEXELS_PER_SEGMENT: usize = 2;
/// Segments one texture row holds within 4096 texels, the widest texture Vulkan guarantees.
const MAX_SEGMENTS: usize = 4096 / TEXELS_PER_SEGMENT;
const BYTES_PER_TEXEL: usize = 4;
/// Coordinates are 15-bit fractions of the bounds, shifted left by one to leave room for the cap
/// flag in the low bit of x.
const FIXED_POINT_MAX: f32 = 32767.0;

/// A flattened path in the one-row RGBA8 texture `flattened_path.frag` reads.
#[derive(Debug)]
pub(crate) struct SegmentTexture {
    pub(crate) texture: Texture,
    pub(crate) segment_count: usize,
    /// The box the texture's coordinates are fractions of: the path's bounds.
    pub(crate) bounds: Rect,
}

impl SegmentTexture {
    /// Flattens and uploads `path` with `context` current; `None` when the path has no segments
    /// or more than one row holds, or the upload fails.
    pub(crate) fn upload(context: &Context, path: &Path, flattening: Flattening) -> Option<Self> {
        let segments = flatten(path, flattening);
        if segments.is_empty() || segments.len() > MAX_SEGMENTS {
            return None;
        }
        let bounds = path.bounds();
        let width = u32::try_from(segments.len() * TEXELS_PER_SEGMENT).ok()?;
        let texels = Cow::Owned(encode_segments(&segments, bounds));
        // SAFETY: `ImpellerRenderer` drops its resources before the context.
        let texture = unsafe { context.create_texture_with_rgba8(texels, width, 1) }
            .inspect_err(|error| tracing::warn!(%error, "uploading path segments failed"))
            .ok()?;
        Some(Self {
            texture,
            segment_count: segments.len(),
            bounds,
        })
    }
}

/// The RGBA8 row `flattened_path.frag` decodes: each point as big-endian x and y words, x
/// carrying the cap flag of the segment end in its low bit.
fn encode_segments(segments: &[Segment], bounds: Rect) -> Vec<u8> {
    let mut texels = Vec::with_capacity(segments.len() * TEXELS_PER_SEGMENT * BYTES_PER_TEXEL);
    for segment in segments {
        texels.extend(texel(segment.from, segment.start_cap, bounds));
        texels.extend(texel(segment.to, segment.end_cap, bounds));
    }
    texels
}

fn texel(point: Point, cap: bool, bounds: Rect) -> [u8; BYTES_PER_TEXEL] {
    let x = (fixed_point(point.x, bounds.origin.x, bounds.size.width) << 1) | u16::from(cap);
    let y = fixed_point(point.y, bounds.origin.y, bounds.size.height) << 1;
    let ([x_high, x_low], [y_high, y_low]) = (x.to_be_bytes(), y.to_be_bytes());
    [x_high, x_low, y_high, y_low]
}

fn fixed_point(value: f32, origin: f32, span: f32) -> u16 {
    let fraction = if span > 0.0 {
        (value - origin) / span
    } else {
        0.0
    };
    (fraction.clamp(0.0, 1.0) * FIXED_POINT_MAX).round() as u16
}

#[cfg(test)]
mod tests {
    use super::*;

    fn decode(texel: &[u8], bounds: Rect) -> (Point, bool) {
        let x = u16::from_be_bytes([texel[0], texel[1]]);
        let y = u16::from_be_bytes([texel[2], texel[3]]);
        let coordinate = |word: u16, origin: f32, span: f32| {
            origin + f32::from(word >> 1) / FIXED_POINT_MAX * span
        };
        (
            Point::new(
                coordinate(x, bounds.origin.x, bounds.size.width),
                coordinate(y, bounds.origin.y, bounds.size.height),
            ),
            x & 1 == 1,
        )
    }

    #[test]
    fn points_round_trip_within_a_fixed_point_step() {
        let bounds = Rect::from_xywh(-3.0, 10.0, 24.0, 16.0);
        let segment = Segment {
            from: Point::new(-3.0, 26.0),
            to: Point::new(13.37, 17.25),
            start_cap: true,
            end_cap: false,
        };
        let texels = encode_segments(&[segment], bounds);
        assert_eq!(texels.len(), TEXELS_PER_SEGMENT * BYTES_PER_TEXEL);
        let (from, start_cap) = decode(&texels[..4], bounds);
        let (to, end_cap) = decode(&texels[4..], bounds);
        assert_eq!((from, start_cap, end_cap), (segment.from, true, false));
        let step = bounds.size.width / FIXED_POINT_MAX;
        assert!((to.x - segment.to.x).abs() <= step);
        assert!((to.y - segment.to.y).abs() <= step);
    }

    #[test]
    fn flat_bounds_encode_their_origin() {
        let bounds = Rect::from_xywh(2.0, 5.0, 8.0, 0.0);
        let segment = Segment {
            from: Point::new(2.0, 5.0),
            to: Point::new(10.0, 5.0),
            start_cap: false,
            end_cap: true,
        };
        let texels = encode_segments(&[segment], bounds);
        assert_eq!(decode(&texels[4..], bounds), (segment.to, true));
        assert_eq!(texels[..4], [0, 0, 0, 0]);
    }
}
