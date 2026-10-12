use input::Keybind;
use semantics::UiAction;
use serde_json::Value;

use super::tool_calls::ToolFailure;

pub(super) fn parse_action(mut action: Value) -> Result<UiAction, ToolFailure> {
    if let Some(keybind) = action.get_mut("keybind")
        && let Some(shortcut) = keybind.as_str()
    {
        let parsed: Keybind = shortcut.parse()?;
        *keybind = serde_json::to_value(parsed)?;
    }
    serde_json::from_value(action)
        .map_err(|error| ToolFailure::from(format_args!("invalid action: {error}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn documented_action_shapes_parse() {
        let actions = [
            json!({ "action": "click" }),
            json!({ "action": "type_text", "text": "hello" }),
            json!({ "action": "press_key", "keybind": "Ctrl+A" }),
            json!({ "action": "scroll", "delta": { "x": 0, "y": 120 } }),
            json!({ "action": "drag", "to": { "x": 40, "y": 300 } }),
        ];
        for action in actions {
            assert!(parse_action(action.clone()).is_ok(), "{action}");
        }
        assert!(parse_action(json!({ "action": "explode" })).is_err());
    }
}
