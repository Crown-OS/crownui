use anyhow::{Context, Result, bail};
use serde_json::{Value, json};

const SIMPLE_ACTIONS: [&str; 4] = ["click", "double_click", "hover", "focus"];

/// Turns `click`, `type_text=hello`, `scroll=0,120` … or a raw JSON object into a `ui_act` action.
pub(crate) fn parse_action(source: &str) -> Result<Value> {
    if source.trim_start().starts_with('{') {
        return serde_json::from_str(source).context("action is not valid JSON");
    }
    let (name, argument) = source.split_once('=').unwrap_or((source, ""));
    Ok(match name {
        name if SIMPLE_ACTIONS.contains(&name) && argument.is_empty() => json!({ "action": name }),
        "type_text" => json!({ "action": name, "text": argument }),
        "press_key" if !argument.is_empty() => json!({ "action": name, "keybind": argument }),
        "scroll" => {
            let (x, y) = parse_pair(argument)?;
            json!({ "action": name, "delta": { "x": x, "y": y } })
        }
        "drag" => {
            let (x, y) = parse_pair(argument)?;
            json!({ "action": name, "to": { "x": x, "y": y } })
        }
        _ => bail!(
            "unknown action `{source}`; use click, double_click, hover, focus, type_text=…, press_key=…, scroll=x,y, drag=x,y or a JSON object"
        ),
    })
}

fn parse_pair(source: &str) -> Result<(f64, f64)> {
    let (x, y) = source
        .split_once(',')
        .with_context(|| format!("expected `x,y`, found `{source}`"))?;
    Ok((
        x.trim()
            .parse()
            .with_context(|| format!("`{x}` is not a number"))?,
        y.trim()
            .parse()
            .with_context(|| format!("`{y}` is not a number"))?,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shorthand_actions_become_tagged_objects() -> Result<()> {
        assert_eq!(parse_action("click")?, json!({ "action": "click" }));
        assert_eq!(
            parse_action("type_text=a=b")?,
            json!({ "action": "type_text", "text": "a=b" })
        );
        assert_eq!(
            parse_action("press_key=Ctrl+A")?,
            json!({ "action": "press_key", "keybind": "Ctrl+A" })
        );
        assert_eq!(
            parse_action("scroll=0, 120")?,
            json!({ "action": "scroll", "delta": { "x": 0.0, "y": 120.0 } })
        );
        assert_eq!(
            parse_action("drag=40,300")?,
            json!({ "action": "drag", "to": { "x": 40.0, "y": 300.0 } })
        );
        Ok(())
    }

    #[test]
    fn raw_json_passes_through() -> Result<()> {
        assert_eq!(
            parse_action(r#"{"action":"hover"}"#)?,
            json!({ "action": "hover" })
        );
        Ok(())
    }

    #[test]
    fn malformed_actions_are_rejected() {
        for source in [
            "explode",
            "click=now",
            "press_key",
            "scroll=1",
            "drag=a,b",
            "{oops",
        ] {
            assert!(parse_action(source).is_err(), "{source}");
        }
    }
}
