/// How much work the UI thread did since the counters were last taken, for benchmarks and
/// devtools: every number should grow with what changed, not with the size of the tree.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct UiCounters {
    /// Layout passes that ran the layout engine.
    pub layout_passes: u32,
    /// Nodes visited while syncing styles into the engine and reading layouts back.
    pub layout_visits: u32,
    /// Nodes visited while collecting render ops.
    pub op_visits: u32,
    /// Render ops sent to the render thread.
    pub ops: u32,
    /// Hit tests performed.
    pub hit_tests: u32,
    /// Nodes visited by hit tests.
    pub hit_visits: u32,
    /// Semantic tree rebuilds.
    pub semantics_rebuilds: u32,
}
