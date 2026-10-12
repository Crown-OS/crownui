use geometry::Size;
use view::{Cx, Element, Kind, NodeId};

/// The kind behind [`image`].
pub struct ImageKind {
    width: u32,
    height: u32,
    rgba: Vec<u8>,
}

impl Kind for ImageKind {
    const NAME: &'static str = "image";

    fn mount(self, cx: &mut Cx, node: NodeId) {
        let id = cx.register_image(self.width, self.height, self.rgba);
        cx.release_image_on_cleanup(id);
        cx.set_image(node, id, Size::new(self.width as f32, self.height as f32));
    }
}

/// An image from straight-alpha RGBA8 pixels, sized to its pixel dimensions unless styled. The
/// pixels are freed from the renderer when the image unmounts.
pub fn image(width: u32, height: u32, rgba: Vec<u8>) -> Element<ImageKind> {
    Element::new(ImageKind {
        width,
        height,
        rgba,
    })
}
