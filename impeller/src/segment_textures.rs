use std::{
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
    mem,
};

use geometry::Point;
use graphics::{Path, PathCommand};
use impellers::Context;

use crate::util::{flatten::Flattening, segment_texture::SegmentTexture};

/// Textures kept before the cache starts over: icons redraw from a few hundred paths at most.
const MAX_CACHED_PATHS: usize = 256;

/// Segment textures of recently drawn paths, keyed by a hash of their commands and flattening.
#[derive(Debug, Default)]
pub(crate) struct SegmentTextures(HashMap<u64, SegmentTexture>);

impl SegmentTextures {
    /// The texture of `path` cut by `flattening`, uploaded on first use with `context` current;
    /// `None` when [`SegmentTexture::upload`] cannot make one.
    pub(crate) fn get_or_upload(
        &mut self,
        context: &Context,
        path: &Path,
        flattening: Flattening,
    ) -> Option<&SegmentTexture> {
        let key = cache_key(path, flattening);
        if !self.0.contains_key(&key) {
            if self.0.len() >= MAX_CACHED_PATHS {
                self.0.clear();
            }
            self.0
                .insert(key, SegmentTexture::upload(context, path, flattening)?);
        }
        self.0.get(&key)
    }

    pub(crate) fn clear(&mut self) {
        self.0.clear();
    }
}

fn cache_key(path: &Path, flattening: Flattening) -> u64 {
    let mut hasher = DefaultHasher::new();
    flattening.tolerance.to_bits().hash(&mut hasher);
    flattening.open_contours.hash(&mut hasher);
    for command in &path.commands {
        mem::discriminant(command).hash(&mut hasher);
        match *command {
            PathCommand::MoveTo(point) | PathCommand::LineTo(point) => {
                hash_points(&mut hasher, [point]);
            }
            PathCommand::QuadTo(control, point) => hash_points(&mut hasher, [control, point]),
            PathCommand::CubicTo(first, second, point) => {
                hash_points(&mut hasher, [first, second, point]);
            }
            PathCommand::Close => {}
        }
    }
    hasher.finish()
}

fn hash_points<const COUNT: usize>(hasher: &mut impl Hasher, points: [Point; COUNT]) {
    for point in points {
        hasher.write_u32(point.x.to_bits());
        hasher.write_u32(point.y.to_bits());
    }
}

#[cfg(test)]
mod tests {
    use crate::util::flatten::OpenContours;

    use super::*;

    fn flattening(tolerance: f32, open_contours: OpenContours) -> Flattening {
        Flattening {
            tolerance,
            open_contours,
        }
    }

    fn tick() -> Path {
        Path::new()
            .move_to(Point::new(4.0, 12.0))
            .line_to(Point::new(9.0, 17.0))
            .line_to(Point::new(20.0, 6.0))
    }

    #[test]
    fn keys_tell_paths_and_flattenings_apart() {
        let stroke = flattening(0.05, OpenContours::Capped);
        let key = cache_key(&tick(), stroke);
        assert_eq!(key, cache_key(&tick(), stroke));
        assert_ne!(
            key,
            cache_key(&tick(), flattening(0.05, OpenContours::Closed))
        );
        assert_ne!(
            key,
            cache_key(&tick(), flattening(0.1, OpenContours::Capped))
        );
        assert_ne!(key, cache_key(&tick().close(), stroke));
        let moved = Path::new()
            .move_to(Point::new(4.0, 12.0))
            .move_to(Point::new(9.0, 17.0))
            .line_to(Point::new(20.0, 6.0));
        assert_ne!(key, cache_key(&moved, stroke));
    }
}
