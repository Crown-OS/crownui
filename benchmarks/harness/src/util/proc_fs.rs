use std::fs;

/// Fields of `/proc/<pid>/stat` after the parenthesised command name, which may itself contain
/// spaces and parentheses. Index 0 is field 3 (`state`) of proc(5).
pub fn stat_fields(pid: u32) -> Option<Vec<String>> {
    let stat = fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    let after_name = &stat[stat.rfind(')')? + 1..];
    Some(after_name.split_whitespace().map(str::to_owned).collect())
}

/// The number in a `Key:   1234 kB` or `key:\t1234 KiB` line.
pub fn field_value(text: &str, key: &str) -> Option<u64> {
    text.lines().find_map(|line| {
        let value = line.strip_prefix(key)?.strip_prefix(':')?;
        value.split_whitespace().next()?.parse().ok()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_smaps_and_fdinfo_values() {
        let smaps = "Rss:              123456 kB\nPss:               65432 kB\n";
        assert_eq!(field_value(smaps, "Pss"), Some(65_432));
        let fdinfo = "drm-driver:\tamdgpu\ndrm-total-vram:\t25636 KiB\n";
        assert_eq!(field_value(fdinfo, "drm-total-vram"), Some(25_636));
        assert_eq!(field_value(fdinfo, "drm-total"), None);
    }
}
