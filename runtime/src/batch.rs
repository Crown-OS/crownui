use graphics::{ImageId, Renderer, TextLayoutId};
use view::{ImageUpload, RenderOp};
use windowing::SurfaceState;

/// Everything the UI thread sends the render thread for one update.
#[derive(Debug)]
pub struct RenderBatch<T> {
    /// Tree changes.
    pub ops: Vec<RenderOp>,
    /// Newly shaped text layouts.
    pub text_uploads: Vec<(TextLayoutId, T)>,
    /// Text layouts no node uses anymore.
    pub text_releases: Vec<TextLayoutId>,
    /// Images to upload.
    pub images: Vec<ImageUpload>,
    /// Images to free once no node draws them.
    pub image_releases: Vec<ImageId>,
    /// Surface state to commit with the next frame.
    pub surface: Option<SurfaceState>,
}

impl<T> RenderBatch<T> {
    /// Whether the batch carries nothing.
    pub fn is_empty(&self) -> bool {
        self.ops.is_empty()
            && self.text_uploads.is_empty()
            && self.text_releases.is_empty()
            && self.images.is_empty()
            && self.image_releases.is_empty()
            && self.surface.is_none()
    }

    /// Hands new resources to `renderer`; returns the ops, surface state and releases for the
    /// render tree (which frees a resource only once no node, exiting ones included, uses it).
    pub(crate) fn upload<R: Renderer<TextLayout = T>>(
        self,
        renderer: &mut R,
    ) -> (Vec<RenderOp>, Option<SurfaceState>, Releases) {
        for (id, layout) in self.text_uploads {
            renderer.upload_text(id, layout);
        }
        for image in self.images {
            if let Err(error) =
                renderer.upload_image(image.id, image.width, image.height, &image.rgba)
            {
                tracing::error!("CUI0401: image {} rejected: {error}", image.id.0);
            }
        }
        let releases = Releases {
            text: self.text_releases,
            images: self.image_releases,
        };
        (self.ops, self.surface, releases)
    }
}

/// Renderer resources the UI no longer references.
#[derive(Debug, Default)]
pub(crate) struct Releases {
    pub(crate) text: Vec<TextLayoutId>,
    pub(crate) images: Vec<ImageId>,
}
