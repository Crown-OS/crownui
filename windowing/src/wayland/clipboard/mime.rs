/// Text MIME types, best first; offered when copying and matched when pasting.
pub(crate) const TEXT_MIME_TYPES: [&str; 4] = [
    "text/plain;charset=utf-8",
    "UTF8_STRING",
    "text/plain",
    "STRING",
];

/// The best text MIME type among those a selection offers.
pub(crate) fn preferred_text_mime(offered: &[String]) -> Option<&'static str> {
    TEXT_MIME_TYPES
        .into_iter()
        .find(|candidate| offered.iter().any(|mime| mime == candidate))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utf8_text_wins_over_legacy_types() {
        let offered = ["STRING", "text/plain;charset=utf-8", "image/png"].map(String::from);
        assert_eq!(
            preferred_text_mime(&offered),
            Some("text/plain;charset=utf-8")
        );
        assert_eq!(preferred_text_mime(&["image/png".to_owned()]), None);
    }
}
