use std::io::Write;

/// Ends a capture run with exit status 0, skipping process teardown: NVIDIA's EGL `atexit`
/// handler crashes on the headless contexts captures keep alive. Call it last, after
/// [`crate::Carrier::write`].
pub fn finish() -> ! {
    let _ = std::io::stdout().flush();
    let _ = std::io::stderr().flush();
    // SAFETY: `_exit` only ends the process; every file was written and flushed above.
    unsafe { libc::_exit(0) }
}
