use geometry::Point;

/// Data another client (or this one) drags over a window (HTML `DataTransfer.types`).
#[derive(Debug, Clone, PartialEq)]
pub struct DragOffer {
    /// Where the pointer entered, in window-local logical pixels.
    pub position: Point,
    /// MIME types the drag can be read as.
    pub mime_types: Vec<String>,
}

/// A drop the window accepted, read in the MIME type it accepted.
#[derive(Debug, Clone, PartialEq)]
pub struct DroppedData {
    /// Where it was dropped, in window-local logical pixels.
    pub position: Point,
    /// The accepted MIME type.
    pub mime_type: String,
    /// The bytes the source sent.
    pub data: Vec<u8>,
}

/// The picture the platform moves with the pointer during a drag, rendered by the compositor.
#[derive(Debug, Clone, PartialEq)]
pub struct DragIcon {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// Straight-alpha RGBA8 pixels.
    pub rgba: Vec<u8>,
    /// Size the icon is shown at, in logical pixels.
    pub logical_width: u32,
    /// Size the icon is shown at, in logical pixels.
    pub logical_height: u32,
    /// The point of the icon held under the pointer, in logical pixels from its top-left.
    pub hotspot: Point,
}

/// What a drag started by this app carries.
#[derive(Debug, Clone, PartialEq)]
pub struct DragSource {
    /// The same content in every MIME type it is offered as.
    pub items: Vec<(String, Vec<u8>)>,
    /// What follows the pointer, if anything.
    pub icon: Option<DragIcon>,
}

/// How a drag started by this app ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DragOutcome {
    /// A target took the data.
    Dropped,
    /// Released where nothing accepted it, or the platform aborted it.
    Cancelled,
}
