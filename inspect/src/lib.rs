#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

use serde::{Deserialize, Serialize};

/// Bumped on any breaking change to these types.
pub const SNAPSHOT_VERSION: u32 = 1;

/// One laid-out node.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayoutNodeInfo {
    /// Component path, e.g. `app > sidebar > row[42] > button`.
    pub path: String,
    /// Window-relative rect: `[x, y, width, height]` in logical pixels.
    pub rect: [f32; 4],
    /// Human-readable style summary (`flex_col gap8 p16`).
    pub style: String,
}

/// The layout tree, flattened in paint order.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct LayoutSnapshot {
    /// Nodes in paint order.
    pub nodes: Vec<LayoutNodeInfo>,
}

/// Reactive graph totals and the scope tree.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SignalGraphSnapshot {
    /// Live signals, memos and stores.
    pub signals: usize,
    /// Live effects and memos.
    pub observers: usize,
    /// Live scopes.
    pub scopes: usize,
    /// Component scopes by path.
    pub components: Vec<String>,
}

/// One repaint boundary.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BoundaryInfo {
    /// Boundary id.
    pub id: u64,
    /// Component path that owns it.
    pub path: String,
    /// Number of recorded commands.
    pub commands: usize,
    /// Frame at which it was last re-recorded.
    pub recorded_at_frame: u64,
}

/// The render tree.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct RenderTreeSnapshot {
    /// Cached boundaries.
    pub boundaries: Vec<BoundaryInfo>,
    /// Properties currently animating, as `path.property`.
    pub active_animations: Vec<String>,
}

/// Timings of one frame, in microseconds.
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct FrameStats {
    /// Frame number.
    pub frame: u64,
    /// Reserved for a separate reactive-update timing; currently included in `layout_us`.
    pub update_us: u32,
    /// UI-thread work of the latest update: reactive flush, layout, text shaping, op collection.
    pub layout_us: u32,
    /// Recording changed boundaries.
    pub record_us: u32,
    /// Composing and submitting to the GPU.
    pub render_us: u32,
}

/// Everything at once.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct InspectSnapshot {
    /// [`SNAPSHOT_VERSION`] of the producer.
    pub version: u32,
    /// The semantic tree as indented text.
    pub semantics: String,
    /// Layout.
    pub layout: LayoutSnapshot,
    /// Reactive graph.
    pub signals: SignalGraphSnapshot,
    /// Render tree.
    pub render: RenderTreeSnapshot,
    /// The latest frame's composition with boundaries inlined, as text.
    pub draw_dump: String,
    /// Recent frames, oldest first.
    pub frames: Vec<FrameStats>,
}
