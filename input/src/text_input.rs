/// Input-method events for text fields, like the DOM's composition events.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TextInputEvent {
    /// Uncommitted composition text, with the cursor as a byte range inside it.
    Preedit {
        /// The composing text; empty clears the composition.
        text: String,
        /// Cursor or selection inside `text`, in bytes.
        cursor: Option<(usize, usize)>,
    },
    /// Final text to insert at the cursor.
    Commit(String),
    /// Delete bytes around the cursor before the next commit.
    DeleteSurrounding {
        /// Bytes before the cursor.
        before: usize,
        /// Bytes after the cursor.
        after: usize,
    },
}
