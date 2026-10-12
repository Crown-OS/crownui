/// Where the keyboard moves a highlight in a list of rows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Step {
    Next,
    Previous,
    First,
    Last,
}

/// The row `direction` leads to from `current`, skipping rows that cannot be picked and stopping
/// at the ends of the list.
pub(crate) fn step(actionable: &[bool], current: Option<usize>, direction: Step) -> Option<usize> {
    let first = || actionable.iter().position(|&pickable| pickable);
    let last = || actionable.iter().rposition(|&pickable| pickable);
    match (direction, current) {
        (Step::First, _) | (Step::Next, None) => first(),
        (Step::Last, _) | (Step::Previous, None) => last(),
        (Step::Next, Some(from)) => actionable
            .iter()
            .enumerate()
            .skip(from + 1)
            .find_map(|(index, &pickable)| pickable.then_some(index))
            .or(Some(from)),
        (Step::Previous, Some(from)) => actionable[..from.min(actionable.len())]
            .iter()
            .rposition(|&pickable| pickable)
            .or(Some(from)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ROWS: [bool; 5] = [true, true, false, true, false];

    #[test]
    fn next_skips_rows_that_cannot_be_picked() {
        assert_eq!(step(&ROWS, Some(1), Step::Next), Some(3));
    }

    #[test]
    fn next_from_nothing_lands_on_the_first_pickable_row() {
        assert_eq!(step(&[false, true], None, Step::Next), Some(1));
    }

    #[test]
    fn the_ends_of_the_list_hold_the_highlight() {
        assert_eq!(step(&ROWS, Some(3), Step::Next), Some(3));
        assert_eq!(step(&ROWS, Some(0), Step::Previous), Some(0));
    }

    #[test]
    fn previous_from_nothing_lands_on_the_last_pickable_row() {
        assert_eq!(step(&ROWS, None, Step::Previous), Some(3));
    }

    #[test]
    fn first_and_last_ignore_the_current_row() {
        assert_eq!(step(&ROWS, Some(3), Step::First), Some(0));
        assert_eq!(step(&ROWS, Some(0), Step::Last), Some(3));
    }

    #[test]
    fn a_list_with_nothing_pickable_has_no_highlight() {
        assert_eq!(step(&[false, false], None, Step::Next), None);
    }
}
