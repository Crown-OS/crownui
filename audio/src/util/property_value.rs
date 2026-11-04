use std::borrow::Cow;

/// `text` without the NUL bytes a PipeWire property value cannot hold.
pub(crate) fn property_value(text: &str) -> Cow<'_, str> {
    if text.contains('\0') {
        Cow::Owned(text.replace('\0', ""))
    } else {
        Cow::Borrowed(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_values_are_borrowed_and_nuls_are_dropped() {
        assert!(matches!(
            property_value("alsa_input"),
            Cow::Borrowed("alsa_input")
        ));
        assert_eq!(property_value("al\0sa"), "alsa");
    }
}
