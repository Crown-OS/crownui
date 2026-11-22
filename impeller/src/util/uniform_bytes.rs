/// `floats` in native byte order, as runtime stages read their uniforms.
pub(crate) fn uniform_bytes<const BYTES: usize>(floats: &[f32]) -> [u8; BYTES] {
    let mut bytes = [0; BYTES];
    let (chunks, _) = bytes.as_chunks_mut::<{ size_of::<f32>() }>();
    for (chunk, value) in chunks.iter_mut().zip(floats) {
        *chunk = value.to_ne_bytes();
    }
    bytes
}
