use accesskit::Action;
use semantics::ActionKind;

const SCROLL_ACTIONS: &[Action] = &[
    Action::ScrollUp,
    Action::ScrollDown,
    Action::ScrollLeft,
    Action::ScrollRight,
];

pub(crate) fn accesskit_actions(kind: ActionKind) -> &'static [Action] {
    match kind {
        ActionKind::Click => &[Action::Click],
        ActionKind::ContextMenu | ActionKind::LongPress => &[Action::ShowContextMenu],
        ActionKind::Focus => &[Action::Focus],
        ActionKind::Blur => &[Action::Blur],
        ActionKind::Scroll => SCROLL_ACTIONS,
        ActionKind::SetValue => &[Action::SetValue],
        ActionKind::Increment => &[Action::Increment],
        ActionKind::Decrement => &[Action::Decrement],
        ActionKind::Expand => &[Action::Expand],
        ActionKind::Collapse => &[Action::Collapse],
        ActionKind::DoubleClick | ActionKind::Dismiss | ActionKind::Drag => &[],
    }
}
