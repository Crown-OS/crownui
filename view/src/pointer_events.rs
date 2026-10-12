/// Whether a node is a target for pointer events (CSS `pointer-events`).
///
/// On a window whose root is `None`, the nodes set back to `Auto` also form the surface's input
/// region: everywhere else, clicks fall through to whatever is behind the window.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum PointerEvents {
    /// The node and its subtree receive pointer events.
    #[default]
    Auto,
    /// The node is transparent to the pointer; descendants inherit this unless they opt back in.
    None,
}
