/// How many leading entries to fold into the ellipsis so the rest fit in `available`: none when
/// everything fits, never the last one.
pub(super) fn leading_to_fold(widths: &[f32], available: f32, gap: f32, ellipsis: f32) -> usize {
    let span = |entries: &[f32]| {
        entries.iter().sum::<f32>() + gap * entries.len().saturating_sub(1) as f32
    };
    if widths.is_empty() || span(widths) <= available {
        return 0;
    }
    let last = widths.len() - 1;
    (1..last)
        .find(|&folded| ellipsis + gap + span(&widths[folded..]) <= available)
        .unwrap_or(last)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_path_that_fits_folds_nothing() {
        assert_eq!(leading_to_fold(&[40.0, 60.0, 80.0], 184.0, 2.0, 26.0), 0);
        assert_eq!(leading_to_fold(&[], 0.0, 2.0, 26.0), 0);
    }

    #[test]
    fn leading_entries_fold_until_the_rest_fits_beside_the_ellipsis() {
        let widths = [40.0, 60.0, 80.0, 50.0];
        assert_eq!(leading_to_fold(&widths, 200.0, 2.0, 26.0), 2);
        assert_eq!(leading_to_fold(&widths, 160.0, 2.0, 26.0), 2);
        assert_eq!(leading_to_fold(&widths, 100.0, 2.0, 26.0), 3);
    }

    #[test]
    fn the_last_entry_always_stays() {
        assert_eq!(leading_to_fold(&[40.0, 400.0], 50.0, 2.0, 26.0), 1);
    }
}
