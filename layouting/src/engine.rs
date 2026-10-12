use std::{fmt::Debug, hash::Hash};

use geometry::{Rect, Size};

use crate::LayoutStyle;

/// Space offered to a node along one axis.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AvailableSpace {
    /// A definite amount of logical pixels.
    Definite(f32),
    /// As narrow as content allows (CSS min-content).
    MinContent,
    /// As wide as content wants (CSS max-content).
    MaxContent,
}

impl AvailableSpace {
    /// The definite amount, if any.
    pub fn definite(self) -> Option<f32> {
        match self {
            Self::Definite(length) => Some(length),
            Self::MinContent | Self::MaxContent => None,
        }
    }
}

/// What a measured leaf is asked.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MeasureInput {
    /// Width already decided by the parent, if any.
    pub known_width: Option<f32>,
    /// Height already decided by the parent, if any.
    pub known_height: Option<f32>,
    /// Width on offer.
    pub available_width: AvailableSpace,
    /// Height on offer.
    pub available_height: AvailableSpace,
}

/// A computed box.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct NodeLayout {
    /// Border box relative to the parent's border box.
    pub rect: Rect,
    /// Size of the content, which may exceed `rect` in scroll containers.
    pub content_size: Size,
}

/// A layout algorithm over a tree of nodes it owns.
pub trait LayoutEngine {
    /// Node handle.
    type Node: Copy + Eq + Hash + Debug;

    /// Creates a node; `measured` leaves get their size from the measure closure.
    fn create_node(&mut self, style: &LayoutStyle, measured: bool) -> Self::Node;

    /// Replaces a node's style.
    fn set_style(&mut self, node: Self::Node, style: &LayoutStyle);

    /// Replaces a node's children, in order.
    fn set_children(&mut self, node: Self::Node, children: &[Self::Node]);

    /// Removes a node; its children are detached, not removed.
    fn remove_node(&mut self, node: Self::Node);

    /// Marks a measured leaf's content as changed.
    fn mark_dirty(&mut self, node: Self::Node);

    /// Lays out the tree under `root` in `available` space.
    fn compute_layout<M>(&mut self, root: Self::Node, available: Size, measure: M)
    where
        M: FnMut(Self::Node, MeasureInput) -> Size;

    /// The computed box of `node`.
    fn layout(&self, node: Self::Node) -> NodeLayout;
}
