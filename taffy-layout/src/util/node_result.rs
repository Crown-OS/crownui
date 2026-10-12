use taffy::TaffyResult;

/// Taffy only reports errors for child indices, which the engine never uses, so a failure is a bug.
pub(crate) fn debug_assert_node_ok<T>(result: TaffyResult<T>) {
    debug_assert!(
        result.is_ok(),
        "taffy rejected a node operation: {:?}",
        result.as_ref().err()
    );
}
