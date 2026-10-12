#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct LogicalSize {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct PhysicalSize {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct LogicalPosition {
    pub x: f64,
    pub y: f64,
}

impl LogicalSize {
    pub const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    pub fn to_physical(self, scale: ScaleFactor) -> PhysicalSize {
        let scale_length = |length: u32| (f64::from(length) * scale.get()).round() as u32;
        PhysicalSize {
            width: scale_length(self.width),
            height: scale_length(self.height),
        }
    }
}

impl From<(f64, f64)> for LogicalPosition {
    fn from((x, y): (f64, f64)) -> Self {
        Self { x, y }
    }
}

/// A surface scale stored in 120ths, the unit `wp_fractional_scale_v1` speaks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ScaleFactor(u32);

impl ScaleFactor {
    pub(crate) const DENOMINATOR: u32 = 120;
    pub const IDENTITY: Self = Self(Self::DENOMINATOR);

    pub const fn from_120ths(numerator: u32) -> Self {
        Self(numerator)
    }

    pub const fn from_integer(scale: u32) -> Self {
        Self(scale * Self::DENOMINATOR)
    }

    pub const fn as_120ths(self) -> u32 {
        self.0
    }

    pub fn get(self) -> f64 {
        f64::from(self.0) / f64::from(Self::DENOMINATOR)
    }
}

impl Default for ScaleFactor {
    fn default() -> Self {
        Self::IDENTITY
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fractional_scale_rounds_to_whole_pixels() {
        let size = LogicalSize::new(801, 601).to_physical(ScaleFactor::from_120ths(150));
        assert_eq!(
            size,
            PhysicalSize {
                width: 1001,
                height: 751
            }
        );
    }

    #[test]
    fn integer_scale_is_exact() {
        let size = LogicalSize::new(800, 600).to_physical(ScaleFactor::from_integer(2));
        assert_eq!(
            size,
            PhysicalSize {
                width: 1600,
                height: 1200
            }
        );
    }
}
