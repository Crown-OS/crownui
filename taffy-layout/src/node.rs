use taffy::NodeId;

/// Handle to a node owned by a [`TaffyEngine`](crate::TaffyEngine).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TaffyNode(pub(crate) NodeId);
