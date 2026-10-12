use semantics::UiAction;

/// An action requested by assistive technology, in CrownUI terms.
///
/// Requests that the shared [`UiAction`] vocabulary already covers arrive as
/// [`AccessibilityAction::Ui`]; the rest are value, disclosure and focus requests
/// that only screen readers make.
#[derive(Debug, Clone, PartialEq)]
pub enum AccessibilityAction {
    /// A request expressible as a shared UI action.
    Ui(UiAction),
    /// Step a slider, stepper or similar control up.
    Increment,
    /// Step a slider, stepper or similar control down.
    Decrement,
    /// Open a disclosure, menu or combo box.
    Expand,
    /// Close a disclosure, menu or combo box.
    Collapse,
    /// Move keyboard focus away from the node.
    Blur,
    /// Open the node's context menu.
    ShowContextMenu,
    /// Scroll the node's ancestors until the node is visible.
    ScrollIntoView,
}
