use geometry::{Color, CornerRadii, Insets, Point, Rect, Size, Transform, Vector};

/// A value that can be animated: a vector space with a size measure.
pub trait Animatable: Copy + PartialEq + Send + 'static {
    /// Component-wise sum.
    fn add(self, other: Self) -> Self;
    /// Component-wise difference.
    fn sub(self, other: Self) -> Self;
    /// Every component multiplied by `factor`.
    fn scale(self, factor: f32) -> Self;
    /// The largest absolute component, used to decide when motion has settled.
    fn magnitude(self) -> f32;
    /// Linear interpolation; `progress` 0 is `self`, 1 is `target`.
    fn lerp(self, target: Self, progress: f32) -> Self {
        self.add(target.sub(self).scale(progress))
    }
}

impl Animatable for f32 {
    fn add(self, other: Self) -> Self {
        self + other
    }
    fn sub(self, other: Self) -> Self {
        self - other
    }
    fn scale(self, factor: f32) -> Self {
        self * factor
    }
    fn magnitude(self) -> f32 {
        self.abs()
    }
}

macro_rules! componentwise {
    ($type:ty { $($field:ident),+ }) => {
        impl Animatable for $type {
            fn add(self, other: Self) -> Self {
                Self { $($field: self.$field + other.$field),+ }
            }
            fn sub(self, other: Self) -> Self {
                Self { $($field: self.$field - other.$field),+ }
            }
            fn scale(self, factor: f32) -> Self {
                Self { $($field: self.$field * factor),+ }
            }
            fn magnitude(self) -> f32 {
                0.0_f32$(.max(self.$field.abs()))+
            }
        }
    };
}

componentwise!(Point { x, y });
componentwise!(Vector { x, y });
componentwise!(Size { width, height });
componentwise!(Color { r, g, b, a });
componentwise!(Insets { top, right, bottom, left });
componentwise!(CornerRadii { top_left, top_right, bottom_right, bottom_left });

impl Animatable for Rect {
    fn add(self, other: Self) -> Self {
        Self::new(self.origin.add(other.origin), self.size.add(other.size))
    }
    fn sub(self, other: Self) -> Self {
        Self::new(self.origin.sub(other.origin), self.size.sub(other.size))
    }
    fn scale(self, factor: f32) -> Self {
        Self::new(self.origin.scale(factor), self.size.scale(factor))
    }
    fn magnitude(self) -> f32 {
        self.origin.magnitude().max(self.size.magnitude())
    }
}

impl Animatable for Transform {
    fn add(self, other: Self) -> Self {
        Self {
            matrix: std::array::from_fn(|index| self.matrix[index] + other.matrix[index]),
        }
    }
    fn sub(self, other: Self) -> Self {
        Self {
            matrix: std::array::from_fn(|index| self.matrix[index] - other.matrix[index]),
        }
    }
    fn scale(self, factor: f32) -> Self {
        Self {
            matrix: self.matrix.map(|value| value * factor),
        }
    }
    fn magnitude(self) -> f32 {
        self.matrix.iter().fold(0.0, |largest, value| largest.max(value.abs()))
    }
}
