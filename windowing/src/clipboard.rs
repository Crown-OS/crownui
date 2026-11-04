/// Clipboard contents read in one MIME type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClipboardData {
    /// The MIME type the bytes are in.
    pub mime_type: String,
    /// The bytes the clipboard owner sent.
    pub data: Vec<u8>,
}

/// Names one clipboard read, so its answer reaches the code that asked.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClipboardReadId(pub u64);
