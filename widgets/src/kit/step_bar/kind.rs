use semantics::Role;
use view::{Cx, Kind, NodeId, Prop};

use super::{reached, track::track};
use crate::{kit::use_theme, util::shared_prop::shared_prop};

/// The kind behind [`step_bar`](super::step_bar).
pub struct StepBarKind {
    pub(super) total: usize,
    pub(super) current: Prop<usize>,
}

impl Kind for StepBarKind {
    const NAME: &'static str = "kit_step_bar";

    fn mount(self, cx: &mut Cx, node: NodeId) {
        let theme = use_theme(cx);
        let total = self.total;
        let filled = shared_prop(cx, self.current.map(move |current| reached(total, current)));
        let bar = cx.observe_size(node);
        cx.set_role(node, Role::ProgressBar);
        cx.bind_value(node, move |runtime| {
            format!("step {} of {total}", filled.get(runtime) as usize)
        });
        let tracks: Vec<_> = (0..total)
            .map(|index| track(theme, filled, bar, total, index))
            .collect();
        cx.set_children(node, tracks);
    }
}
