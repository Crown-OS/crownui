/// A state name made safe as a file name: `Request · Dark` → `request-dark`.
pub(crate) fn file_name(name: &str) -> String {
    let mut safe = String::with_capacity(name.len());
    for character in name.chars() {
        if character.is_ascii_alphanumeric() {
            safe.push(character.to_ascii_lowercase());
        } else if !safe.ends_with('-') {
            safe.push('-');
        }
    }
    safe.trim_matches('-').to_owned()
}
