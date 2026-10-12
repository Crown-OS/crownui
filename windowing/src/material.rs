use geometry::{Color, Vector};

/// Backdrop blur of a [`Material`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MaterialBlur {
    /// Blur radius in logical pixels.
    pub radius: f32,
    /// Color laid over the blurred backdrop.
    pub tint: Color,
    /// Chroma multiplier: 1 leaves the backdrop alone, above 1 adds vibrancy.
    pub saturation: f32,
}

/// Drop shadow of a [`Material`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MaterialShadow {
    /// Blur radius in logical pixels.
    pub radius: f32,
    /// Offset from the casting shape.
    pub offset: Vector,
    /// Shadow color.
    pub color: Color,
}

/// A platform-rendered window background: blur, tint, rim and shadow drawn by the
/// compositor (crownos-background-effects on CrownOS) behind and around a surface.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Material {
    /// Corner radius of the window shape.
    pub corner_radius: f32,
    /// Width of the refractive rim along the edge.
    pub border_width: f32,
    /// Backdrop blur, if any.
    pub blur: Option<MaterialBlur>,
    /// Shadow, if any.
    pub shadow: Option<MaterialShadow>,
}

impl Material {
    /// A thin frosted-glass material.
    pub fn thin() -> Self {
        Self {
            corner_radius: 16.0,
            border_width: 1.0,
            blur: Some(MaterialBlur {
                radius: 24.0,
                tint: Color::new(0.12, 0.13, 0.17, 0.25),
                saturation: 1.6,
            }),
            shadow: Some(MaterialShadow {
                radius: 24.0,
                offset: Vector::new(0.0, 8.0),
                color: Color::new(0.0, 0.0, 0.0, 0.35),
            }),
        }
    }

    /// A thicker, more opaque material.
    pub fn thick() -> Self {
        let mut material = Self::thin();
        if let Some(blur) = material.blur.as_mut() {
            blur.radius = 48.0;
            blur.tint.a = 0.55;
        }
        material
    }
}
