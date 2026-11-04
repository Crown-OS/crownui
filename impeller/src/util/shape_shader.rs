use impellers::{Context, FragmentProgram, Paint};

use super::{fragment_program::load_fragment_program, shape_uniforms::ShapeUniforms};

/// `rounded_box.frag` compiled by `cargo xtask shaders` with the SDK's own `impellerc`.
const ROUNDED_BOX: &[u8] = include_bytes!("../../shaders/rounded_box.iplr");

/// Parses the rounded-box program; `None` (logged) leaves rounded shapes to MSAA.
pub(crate) fn load_shape_program() -> Option<FragmentProgram> {
    load_fragment_program(ROUNDED_BOX, "rounded-box")
}

/// The rounded-box program on the context it draws with.
#[derive(Debug, Clone, Copy)]
pub(crate) struct ShapeShader<'gpu> {
    context: &'gpu Context,
    program: &'gpu FragmentProgram,
}

impl<'gpu> ShapeShader<'gpu> {
    pub(crate) fn new(context: &'gpu Context, program: &'gpu FragmentProgram) -> Self {
        Self { context, program }
    }

    pub(crate) fn paint(self, uniforms: &ShapeUniforms) -> Paint {
        // SAFETY: `ShapeUniforms` lays its floats out as the program declares its uniforms.
        let source = unsafe {
            self.context.new_color_source_from_fragment_program(
                self.program,
                &[],
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
        assert!(load_shape_program().is_some());
    }
}
