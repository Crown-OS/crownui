/// A PipeWire audio source, named by its `node.name`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SourceId(String);

impl SourceId {
    /// The source whose PipeWire `node.name` is `node_name`.
    pub fn new(node_name: impl Into<String>) -> Self {
        Self(node_name.into())
    }

    /// The PipeWire `node.name`.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A microphone or other capture device currently in the PipeWire graph.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceInfo {
    /// What to pass to [`MicrophoneOptions::source`](crate::MicrophoneOptions::source).
    pub id: SourceId,
    /// A human-readable name, such as "Built-in Audio Analog Stereo".
    pub description: String,
    /// Whether this is the system's default input.
    pub is_default: bool,
}
