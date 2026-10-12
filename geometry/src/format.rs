use std::fmt;

/// Formats a length rounded to hundredths with trailing zeros removed, so text snapshots stay
/// stable across tiny floating-point differences.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Length(pub f32);

impl fmt::Display for Length {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let rounded = (self.0 * 100.0).round() / 100.0;
        let rounded = if rounded == 0.0 { 0.0 } else { rounded };
        write!(formatter, "{rounded}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lengths_print_without_float_noise() {
        assert_eq!(Length(40.800003).to_string(), "40.8");
        assert_eq!(Length(12.000001).to_string(), "12");
        assert_eq!(Length(-0.001).to_string(), "0");
    }
}
