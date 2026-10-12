use unicode_segmentation::UnicodeSegmentation;

pub(crate) const MASK_CHAR: char = '•';

pub(crate) fn mask(text: &str) -> String {
    text.graphemes(true).map(|_| MASK_CHAR).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const DECOMPOSED_E_ACUTE: &str = "e\u{0301}";
    const FAMILY: &str = "\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467}";

    #[test]
    fn one_bullet_hides_each_character() {
        assert_eq!(mask("hunter2"), "•••••••");
        assert_eq!(mask("pässwörd").chars().count(), 8);
        assert_eq!(mask(""), "");
    }

    #[test]
    fn one_bullet_hides_each_grapheme_cluster() {
        assert_eq!(mask(DECOMPOSED_E_ACUTE), "•");
        assert_eq!(mask(FAMILY), "•");
        assert_eq!(mask(&format!("a{FAMILY}b{DECOMPOSED_E_ACUTE}")), "••••");
    }
}
