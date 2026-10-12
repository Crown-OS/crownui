/// A recognizer's opinion about the pointer it follows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Vote {
    /// Not decided yet.
    Pending,
    /// This is my gesture; I want to win the arena.
    Accept,
    /// This is not my gesture; I leave the arena.
    Reject,
}
