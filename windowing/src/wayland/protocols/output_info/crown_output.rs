use super::ShellSurfaces;

/// How the CrownOS compositor treats an output, from `crownos_output_info_v1`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CrownOutput {
    /// The name the settings use for the monitor, exactly as the compositor sent it. Look a
    /// settings entry up by the `wl_output` name first and by this name second.
    pub identity: String,
    /// The shell surfaces the output shows.
    pub shell: ShellSurfaces,
    /// Whether this is the primary output; exactly one output is.
    pub primary: bool,
    /// Whether the pointer is on this output.
    pub focused: bool,
    /// Whether the display claims HDR (a PQ or HLG transfer function).
    pub hdr_capable: bool,
    /// Whether no display is behind the output, as for a remote screen.
    pub virtual_output: bool,
    /// The bits per color channel the output is scanned out at.
    pub scanout_bits: u32,
}
