use sctk::reexports::client::protocol::wl_data_device_manager::DndAction as WaylandAction;

use crate::{DndAction, DndActions};

/// The protocol's flag for `action`.
pub(crate) fn to_wayland_action(action: DndAction) -> WaylandAction {
    match action {
        DndAction::Copy => WaylandAction::Copy,
        DndAction::Move => WaylandAction::Move,
        DndAction::Ask => WaylandAction::Ask,
    }
}

/// The protocol's flags for `actions`.
pub(crate) fn to_wayland_actions(actions: DndActions) -> WaylandAction {
    actions
        .iter()
        .fold(WaylandAction::empty(), |flags, action| {
            flags | to_wayland_action(action)
        })
}

/// The actions in the protocol's `flags`.
pub(crate) fn from_wayland_actions(flags: WaylandAction) -> DndActions {
    [DndAction::Copy, DndAction::Move, DndAction::Ask]
        .into_iter()
        .filter(|action| flags.contains(to_wayland_action(*action)))
        .fold(DndActions::NONE, |actions, action| actions | action.into())
}

/// The one action the protocol settled on; `None` while there is none.
pub(crate) fn from_wayland_action(flags: WaylandAction) -> Option<DndAction> {
    from_wayland_actions(flags).iter().next()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn actions_round_trip_through_the_protocol_flags() {
        for actions in [
            DndActions::NONE,
            DndActions::COPY,
            DndActions::MOVE | DndActions::ASK,
            DndActions::ALL,
        ] {
            assert_eq!(from_wayland_actions(to_wayland_actions(actions)), actions);
        }
        assert_eq!(
            from_wayland_action(WaylandAction::Move),
            Some(DndAction::Move)
        );
        assert_eq!(from_wayland_action(WaylandAction::empty()), None);
    }
}
