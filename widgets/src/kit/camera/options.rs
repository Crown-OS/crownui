use view::Element;

use super::preview::CameraPreviewKind;

/// The options of a [`camera_preview`](super::camera_preview), chained before its modifiers.
pub trait CameraPreviewOptions: Sized {
    /// Shows the frames as the camera sees them instead of mirrored like a selfie view.
    fn unmirrored(self) -> Self;

    /// Rounds the preview's corners by `radius`.
    fn radius(self, radius: f32) -> Self;
}

impl CameraPreviewOptions for Element<CameraPreviewKind> {
    fn unmirrored(self) -> Self {
        self.configure(|kind| kind.mirrored = false)
    }

    fn radius(self, radius: f32) -> Self {
        self.configure(|kind| kind.radius = radius)
    }
}
