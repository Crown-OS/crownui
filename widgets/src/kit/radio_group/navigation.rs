#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Step {
    Previous,
    Next,
}

pub(super) fn step(selected: Option<usize>, count: usize, direction: Step) -> Option<usize> {
    let last = count.checked_sub(1)?;
    Some(match (direction, selected) {
        (Step::Next, None) => 0,
        (Step::Next, Some(index)) if index >= last => 0,
        (Step::Next, Some(index)) => index + 1,
        (Step::Previous, None | Some(0)) => last,
        (Step::Previous, Some(index)) => (index - 1).min(last),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stepping_wraps_around_both_ends() {
        assert_eq!(step(Some(2), 3, Step::Next), Some(0));
        assert_eq!(step(Some(0), 3, Step::Previous), Some(2));
        assert_eq!(step(Some(1), 3, Step::Next), Some(2));
        assert_eq!(step(Some(1), 3, Step::Previous), Some(0));
    }

    #[test]
    fn without_a_selection_next_starts_at_the_top_and_previous_at_the_bottom() {
        assert_eq!(step(None, 3, Step::Next), Some(0));
        assert_eq!(step(None, 3, Step::Previous), Some(2));
    }

    #[test]
    fn an_empty_group_has_nowhere_to_step() {
        assert_eq!(step(None, 0, Step::Next), None);
    }
}
