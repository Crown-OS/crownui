fn starts_with_ignoring_case(option: &str, prefix: &str) -> bool {
    option.to_lowercase().starts_with(&prefix.to_lowercase())
}

fn find_from(options: &[String], prefix: &str, start: usize) -> Option<usize> {
    let count = options.len();
    (0..count)
        .map(|offset| (start + offset) % count)
        .find(|&index| starts_with_ignoring_case(&options[index], prefix))
}

/// The option a typed `character` leads to: the first one from `current` on that starts with
/// everything typed since the buffer was last cleared, or, when nothing does, the next one that
/// starts with the character alone (so repeating a letter cycles through its options).
pub(super) fn type_ahead(
    options: &[String],
    buffer: &mut String,
    character: char,
    current: Option<usize>,
) -> Option<usize> {
    buffer.push(character);
    if let Some(found) = find_from(options, buffer, current.unwrap_or(0)) {
        return Some(found);
    }
    buffer.clear();
    buffer.push(character);
    find_from(options, buffer, current.map_or(0, |index| index + 1))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frameworks() -> Vec<String> {
        ["Next.js", "SvelteKit", "Nuxt.js", "Remix", "Astro"]
            .map(String::from)
            .to_vec()
    }

    #[test]
    fn a_letter_finds_the_first_option_starting_with_it() {
        let mut buffer = String::new();
        assert_eq!(type_ahead(&frameworks(), &mut buffer, 'r', None), Some(3));
    }

    #[test]
    fn matching_ignores_case() {
        let mut buffer = String::new();
        assert_eq!(type_ahead(&frameworks(), &mut buffer, 'S', None), Some(1));
    }

    #[test]
    fn further_letters_narrow_the_match() {
        let mut buffer = String::new();
        let options = frameworks();
        let first = type_ahead(&options, &mut buffer, 'n', None);
        assert_eq!(first, Some(0));
        assert_eq!(type_ahead(&options, &mut buffer, 'u', first), Some(2));
    }

    #[test]
    fn repeating_a_letter_cycles_through_its_options() {
        let mut buffer = String::new();
        let options = frameworks();
        let first = type_ahead(&options, &mut buffer, 'n', None);
        let second = type_ahead(&options, &mut buffer, 'n', first);
        assert_eq!((first, second), (Some(0), Some(2)));
        assert_eq!(type_ahead(&options, &mut buffer, 'n', second), Some(0));
    }

    #[test]
    fn a_letter_nothing_starts_with_finds_nothing() {
        let mut buffer = String::new();
        assert_eq!(type_ahead(&frameworks(), &mut buffer, 'z', Some(1)), None);
    }
}
