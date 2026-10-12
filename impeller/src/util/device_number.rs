/// Major number of a Linux `dev_t`.
pub(crate) fn major(device: u64) -> u64 {
    ((device >> 32) & 0xffff_f000) | ((device >> 8) & 0xfff)
}

/// Minor number of a Linux `dev_t`.
pub(crate) fn minor(device: u64) -> u64 {
    ((device >> 12) & 0xffff_ff00) | (device & 0xff)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_render_node_numbers() {
        let render_d128 = (226 << 8) | 128;
        assert_eq!((major(render_d128), minor(render_d128)), (226, 128));
    }

    #[test]
    fn splits_large_numbers() {
        let device = (0x1234_5000_u64 << 32) | (0xabc << 8) | (0xdef0_0000 << 12) | 0x42;
        assert_eq!(major(device), 0x1234_5abc);
        assert_eq!(minor(device), 0xdef0_0042);
    }
}
