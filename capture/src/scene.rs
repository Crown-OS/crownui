use serde::Serialize;

/// One captured frame, as the Figma builder (`figma/builder.js`) rebuilds it.
#[derive(Debug, Clone, Serialize)]
pub struct Scene {
    /// Frame name, e.g. `Request · Dark`.
    pub name: String,
    /// Group the frame belongs to on its page, e.g. `Dialogs`.
    pub group: String,
    /// `Light` or `Dark`.
    pub theme: String,
    /// What the state shows.
    pub description: String,
    /// Window size in logical pixels.
    pub width: f32,
    /// Window size in logical pixels.
    pub height: f32,
    /// Window clear color (`rrggbbaa`).
    pub clear: String,
    /// Figma corner smoothing matching the renderer's corner shape.
    pub corner_smoothing: f32,
    /// The compositor material behind the window, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub material: Option<SceneMaterial>,
    /// The view tree.
    pub root: SceneNode,
}

/// A window's platform material: compositor blur, tint, corners and shadow.
#[derive(Debug, Clone, Serialize)]
pub struct SceneMaterial {
    /// Window corner radius.
    pub corner_radius: f32,
    /// Backdrop blur radius (CSS).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blur: Option<f32>,
    /// Tint over the blurred backdrop (`rrggbbaa`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tint: Option<String>,
    /// Window shadow.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shadow: Option<Shadow>,
}

impl From<windowing::Material> for SceneMaterial {
    fn from(material: windowing::Material) -> Self {
        Self {
            corner_radius: material.corner_radius,
            blur: material.blur.map(|blur| blur.radius),
            tint: material
                .blur
                .map(|blur| crate::util::hex_color::hex_color(blur.tint)),
            shadow: material.shadow.map(|shadow| Shadow {
                color: crate::util::hex_color::hex_color(shadow.color),
                blur: shadow.radius,
                offset: [shadow.offset.x, shadow.offset.y],
            }),
        }
    }
}

/// A view node: a layer placed in its parent.
#[derive(Debug, Clone, Default, Serialize)]
pub struct SceneNode {
    /// Layer name: the semantic label (`button "Save"`) or the element name.
    pub name: String,
    /// Element path in the view tree, e.g. `app > sidebar > row[2]`.
    pub path: String,
    /// Placement in the parent: `a, b, c, d, tx, ty`.
    pub matrix: [f32; 6],
    /// Laid-out width.
    pub width: f32,
    /// Laid-out height.
    pub height: f32,
    /// Opacity of the whole subtree.
    pub opacity: f32,
    /// Corner radii (`tl, tr, br, bl`) of the box fill, stroke and clip.
    pub radii: [f32; 4],
    /// Whether the subtree is clipped to the box.
    pub clip: bool,
    /// Backdrop blur radius (CSS).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backdrop_blur: Option<f32>,
    /// The box's own fill.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fill: Option<Fill>,
    /// The box's own outline.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stroke: Option<StrokeSpec>,
    /// The box's drop shadows.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub shadows: Vec<Shadow>,
    /// Content in paint order.
    pub items: Vec<Item>,
}

/// Content of a node, in its coordinates.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Item {
    /// A child view.
    Node(SceneNode),
    /// A (rounded) rectangle.
    Rect {
        /// Bounds.
        rect: [f32; 4],
        /// Corner radii `tl, tr, br, bl`.
        radii: [f32; 4],
        /// Fill.
        #[serde(skip_serializing_if = "Option::is_none")]
        fill: Option<Fill>,
        /// Outline, centered on the edge.
        #[serde(skip_serializing_if = "Option::is_none")]
        stroke: Option<StrokeSpec>,
        /// Drop shadows.
        #[serde(skip_serializing_if = "Vec::is_empty")]
        shadows: Vec<Shadow>,
    },
    /// A vector path.
    Path {
        /// SVG path data in node coordinates.
        data: String,
        /// Fill.
        #[serde(skip_serializing_if = "Option::is_none")]
        fill: Option<Fill>,
        /// Outline.
        #[serde(skip_serializing_if = "Option::is_none")]
        stroke: Option<StrokeSpec>,
    },
    /// Live text.
    Text(TextItem),
    /// A raster image.
    Image {
        /// Destination.
        rect: [f32; 4],
        /// Key into the carrier's images (16 hex digits).
        image: String,
    },
    /// A clip, opacity layer, backdrop or scroll offset around items.
    Group {
        /// `clip`, `layer`, `scroll` or `top layer`.
        name: String,
        /// Placement: `a, b, c, d, tx, ty`.
        matrix: [f32; 6],
        /// Clip rectangle in the group's coordinates.
        #[serde(skip_serializing_if = "Option::is_none")]
        clip: Option<[f32; 4]>,
        /// Clip radii.
        radii: [f32; 4],
        /// Opacity.
        opacity: f32,
        /// Backdrop blur radius (CSS).
        #[serde(skip_serializing_if = "Option::is_none")]
        backdrop_blur: Option<f32>,
        /// Content.
        items: Vec<Item>,
    },
}

/// Live text with the style it was shaped with.
#[derive(Debug, Clone, Serialize)]
pub struct TextItem {
    /// Top-left of the layout box.
    pub x: f32,
    /// Top-left of the layout box.
    pub y: f32,
    /// Measured width.
    pub width: f32,
    /// Measured height.
    pub height: f32,
    /// Width lines were broken at.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wrap_width: Option<f32>,
    /// Lines laid out.
    pub lines: usize,
    /// The string.
    pub text: String,
    /// Requested family (`Inter Variable` maps to Figma's `Inter`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub family: Option<String>,
    /// Size in logical pixels.
    pub size: f32,
    /// CSS weight.
    pub weight: u16,
    /// Italic.
    pub italic: bool,
    /// Color (`rrggbbaa`).
    pub color: String,
    /// Extra spacing between letters in pixels.
    pub letter_spacing: f32,
    /// `start`, `center`, `end` or `justify`.
    pub align: String,
    /// Truncation line limit.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_lines: Option<u32>,
    /// Glyph blur (CSS).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blur: Option<f32>,
    /// Placement when not a plain translation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matrix: Option<[f32; 6]>,
}

/// A paint.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Fill {
    /// A flat color (`rrggbbaa`).
    Solid {
        /// Color.
        color: String,
        /// Layer blur (CSS).
        #[serde(skip_serializing_if = "Option::is_none")]
        blur: Option<f32>,
    },
    /// A linear gradient between two points in node coordinates.
    Linear {
        /// Start point.
        start: [f32; 2],
        /// End point.
        end: [f32; 2],
        /// `(offset, rrggbbaa)` stops.
        stops: Vec<(f32, String)>,
        /// Layer blur (CSS).
        #[serde(skip_serializing_if = "Option::is_none")]
        blur: Option<f32>,
    },
    /// An image drawn into `rect`.
    Image {
        /// Key into the carrier's images (16 hex digits).
        image: String,
        /// Where the image lands.
        rect: [f32; 4],
    },
}

/// An outline.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct StrokeSpec {
    /// Paint.
    pub fill: Fill,
    /// Width.
    pub width: f32,
    /// `butt`, `round` or `square`.
    pub cap: &'static str,
    /// `miter`, `round` or `bevel`.
    pub join: &'static str,
}

/// A CSS `box-shadow`.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Shadow {
    /// Color (`rrggbbaa`).
    pub color: String,
    /// Blur radius.
    pub blur: f32,
    /// Offset.
    pub offset: [f32; 2],
}
