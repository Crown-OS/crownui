pub(crate) const fn join_u64(high: u32, low: u32) -> u64 {
    ((high as u64) << 32) | low as u64
}

pub(crate) const fn split_u64(value: u64) -> (u32, u32) {
    ((value >> 32) as u32, value as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splitting_and_joining_round_trips() {
        let value = 0x0123_4567_89ab_cdef;
        let (high, low) = split_u64(value);
        assert_eq!((high, low), (0x0123_4567, 0x89ab_cdef));
        assert_eq!(join_u64(high, low), value);
    }
}
