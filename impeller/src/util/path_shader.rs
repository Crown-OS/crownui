use std::slice;

use impellers::{Context, FragmentProgram, Paint, Texture};

use super::{fragment_program::load_fragment_program, path_uniforms::PathUniforms};

/// `flattened_path.frag` compiled by `cargo xtask shaders` with the SDK's own `impellerc`.
const FLATTENED_PATH: &[u8] = include_bytes!("../../shaders/flattened_path.iplr");

/// Parses the flattened-path program; `None` (logged) leaves paths to MSAA.
pub(crate) fn load_path_program() -> Option<FragmentProgram> {
    load_fragment_program(FLATTENED_PATH, "flattened-path")
}

/// The flattened-path program on the context it draws with.
#[derive(Debug, Clone, Copy)]
pub(crate) struct PathShader<'gpu> {
    context: &'gpu Context,
    program: &'gpu FragmentProgram,
}

impl<'gpu> PathShader<'gpu> {
    pub(crate) fn new(context: &'gpu Context, program: &'gpu FragmentProgram) -> Self {
        Self { context, program }
    }

    /// Shades the path whose segments `segments` holds, as `uniforms` describe it.
    pub(crate) fn paint(self, uniforms: &PathUniforms, segments: &Texture) -> Paint {
        // SAFETY: `PathUniforms` lays its floats out as the program declares its uniforms, and
        // the program samples one texture.
        let source = unsafe {
            self.context.new_color_source_from_fragment_program(
                self.program,
                slice::from_ref(segments),
                &uniforms.to_bytes(),
            )
        };
        let mut paint = Paint::default();
        paint.set_color_source(&source);
        paint
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_committed_shader_parses() {
        assert!(load_path_program().is_some());
    }
}
