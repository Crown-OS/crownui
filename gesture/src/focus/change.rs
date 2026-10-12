/// A move of keyboard focus, so the caller can dispatch `blur` and `focus`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FocusChange<K> {
    /// The member that had focus.
    pub previous: Option<K>,
    /// The member that has focus now.
    pub current: Option<K>,
}
