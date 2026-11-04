use geometry::Point;

use crate::{DndAction, DndActions};

/// Data another client (or this one) drags over a window (HTML `DataTransfer.types`).
#[derive(Debug, Clone, PartialEq)]
pub struct DragOffer {
    /// Where the pointer entered, in window-local logical pixels.
    pub position: Point,
    /// MIME types the drag can be read as.
    pub mime_types: Vec<String>,
    /// What the source allows the drop to do.
    pub actions: DndActions,
}

/// How a window takes the drag over it: read as `mime_type`, preferring `action`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DropAcceptance {
    /// The MIME type the drop is read as.
    pub mime_type: String,
    /// The action the window prefers; the platform settles on one the source allows.
    pub action: DndAction,
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
    /// The action the source and the window settled on.
    pub action: DndAction,
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
#[derive(Debug, Clone, Default, PartialEq)]
pub struct DragSource {
    /// The same content in every MIME type it is offered as.
    pub items: Vec<(String, Vec<u8>)>,
    /// What follows the pointer, if anything.
    pub icon: Option<DragIcon>,
    /// What a target may do with the data; copy or move unless narrowed.
    pub actions: DndActions,
}

/// How a drag started by this app ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DragOutcome {
    /// A target took the data with this action; after [`DndAction::Move`] the source removes
    /// its original.
    Dropped(DndAction),
    /// Released where nothing accepted it, or the platform aborted it.
    Cancelled,
}
