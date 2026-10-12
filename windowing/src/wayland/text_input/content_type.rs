use sctk::reexports::protocols::wp::text_input::zv3::client::zwp_text_input_v3::{
    ContentHint, ContentPurpose,
};

use crate::TextInputPurpose;

/// The text-input-v3 content type for `purpose`; secret purposes ask the input method to
/// neither show nor learn the text.
pub(crate) fn content_type(purpose: TextInputPurpose) -> (ContentHint, ContentPurpose) {
    let secret = ContentHint::SensitiveData | ContentHint::HiddenText;
    match purpose {
        TextInputPurpose::Normal => (ContentHint::None, ContentPurpose::Normal),
        TextInputPurpose::Alpha => (ContentHint::None, ContentPurpose::Alpha),
        TextInputPurpose::Digits => (ContentHint::None, ContentPurpose::Digits),
        TextInputPurpose::Number => (ContentHint::None, ContentPurpose::Number),
        TextInputPurpose::Phone => (ContentHint::None, ContentPurpose::Phone),
        TextInputPurpose::Url => (ContentHint::None, ContentPurpose::Url),
        TextInputPurpose::Email => (ContentHint::None, ContentPurpose::Email),
        TextInputPurpose::Name => (ContentHint::AutoCapitalization, ContentPurpose::Name),
        TextInputPurpose::Password => (secret, ContentPurpose::Password),
        TextInputPurpose::Pin => (secret, ContentPurpose::Pin),
        TextInputPurpose::Date => (ContentHint::None, ContentPurpose::Date),
        TextInputPurpose::Time => (ContentHint::None, ContentPurpose::Time),
        TextInputPurpose::DateTime => (ContentHint::None, ContentPurpose::Datetime),
        TextInputPurpose::Terminal => (ContentHint::None, ContentPurpose::Terminal),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn passwords_are_hidden_from_the_input_method() {
        let (hint, purpose) = content_type(TextInputPurpose::Password);
        assert_eq!(purpose, ContentPurpose::Password);
        assert!(hint.contains(ContentHint::HiddenText | ContentHint::SensitiveData));
        assert_eq!(content_type(TextInputPurpose::Email).0, ContentHint::None);
    }
}
