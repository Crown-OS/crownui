pub(crate) fn current_uid() -> u32 {
    rustix::process::getuid().as_raw()
}
