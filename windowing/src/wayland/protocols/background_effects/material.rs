/// A non-premultiplied ARGB8888 colour.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Argb(pub u32);

impl Argb {
    pub const TRANSPARENT: Self = Self(0);

    pub const fn new(alpha: u8, red: u8, green: u8, blue: u8) -> Self {
        Self(u32::from_be_bytes([alpha, red, green, blue]))
    }
}

/// Geometry in surface-local logical pixels, rendered by the compositor as a signed distance field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShapePrimitive {
    RoundedRect {
        x: i32,
        y: i32,
        width: u32,
        height: u32,
        radius: u32,
    },
    Circle {
        center_x: i32,
        center_y: i32,
        radius: u32,
    },
}

/// The union of its primitives.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct Shape(pub Vec<ShapePrimitive>);

impl FromIterator<ShapePrimitive> for Shape {
    fn from_iter<I: IntoIterator<Item = ShapePrimitive>>(primitives: I) -> Self {
        Self(primitives.into_iter().collect())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Blur {
    /// `None` covers the whole surface, clipped by the corner radius.
    pub shape: Option<Shape>,
    pub radius: u32,
    pub tint: Argb,
    /// Chroma multiplier: 1.0 leaves the backdrop alone, above 1.0 adds vibrancy.
    pub saturation: f64,
}

impl Default for Blur {
    fn default() -> Self {
        Self {
            shape: None,
            radius: 0,
            tint: Argb::TRANSPARENT,
            saturation: 1.0,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Shadow {
    /// `None` casts the whole surface, clipped by the corner radius.
    pub shape: Option<Shape>,
    pub radius: u32,
    pub offset_x: i32,
    pub offset_y: i32,
    pub color: Argb,
}

/// The material the compositor renders behind and around a window.
/// The default value turns every effect off.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct BackgroundEffect {
    pub corner_radius: u32,
    /// Width of the refractive rim along the blur shape's edge.
    pub border_width: u32,
    pub blur: Option<Blur>,
    pub shadow: Option<Shadow>,
}

impl Argb {
    pub fn from_color(color: geometry::Color) -> Self {
        let [red, green, blue, alpha] = color.to_rgba8();
        Self::new(alpha, red, green, blue)
    }
}

fn rounded_length(length: f32) -> u32 {
    length.max(0.0).round() as u32
}

impl Shape {
    pub fn from_material_shapes(shapes: &[crate::MaterialShape]) -> Self {
        shapes
            .iter()
            .map(|shape| ShapePrimitive::RoundedRect {
                x: shape.rect.min_x().round() as i32,
                y: shape.rect.min_y().round() as i32,
                width: rounded_length(shape.rect.size.width),
                height: rounded_length(shape.rect.size.height),
                radius: rounded_length(shape.radii.max()),
            })
            .collect()
    }
}

impl BackgroundEffect {
    /// Maps a neutral material onto the CrownOS protocol; `blur_shape` limits the blur
    /// to part of the surface.
    pub fn from_material(material: &crate::Material, blur_shape: Option<Shape>) -> Self {
        Self {
            corner_radius: rounded_length(material.corner_radius),
            border_width: rounded_length(material.border_width),
            blur: material.blur.map(|blur| Blur {
                shape: blur_shape,
                radius: rounded_length(blur.radius),
                tint: Argb::from_color(blur.tint),
                saturation: f64::from(blur.saturation),
            }),
            shadow: material.shadow.map(|shadow| Shadow {
                shape: None,
                radius: rounded_length(shadow.radius),
                offset_x: shadow.offset.x.round() as i32,
                offset_y: shadow.offset.y.round() as i32,
                color: Argb::from_color(shadow.color),
            }),
        }
    }
}
