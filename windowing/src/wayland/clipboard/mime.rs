/// Text MIME types, best first; offered when copying and matched when pasting.
pub(crate) const TEXT_MIME_TYPES: [&str; 4] = [
    "text/plain;charset=utf-8",
    "UTF8_STRING",
    "text/plain",
    "STRING",
];

/// Whether `mime_type` is one of the text types copying offers and pasting matches.
pub(crate) fn is_text_mime(mime_type: &str) -> bool {
    TEXT_MIME_TYPES.contains(&mime_type)
}

/// The first of `wanted` that `offered` holds.
pub(crate) fn first_offered<'a>(wanted: &'a [String], offered: &[String]) -> Option<&'a str> {
    wanted
        .iter()
        .find(|candidate| offered.contains(candidate))
        .map(String::as_str)
}

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

    #[test]
    fn the_first_wanted_type_on_offer_wins() {
        let wanted = ["image/png", "text/uri-list"].map(String::from);
        let offered = ["text/uri-list", "image/png"].map(String::from);
        assert_eq!(first_offered(&wanted, &offered), Some("image/png"));
        assert_eq!(
            first_offered(&wanted[1..], &offered[..1]),
            Some("text/uri-list")
        );
        assert_eq!(first_offered(&wanted, &[]), None);
    }
}
