use geometry::{TRANSPARENT, Vector};
use reactive::{Runtime, Signal};
use view::{Cx, NodeId, Shadow, StyleProp, Styled, View};

use crate::{kit::Theme, stack::vstack, util::themed::themed};

/// How far a glass surface blurs what lies behind it.
const GLASS_BLUR: f32 = 30.0;
const RIM_WIDTH: f32 = 1.0;
const SHADOW_BLUR: f32 = 24.0;
const SHADOW_OFFSET_Y: f32 = 8.0;

/// Makes `node` a frosted-glass surface rounded to `radius` and returns the plate to mount as its
/// first child. The node keeps a transparent one-pixel border for layout and casts the soft
/// shadow; the plate, laid over the node's border box, blurs the backdrop under the menu tint and
/// draws the rim. They are two nodes because a backdrop blur clips its node's own drawing, and
/// with it any shadow the node casts.
pub(crate) fn glass_surface(
    cx: &mut Cx,
    node: NodeId,
    theme: Signal<Theme>,
    radius: f32,
) -> impl View {
    cx.set_default_style(node, StyleProp::BorderColor(TRANSPARENT));
    cx.bind_style(node, glass_shadow(theme));
    glass_plate(theme, radius)
}

/// The soft shadow a glass surface casts, for its shadow-casting node.
pub(crate) fn glass_shadow(
    theme: Signal<Theme>,
) -> impl FnMut(&mut Runtime) -> StyleProp + 'static {
    move |runtime| {
        let color = theme.with(runtime, |theme| theme.surface.shadow);
        let offset = Vector::new(0.0, SHADOW_OFFSET_Y);
        StyleProp::Shadow(Some(Shadow::new(color, SHADOW_BLUR, offset)))
    }
}

/// The frosted plate behind a glass surface's content, over its parent's border box.
pub(crate) fn glass_plate(theme: Signal<Theme>, radius: f32) -> impl View {
    vstack(())
        .absolute()
        .inset(0.0)
        .rounded(radius)
        .border(RIM_WIDTH)
        .backdrop_blur(GLASS_BLUR)
        .bg(themed(theme, |theme| theme.menu.bg))
        .border_color(themed(theme, |theme| theme.menu.border))
        .pointer_events_none()
}

/// Lights `node` up as the accent capsule while `lit` holds. The node draws a one-pixel border
/// and rounds to half its height.
pub(crate) fn bind_capsule_highlight(
    cx: &mut Cx,
    node: NodeId,
    theme: Signal<Theme>,
    lit: impl Fn(&mut Runtime) -> bool + Copy + 'static,
) {
    cx.bind_style(node, move |runtime| {
        let fill = lit(runtime).then(|| theme.with(runtime, |theme| theme.menu.selected_bg));
        StyleProp::Background(fill)
    });
    cx.bind_style(node, move |runtime| {
        let rim = if lit(runtime) {
            theme.with(runtime, |theme| theme.menu.selected_border)
        } else {
            TRANSPARENT
        };
        StyleProp::BorderColor(rim)
    });
}
