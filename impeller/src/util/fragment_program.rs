use std::borrow::Cow;

use impellers::FragmentProgram;

/// Parses `compiled`, a program `cargo xtask shaders` built with the SDK's own `impellerc`;
/// `None` (logged) when Impeller rejects it and what `name` draws falls back to MSAA.
pub(crate) fn load_fragment_program(
    compiled: &'static [u8],
    name: &str,
) -> Option<FragmentProgram> {
    // SAFETY: the bytes are impellerc output for this engine version.
    let program = unsafe { FragmentProgram::new(Cow::Borrowed(compiled)) };
    if program.is_none() {
        tracing::warn!("Impeller rejected the {name} shader; it falls back to MSAA");
    }
    program
}
