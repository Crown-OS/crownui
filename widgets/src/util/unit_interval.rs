/// `value` clamped to `0..=1`, with NaN read as silence.
pub(crate) fn clamp_unit(value: f32) -> f32 {
    if value.is_nan() {
        0.0
    } else {
        value.clamp(0.0, 1.0)
    }
}

/// A `0..=1` level packed into a byte.
pub(crate) fn unit_to_byte(level: f32) -> u8 {
    (clamp_unit(level) * f32::from(u8::MAX)).round() as u8
}

/// A byte unpacked into a `0..=1` level.
pub(crate) fn byte_to_unit(byte: u8) -> f32 {
    f32::from(byte) / f32::from(u8::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn levels_survive_a_round_trip_through_a_byte_and_nonsense_clamps() {
        assert_eq!(byte_to_unit(unit_to_byte(1.0)), 1.0);
        assert_eq!(unit_to_byte(0.5), 128);
        assert_eq!(unit_to_byte(f32::NAN), 0);
        assert_eq!(unit_to_byte(7.0), u8::MAX);
        assert_eq!(clamp_unit(-1.0), 0.0);
    }
}
