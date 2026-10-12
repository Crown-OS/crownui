use std::ops::{Add, Mul, Neg, Sub};

/// A position in logical pixels.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Point {
    /// Horizontal position.
    pub x: f32,
    /// Vertical position, growing downwards.
    pub y: f32,
}

/// A displacement in logical pixels.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Vector {
    /// Horizontal component.
    pub x: f32,
    /// Vertical component.
    pub y: f32,
}

impl Point {
    /// The origin.
    pub const ZERO: Self = Self::new(0.0, 0.0);

    /// Creates a point.
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// The displacement from the origin to this point.
    pub const fn to_vector(self) -> Vector {
        Vector::new(self.x, self.y)
    }
}

impl Vector {
    /// No displacement.
    pub const ZERO: Self = Self::new(0.0, 0.0);

    /// Creates a vector.
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Euclidean length.
    pub fn length(self) -> f32 {
        self.x.hypot(self.y)
    }
}

impl Add<Vector> for Point {
    type Output = Self;
    fn add(self, offset: Vector) -> Self {
        Self::new(self.x + offset.x, self.y + offset.y)
    }
}

impl Sub<Vector> for Point {
    type Output = Self;
    fn sub(self, offset: Vector) -> Self {
        Self::new(self.x - offset.x, self.y - offset.y)
    }
}

impl Sub for Point {
    type Output = Vector;
    fn sub(self, other: Self) -> Vector {
        Vector::new(self.x - other.x, self.y - other.y)
    }
}

impl Add for Vector {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self::new(self.x + other.x, self.y + other.y)
    }
}

impl Sub for Vector {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self::new(self.x - other.x, self.y - other.y)
    }
}

impl Mul<f32> for Vector {
    type Output = Self;
    fn mul(self, factor: f32) -> Self {
        Self::new(self.x * factor, self.y * factor)
    }
}

impl Neg for Vector {
    type Output = Self;
    fn neg(self) -> Self {
        Self::new(-self.x, -self.y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subtracting_points_gives_the_offset_between_them() {
        assert_eq!(
            Point::new(5.0, 7.0) - Point::new(2.0, 3.0),
            Vector::new(3.0, 4.0)
        );
        assert_eq!(Vector::new(3.0, 4.0).length(), 5.0);
    }
}
