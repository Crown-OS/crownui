/// The value below which `fraction` of `values` fall (nearest rank).
pub fn percentile(values: &[f64], fraction: f64) -> Option<f64> {
    let mut sorted: Vec<f64> = values
        .iter()
        .copied()
        .filter(|value| value.is_finite())
        .collect();
    if sorted.is_empty() {
        return None;
    }
    sorted.sort_by(f64::total_cmp);
    let rank = (fraction * sorted.len() as f64).ceil() as usize;
    sorted.get(rank.saturating_sub(1)).copied()
}

pub fn median(values: &[f64]) -> Option<f64> {
    percentile(values, 0.5)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nearest_rank_percentiles() {
        let values = [5.0, 1.0, 4.0, 2.0, 3.0];
        assert_eq!(median(&values), Some(3.0));
        assert_eq!(percentile(&values, 0.9), Some(5.0));
        assert_eq!(percentile(&values, 0.2), Some(1.0));
    }

    #[test]
    fn empty_and_non_finite_values_have_no_percentile() {
        assert_eq!(median(&[]), None);
        assert_eq!(median(&[f64::NAN]), None);
    }
}
