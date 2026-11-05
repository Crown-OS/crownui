use crate::NodeId;

/// Which passes a node already waits for, so each queue holds it at most once, and whether the
/// layout write-back has to look below it.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct Queued {
    /// In [`DirtyQueues::layout`].
    pub(crate) layout: bool,
    /// In [`DirtyQueues::render`].
    pub(crate) render: bool,
    /// This node or a descendant changed a layout input since the last write-back.
    pub(crate) needs_visit: bool,
}

/// The nodes the next layout pass and the next render-op collection have to look at: every
/// change enqueues the node it touched, so neither pass walks the whole tree.
#[derive(Debug, Default)]
pub(crate) struct DirtyQueues {
    pub(crate) layout: Vec<NodeId>,
    pub(crate) render: Vec<NodeId>,
}
