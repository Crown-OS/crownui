#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Step {
    Previous,
    Next,
}

pub(super) fn step(selected: Option<usize>, available: &[bool], direction: Step) -> Option<usize> {
    let count = available.len();
    let start = match (direction, selected) {
        (Step::Next, Some(index)) => index + 1,
        (Step::Next, None) => 0,
        (Step::Previous, Some(index)) => index + count - 1,
        (Step::Previous, None) => count.checked_sub(1)?,
    };
    (0..count)
        .map(|offset| match direction {
            Step::Next => (start + offset) % count,
            Step::Previous => (start + count - offset) % count,
        })
        .find(|index| available[*index])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stepping_skips_unavailable_rows_and_wraps() {
        let available = [true, false, true];
        assert_eq!(step(Some(0), &available, Step::Next), Some(2));
        assert_eq!(step(Some(2), &available, Step::Next), Some(0));
        assert_eq!(step(Some(2), &available, Step::Previous), Some(0));
        assert_eq!(step(Some(0), &available, Step::Previous), Some(2));
    }

    #[test]
    fn without_a_selection_next_picks_the_first_and_previous_the_last_available() {
        let available = [false, true, true];
        assert_eq!(step(None, &available, Step::Next), Some(1));
        assert_eq!(step(None, &available, Step::Previous), Some(2));
    }

    #[test]
    fn nothing_is_pickable_in_an_empty_or_fully_unavailable_list() {
        assert_eq!(step(None, &[], Step::Next), None);
        assert_eq!(step(Some(0), &[false, false], Step::Next), None);
    }
}
