/// The PNG chunk checksum (CRC-32/ISO-HDLC).
pub(crate) fn crc32(parts: &[&[u8]]) -> u32 {
    const POLYNOMIAL: u32 = 0xEDB8_8320;
    let mut crc = !0_u32;
    for byte in parts.iter().flat_map(|part| part.iter()) {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            crc = if crc & 1 == 1 {
                (crc >> 1) ^ POLYNOMIAL
            } else {
                crc >> 1
            };
        }
    }
    !crc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_the_reference_check_value() {
        assert_eq!(crc32(&[b"1234", b"56789"]), 0xCBF4_3926);
    }
}
