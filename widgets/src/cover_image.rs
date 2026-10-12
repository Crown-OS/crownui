use geometry::{CornerRadii, Point, Rect, Size};
use graphics::{ImageId, Paint};
use semantics::Role;
use view::{Cx, Element, Kind, NodeId};

/// The kind behind [`cover_image`].
pub struct CoverImageKind {
    image: ImageId,
    natural_size: Size,
    radius: f32,
}

impl Kind for CoverImageKind {
    const NAME: &'static str = "cover_image";

    fn mount(self, cx: &mut Cx, node: NodeId) {
        let Self {
            image,
            natural_size,
            radius,
        } = self;
        cx.set_canvas(node, move |_, size, list| {
            list.fill(
                Rect::new(Point::ZERO, size),
                CornerRadii::uniform(radius),
                Paint::image(image, covering(natural_size, size)),
            );
        });
        cx.set_role(node, Role::Image);
    }
}

/// `natural_size` scaled to cover `target`, centred, so the long axis overflows evenly.
fn covering(natural_size: Size, target: Size) -> Rect {
    if natural_size.width <= 0.0 || natural_size.height <= 0.0 {
        return Rect::new(Point::ZERO, target);
    }
    let scale = (target.width / natural_size.width).max(target.height / natural_size.height);
    let width = natural_size.width * scale;
    let height = natural_size.height * scale;
    Rect::from_xywh(
        (target.width - width) / 2.0,
        (target.height - height) / 2.0,
        width,
        height,
    )
}

/// An already registered image (see `Cx::register_image`) scaled to cover the box it is given,
/// centred and cropped to a rounded rect of `radius` — one image-filled shape, so no clip and
/// anti-aliased corners. It never sizes itself, so give it one. The caller owns `image` and
/// releases it.
pub fn cover_image(image: ImageId, natural_size: Size, radius: f32) -> Element<CoverImageKind> {
    Element::new(CoverImageKind {
        image,
        natural_size,
        radius,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_wide_image_overflows_sideways_and_stays_centred() {
        let dest = covering(Size::new(400.0, 100.0), Size::new(100.0, 100.0));
        assert_eq!(dest, Rect::from_xywh(-150.0, 0.0, 400.0, 100.0));
    }

    #[test]
    fn a_tall_image_overflows_vertically() {
        let dest = covering(Size::new(100.0, 400.0), Size::new(200.0, 100.0));
        assert_eq!(dest, Rect::from_xywh(0.0, -350.0, 200.0, 800.0));
    }
}
