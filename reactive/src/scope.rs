use slotmap::new_key_type;
use smallvec::SmallVec;

use crate::handles::{ObserverId, SignalId};

new_key_type! {
    /// Owns reactive nodes; disposing it frees everything created inside.
    pub struct ScopeId;
}

pub(crate) type Cleanup = Box<dyn FnOnce(&mut crate::Runtime)>;
pub(crate) type ErrorHandler = Box<dyn FnMut(&mut crate::Runtime, &crate::ObserverPanic)>;

#[derive(Default)]
pub(crate) struct ScopeSlot {
    pub(crate) parent: Option<ScopeId>,
    pub(crate) children: SmallVec<[ScopeId; 4]>,
    pub(crate) signals: SmallVec<[SignalId; 8]>,
    pub(crate) observers: SmallVec<[ObserverId; 8]>,
    pub(crate) cleanups: Vec<Cleanup>,
    pub(crate) environment: SmallVec<[(std::any::TypeId, SignalId); 2]>,
    pub(crate) error_handler: Option<ErrorHandler>,
}
