use geometry::Vector;
use view::{Corners, Cx, Kind, NodeId, Shadow, StyleProp, ViewSeq};

use crate::{kit::use_theme, util::themed::bind_themed_style};

const BLUR_SIGMA: f32 = 10.0;
const BLUR_RADIUS: f32 = BLUR_SIGMA * 2.0;
const OFFSET: Vector = Vector::new(0.0, 3.0);

/// The kind behind [`soft_shadow`](super::soft_shadow).
pub struct SoftShadowKind<C> {
    pub(super) content: C,
    pub(super) radius: f32,
}

impl<C: ViewSeq> Kind for SoftShadowKind<C> {
    const NAME: &'static str = "kit_soft_shadow";

    fn mount(self, cx: &mut Cx, node: NodeId) {
        let theme = use_theme(cx);
        cx.set_default_style(node, StyleProp::Radius(Corners::All, self.radius));
        bind_themed_style(cx, node, theme, |theme| {
            StyleProp::Shadow(Some(Shadow::new(theme.surface.shadow, BLUR_RADIUS, OFFSET)))
        });
        cx.set_children(node, self.content);
    }
}
