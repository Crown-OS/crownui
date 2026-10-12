use geometry::Rect;

/// What kind of text a field expects, so the input method can pick a fitting layout.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum TextInputPurpose {
    /// Free-form text.
    #[default]
    Normal,
    /// Letters only.
    Alpha,
    /// Digits only.
    Digits,
    /// A number, possibly with sign and decimal separator.
    Number,
    /// A phone number.
    Phone,
    /// A URL.
    Url,
    /// An email address.
    Email,
    /// A person's name.
    Name,
    /// A password; input methods must not learn or show it.
    Password,
    /// A numeric PIN; input methods must not learn or show it.
    Pin,
    /// A date.
    Date,
    /// A time of day.
    Time,
    /// A date and time.
    DateTime,
    /// A terminal emulator.
    Terminal,
}

/// The focused text field as the input method should see it.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct TextInputState {
    /// The caret or selection, in surface-local logical pixels, so candidate popups avoid it.
    pub cursor_area: Rect,
    /// Text around the caret, with the cursor and selection anchor as byte offsets into it.
    pub surrounding_text: Option<(String, usize, usize)>,
    /// The kind of text expected.
    pub content_purpose: TextInputPurpose,
}
