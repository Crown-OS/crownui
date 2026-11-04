/// The node name in a `default.audio.source` metadata value such as `{"name":"alsa_input.usb"}`.
pub(crate) fn default_node_name(json: &str) -> Option<&str> {
    let (_, after_key) = json.split_once("\"name\"")?;
    let (name, _) = after_key
        .trim_start()
        .strip_prefix(':')?
        .trim_start()
        .strip_prefix('"')?
        .split_once('"')?;
    Some(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_name_with_or_without_spaces() {
        assert_eq!(
            default_node_name(r#"{"name":"alsa_input.pci-0000_00_1f.3.analog-stereo"}"#),
            Some("alsa_input.pci-0000_00_1f.3.analog-stereo")
        );
        assert_eq!(default_node_name(r#"{ "name" : "mic" }"#), Some("mic"));
    }

    #[test]
    fn rejects_values_without_a_name() {
        assert_eq!(default_node_name("{}"), None);
        assert_eq!(default_node_name(r#"{"name":42}"#), None);
    }
}
