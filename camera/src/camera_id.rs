/// A PipeWire camera, named by its `node.name`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CameraId(String);

impl CameraId {
    /// The camera whose PipeWire `node.name` is `node_name`.
    pub fn new(node_name: impl Into<String>) -> Self {
        Self(node_name.into())
    }

    /// The PipeWire `node.name`.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A camera currently in the PipeWire graph.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CameraInfo {
    /// What to pass to [`CameraOptions::camera`](crate::CameraOptions::camera).
    pub id: CameraId,
    /// A human-readable name, such as "Integrated Camera".
    pub description: String,
}
