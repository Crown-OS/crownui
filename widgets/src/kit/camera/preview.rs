use geometry::{CornerRadii, Point, Rect, Transform};
use graphics::Paint;
use semantics::Role;
use view::{Cx, Element, Kind, NodeId};

use crate::cover_image::covering;

use super::CameraSignals;

/// The kind behind [`camera_preview`].
pub struct CameraPreviewKind {
    camera: CameraSignals,
    pub(super) mirrored: bool,
    pub(super) radius: f32,
}

impl Kind for CameraPreviewKind {
    const NAME: &'static str = "camera_preview";

    fn mount(self, cx: &mut Cx, node: NodeId) {
        let Self {
            camera,
            mirrored,
            radius,
        } = self;
        cx.set_canvas(node, move |runtime, size, list| {
            let natural_size = camera.frame_size(runtime).unwrap_or(size);
            if mirrored {
                list.save();
                list.transform(
                    Transform::scale(-1.0, 1.0).about(Point::new(size.width / 2.0, 0.0)),
                );
            }
            list.fill(
                Rect::new(Point::ZERO, size),
                CornerRadii::uniform(radius),
                Paint::image(camera.image, covering(natural_size, size)),
            );
            if mirrored {
                list.restore();
            }
        });
        cx.set_role(node, Role::Image);
    }
}

/// The live frames of a [`use_camera`](super::use_camera) camera, scaled to cover the box it is
/// given and mirrored like a selfie view (see [`CameraPreviewOptions`](super::CameraPreviewOptions)).
/// Black until the first frame. It never sizes itself, so give it one. Frames redraw only this
/// box, on the render thread, without waking the UI.
pub fn camera_preview(camera: CameraSignals) -> Element<CameraPreviewKind> {
    Element::new(CameraPreviewKind {
        camera,
        mirrored: true,
        radius: 0.0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mirroring_flips_about_the_vertical_centre_line() {
        let mirror = Transform::scale(-1.0, 1.0).about(Point::new(50.0, 0.0));
        assert_eq!(mirror.apply(Point::new(0.0, 7.0)), Point::new(100.0, 7.0));
        assert_eq!(mirror.apply(Point::new(100.0, 7.0)), Point::new(0.0, 7.0));
    }
}
