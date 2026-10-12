use std::ops::BitOr;

/// The stacking band a layer surface lives in, from below every window to above everything.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum Layer {
    /// Under the desktop's windows, e.g. a wallpaper.
    Background,
    /// Under windows but above the background, e.g. desktop icons.
    Bottom,
    /// Above normal windows, e.g. a panel or a dock.
    #[default]
    Top,
    /// Above everything, fullscreen windows included, e.g. a lock screen or a notification.
    Overlay,
}

/// Edges of the output a layer surface is attached to.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Anchor(u8);

impl Anchor {
    /// Attached to no edge: centered on the output.
    pub const NONE: Self = Self(0);
    /// The top edge.
    pub const TOP: Self = Self(1);
    /// The bottom edge.
    pub const BOTTOM: Self = Self(2);
    /// The left edge.
    pub const LEFT: Self = Self(4);
    /// The right edge.
    pub const RIGHT: Self = Self(8);
    /// Every edge: the surface covers the output.
    pub const ALL: Self = Self(15);

    /// Whether every edge of `other` is part of this anchor.
    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    /// The edges as the bitmask the layer-shell protocol uses (top 1, bottom 2, left 4, right 8).
    pub const fn bits(self) -> u32 {
        self.0 as u32
    }

    /// Whether the surface is stretched horizontally between the left and right edges.
    pub const fn spans_width(self) -> bool {
        self.contains(Self(Self::LEFT.0 | Self::RIGHT.0))
    }

    /// Whether the surface is stretched vertically between the top and bottom edges.
    pub const fn spans_height(self) -> bool {
        self.contains(Self(Self::TOP.0 | Self::BOTTOM.0))
    }
}

impl BitOr for Anchor {
    type Output = Self;

    fn bitor(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

/// Whether and how a layer surface takes keyboard focus.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum KeyboardInteractivity {
    /// Never focused; keys go to the windows below.
    #[default]
    None,
    /// Takes every key while mapped (lock screens, launchers).
    Exclusive,
    /// Focused like a window when the user clicks it.
    OnDemand,
}

/// How long a layer surface is along one axis.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum LayerExtent {
    /// Fill the space between two opposite anchors; the surface must be anchored to both.
    #[default]
    Stretch,
    /// A fixed length in logical pixels.
    Fixed(u32),
    /// The output's whole length along this axis, without anchoring both edges. A top panel uses it
    /// to keep room below itself for popups while reserving only its own height.
    Output,
}

/// Space kept between a layer surface and the edges it is anchored to, in logical pixels.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct LayerMargin {
    /// From the top edge.
    pub top: i32,
    /// From the right edge.
    pub right: i32,
    /// From the bottom edge.
    pub bottom: i32,
    /// From the left edge.
    pub left: i32,
}

/// A desktop-shell surface (panel, dock, wallpaper, overlay) instead of an application window.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LayerOptions {
    /// What the surface is, for the compositor's rules (e.g. `"crownbar"`).
    pub namespace: String,
    /// Stacking band.
    pub layer: Layer,
    /// Edges it is attached to.
    pub anchor: Anchor,
    /// Width along the output.
    pub width: LayerExtent,
    /// Height along the output.
    pub height: LayerExtent,
    /// Space reserved from the anchored edge so windows are not placed under the surface; 0
    /// reserves nothing and -1 asks to cover other surfaces' reserved space as well.
    pub exclusive_zone: i32,
    /// Distance from the anchored edges.
    pub margin: LayerMargin,
    /// Keyboard focus behavior.
    pub keyboard: KeyboardInteractivity,
}

impl LayerOptions {
    /// A surface in `namespace` on the top layer, stretched across the edges it is anchored to.
    pub fn new(namespace: impl Into<String>, anchor: Anchor) -> Self {
        Self {
            namespace: namespace.into(),
            layer: Layer::default(),
            anchor,
            width: LayerExtent::default(),
            height: LayerExtent::default(),
            exclusive_zone: 0,
            margin: LayerMargin::default(),
            keyboard: KeyboardInteractivity::default(),
        }
    }

    /// Places it in `layer`.
    pub fn layer(mut self, layer: Layer) -> Self {
        self.layer = layer;
        self
    }

    /// Sets the width.
    pub fn width(mut self, width: LayerExtent) -> Self {
        self.width = width;
        self
    }

    /// Sets the height.
    pub fn height(mut self, height: LayerExtent) -> Self {
        self.height = height;
        self
    }

    /// Reserves `zone` logical pixels from the anchored edge.
    pub fn exclusive_zone(mut self, zone: i32) -> Self {
        self.exclusive_zone = zone;
        self
    }

    /// Keeps `margin` from the anchored edges.
    pub fn margin(mut self, margin: LayerMargin) -> Self {
        self.margin = margin;
        self
    }

    /// Sets keyboard focus behavior.
    pub fn keyboard(mut self, keyboard: KeyboardInteractivity) -> Self {
        self.keyboard = keyboard;
        self
    }

    /// The size to request, in logical pixels, with 0 where the compositor decides. `output` is the
    /// output's logical size when known; `fallback` stands in for it until then.
    pub fn requested_size(&self, output: Option<(u32, u32)>, fallback: (u32, u32)) -> (u32, u32) {
        let (output_width, output_height) = output.unwrap_or(fallback);
        (
            resolve(
                self.width,
                self.anchor.spans_width(),
                output_width,
                fallback.0,
            ),
            resolve(
                self.height,
                self.anchor.spans_height(),
                output_height,
                fallback.1,
            ),
        )
    }

    /// Whether a change of output size changes the requested size.
    pub fn follows_output(&self) -> bool {
        self.width == LayerExtent::Output || self.height == LayerExtent::Output
    }
}

/// One axis: 0 lets the compositor fill between two anchors, which is only legal when both are
/// set, so a stretch without them falls back to a fixed length.
fn resolve(extent: LayerExtent, spans: bool, output: u32, fallback: u32) -> u32 {
    match extent {
        LayerExtent::Stretch if spans => 0,
        LayerExtent::Stretch => fallback.max(1),
        LayerExtent::Fixed(length) => length.max(1),
        LayerExtent::Output => output.max(1),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_top_panel_spans_the_width_and_takes_the_output_height() {
        let options = LayerOptions::new("bar", Anchor::TOP | Anchor::LEFT | Anchor::RIGHT)
            .height(LayerExtent::Output);
        assert_eq!(
            options.requested_size(Some((2560, 1440)), (800, 600)),
            (0, 1440)
        );
        assert_eq!(options.requested_size(None, (800, 600)), (0, 600));
        assert!(options.follows_output());
    }

    #[test]
    fn a_stretch_without_both_anchors_never_requests_zero() {
        let options = LayerOptions::new("toast", Anchor::TOP);
        assert_eq!(options.requested_size(None, (320, 80)), (320, 80));
    }

    #[test]
    fn anchors_combine_like_the_protocol_bitmask() {
        let anchor = Anchor::TOP | Anchor::LEFT | Anchor::RIGHT;
        assert_eq!(anchor.bits(), 13);
        assert!(anchor.spans_width());
        assert!(!anchor.spans_height());
        assert!(Anchor::ALL.spans_height());
    }
}
