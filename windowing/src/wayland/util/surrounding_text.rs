/// Cuts `text` down to at most `max_bytes` around the cursor and anchor, on character
/// boundaries, and returns the slice with both offsets made relative to it.
pub(crate) fn surrounding_window(
    text: &str,
    cursor: usize,
    anchor: usize,
    max_bytes: usize,
) -> (&str, usize, usize) {
    let cursor = text.floor_char_boundary(cursor);
    let anchor = text.floor_char_boundary(anchor);
    if text.len() <= max_bytes {
        return (text, cursor, anchor);
    }
    let anchor = if cursor.abs_diff(anchor) > max_bytes {
        cursor
    } else {
        anchor
    };
    let (low, high) = (cursor.min(anchor), cursor.max(anchor));
    let slack = (max_bytes - (high - low)) / 2;
    let end = (low.saturating_sub(slack) + max_bytes).min(text.len());
    let start = text.ceil_char_boundary(end.saturating_sub(max_bytes));
    let end = text.floor_char_boundary(end).max(start);
    let relative = |offset: usize| offset.clamp(start, end) - start;
    (&text[start..end], relative(cursor), relative(anchor))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_text_is_kept_whole() {
        assert_eq!(surrounding_window("hello", 2, 4, 16), ("hello", 2, 4));
    }

    #[test]
    fn long_text_is_cut_around_the_cursor_on_char_boundaries() {
        let text = "é".repeat(100);
        let cursor = 100;
        let (window, relative_cursor, relative_anchor) =
            surrounding_window(&text, cursor, cursor, 21);
        assert!(window.len() <= 21);
        assert_eq!(relative_cursor, relative_anchor);
        let start = window.as_ptr() as usize - text.as_ptr() as usize;
        assert_eq!(start + relative_cursor, cursor);
    }

    #[test]
    fn a_selection_wider_than_the_budget_keeps_the_cursor() {
        let text = "a".repeat(100);
        let (window, cursor, anchor) = surrounding_window(&text, 90, 10, 20);
        assert_eq!(window.len(), 20);
        assert_eq!(cursor, anchor);
        assert!(cursor <= window.len());
    }
}
