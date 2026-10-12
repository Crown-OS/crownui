use accesskit::{Action, ActionData, ActionRequest, ScrollUnit, TreeId};
use geometry::Vector;
use semantics::{SemanticId, UiAction};

use crate::{AccessibilityAction, node::SYNTHETIC_WINDOW_ID};

/// Logical pixels scrolled by one line-sized scroll request.
pub const SCROLL_LINE_STEP: f32 = 40.0;

/// Logical pixels scrolled by one page-sized scroll request.
pub const SCROLL_PAGE_STEP: f32 = 400.0;

/// Translates an AccessKit action request into the semantic node it targets and the
/// action to perform there.
///
/// Returns `None` for requests CrownUI does not handle, for requests aimed at
/// [`SYNTHETIC_WINDOW_ID`] or at subtrees, and for value requests without a value.
pub fn action_request_to_ui(request: &ActionRequest) -> Option<(SemanticId, AccessibilityAction)> {
    if request.target_tree != TreeId::ROOT || request.target_node == SYNTHETIC_WINDOW_ID {
        return None;
    }
    let data = request.data.as_ref();
    let action = match request.action {
        Action::Click => AccessibilityAction::Ui(UiAction::Click),
        Action::Focus => AccessibilityAction::Ui(UiAction::Focus),
        Action::ShowTooltip => AccessibilityAction::Ui(UiAction::Hover),
        Action::SetValue | Action::ReplaceSelectedText => {
            AccessibilityAction::Ui(UiAction::TypeText {
                text: requested_text(data?)?,
            })
        }
        Action::ScrollUp => scroll(Vector::new(0.0, -1.0), data),
        Action::ScrollDown => scroll(Vector::new(0.0, 1.0), data),
        Action::ScrollLeft => scroll(Vector::new(-1.0, 0.0), data),
        Action::ScrollRight => scroll(Vector::new(1.0, 0.0), data),
        Action::Blur => AccessibilityAction::Blur,
        Action::Increment => AccessibilityAction::Increment,
        Action::Decrement => AccessibilityAction::Decrement,
        Action::Expand => AccessibilityAction::Expand,
        Action::Collapse => AccessibilityAction::Collapse,
        Action::ShowContextMenu => AccessibilityAction::ShowContextMenu,
        Action::ScrollIntoView => AccessibilityAction::ScrollIntoView,
        _ => return None,
    };
    Some((SemanticId(request.target_node.0), action))
}

fn requested_text(data: &ActionData) -> Option<String> {
    match data {
        ActionData::Value(text) => Some(text.to_string()),
        ActionData::NumericValue(number) => Some(number.to_string()),
        _ => None,
    }
}

fn scroll(direction: Vector, data: Option<&ActionData>) -> AccessibilityAction {
    let step = match data {
        Some(ActionData::ScrollUnit(ScrollUnit::Page)) => SCROLL_PAGE_STEP,
        _ => SCROLL_LINE_STEP,
    };
    AccessibilityAction::Ui(UiAction::Scroll {
        delta: direction * step,
    })
}

#[cfg(test)]
mod tests {
    use accesskit::NodeId;

    use super::*;

    fn request(action: Action, data: Option<ActionData>) -> ActionRequest {
        ActionRequest {
            action,
            target_tree: TreeId::ROOT,
            target_node: NodeId(42),
            data,
        }
    }

    fn translated(action: Action, data: Option<ActionData>) -> Option<AccessibilityAction> {
        action_request_to_ui(&request(action, data)).map(|(id, action)| {
            assert_eq!(id, SemanticId(42));
            action
        })
    }

    #[test]
    fn requests_map_to_ui_actions() {
        assert_eq!(
            translated(Action::Click, None),
            Some(AccessibilityAction::Ui(UiAction::Click))
        );
        assert_eq!(
            translated(Action::Focus, None),
            Some(AccessibilityAction::Ui(UiAction::Focus))
        );
        assert_eq!(
            translated(Action::SetValue, Some(ActionData::Value("hello".into()))),
            Some(AccessibilityAction::Ui(UiAction::TypeText {
                text: "hello".into()
            }))
        );
        assert_eq!(
            translated(Action::ScrollDown, None),
            Some(AccessibilityAction::Ui(UiAction::Scroll {
                delta: Vector::new(0.0, SCROLL_LINE_STEP)
            }))
        );
        assert_eq!(
            translated(
                Action::ScrollLeft,
                Some(ActionData::ScrollUnit(ScrollUnit::Page))
            ),
            Some(AccessibilityAction::Ui(UiAction::Scroll {
                delta: Vector::new(-SCROLL_PAGE_STEP, 0.0)
            }))
        );
        assert_eq!(
            translated(Action::Increment, None),
            Some(AccessibilityAction::Increment)
        );
    }

    #[test]
    fn unusable_requests_are_ignored() {
        assert_eq!(translated(Action::SetValue, None), None);
        assert_eq!(translated(Action::HideTooltip, None), None);
        let mut window_click = request(Action::Click, None);
        window_click.target_node = SYNTHETIC_WINDOW_ID;
        assert_eq!(action_request_to_ui(&window_click), None);
    }
}
