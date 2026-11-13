use std::fmt::Write;

/// Escapes every non-ASCII character of serialized JSON as `\uXXXX`, so the Figma side can read
/// it byte by byte without a UTF-8 decoder.
pub(crate) fn to_ascii_json(json: &str) -> String {
    let mut escaped = String::with_capacity(json.len());
    for character in json.chars() {
        if character.is_ascii() {
            escaped.push(character);
        } else {
            let mut units = [0_u16; 2];
            for unit in character.encode_utf16(&mut units) {
                let _ = write!(escaped, "\\u{unit:04x}");
            }
        }
    }
    escaped
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escapes_non_ascii_including_surrogate_pairs() {
        assert_eq!(to_ascii_json("\"a·b😀\""), "\"a\\u00b7b\\ud83d\\ude00\"");
    }
}
