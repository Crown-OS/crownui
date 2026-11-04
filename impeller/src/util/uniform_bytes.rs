/// `floats` in native byte order, as runtime stages read their uniforms.
pub(crate) fn uniform_bytes<const BYTES: usize>(floats: &[f32]) -> [u8; BYTES] {
    let mut bytes = [0; BYTES];
    for (chunk, value) in bytes.chunks_exact_mut(size_of::<f32>()).zip(floats) {
        chunk.copy_from_slice(&value.to_ne_bytes());
    }
    bytes
}
