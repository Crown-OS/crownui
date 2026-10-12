use std::ops::Range;

use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Selection {
    pub(crate) anchor: usize,
    pub(crate) caret: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Motion {
    PreviousGrapheme,
    NextGrapheme,
    PreviousWord,
    NextWord,
    Start,
    End,
}

impl Motion {
    pub(crate) fn target(self, text: &str, from: usize) -> usize {
        match self {
            Self::PreviousGrapheme => text[..from]
                .grapheme_indices(true)
                .next_back()
                .map_or(0, |(start, _)| start),
            Self::NextGrapheme => text[from..]
                .graphemes(true)
                .next()
                .map_or(from, |grapheme| from + grapheme.len()),
            Self::PreviousWord => text
                .unicode_word_indices()
                .map(|(start, _)| start)
                .take_while(|start| *start < from)
                .last()
                .unwrap_or(0),
            Self::NextWord => text
                .unicode_word_indices()
                .map(|(start, word)| start + word.len())
                .find(|end| *end > from)
                .unwrap_or(text.len()),
            Self::Start => 0,
            Self::End => text.len(),
        }
    }

    const fn collapses_selection(self) -> bool {
        matches!(self, Self::PreviousGrapheme | Self::NextGrapheme)
    }

    const fn is_backward(self) -> bool {
        matches!(
            self,
            Self::PreviousGrapheme | Self::PreviousWord | Self::Start
        )
    }
}

impl Selection {
    pub(crate) const fn at(index: usize) -> Self {
        Self {
            anchor: index,
            caret: index,
        }
    }

    pub(crate) fn range(self) -> Range<usize> {
        self.anchor.min(self.caret)..self.anchor.max(self.caret)
    }

    pub(crate) const fn is_collapsed(self) -> bool {
        self.anchor == self.caret
    }

    pub(crate) fn all(text: &str) -> Self {
        Self {
            anchor: 0,
            caret: text.len(),
        }
    }

    pub(crate) fn clamped(self, text: &str) -> Self {
        let snap = |index: usize| {
            (0..=index.min(text.len()))
                .rev()
                .find(|candidate| text.is_char_boundary(*candidate))
                .unwrap_or(0)
        };
        Self {
            anchor: snap(self.anchor),
            caret: snap(self.caret),
        }
    }

    pub(crate) fn after(self, text: &str, motion: Motion, extend: bool) -> Self {
        if extend {
            return Self {
                caret: motion.target(text, self.caret),
                ..self
            };
        }
        let range = self.range();
        match (self.is_collapsed(), motion.collapses_selection()) {
            (false, true) if motion.is_backward() => Self::at(range.start),
            (false, true) => Self::at(range.end),
            _ => Self::at(motion.target(text, self.caret)),
        }
    }

    pub(crate) fn surrounding(self, text: &str, before: usize, after: usize) -> Self {
        let range = self.range();
        let start = (0..=range.start.saturating_sub(before))
            .rev()
            .find(|index| text.is_char_boundary(*index))
            .unwrap_or(0);
        let end = ((range.end + after).min(text.len())..=text.len())
            .find(|index| text.is_char_boundary(*index))
            .unwrap_or(text.len());
        Self {
            anchor: start,
            caret: end,
        }
    }

    pub(crate) fn replace(self, text: &mut String, inserted: &str) -> Option<Self> {
        let range = self.range();
        if range.is_empty() && inserted.is_empty() {
            return None;
        }
        let caret = range.start + inserted.len();
        text.replace_range(range, inserted);
        Some(Self::at(caret))
    }

    pub(crate) fn delete(self, text: &mut String, motion: Motion) -> Option<Self> {
        let reach = if self.is_collapsed() {
            Self {
                anchor: self.caret,
                caret: motion.target(text, self.caret),
            }
        } else {
            self
        };
        reach.replace(text, "")
    }
}

pub(crate) fn boundary_at_fraction(text: &str, fraction: f32) -> usize {
    let graphemes = text.graphemes(true).count();
    let target = (fraction.clamp(0.0, 1.0) * graphemes as f32).round() as usize;
    text.grapheme_indices(true)
        .nth(target)
        .map_or(text.len(), |(start, _)| start)
}

pub(crate) fn word_range_at(text: &str, index: usize) -> Range<usize> {
    let mut segments = text
        .split_word_bound_indices()
        .map(|(start, word)| start..start + word.len());
    segments
        .clone()
        .find(|segment| index < segment.end)
        .or_else(|| segments.next_back())
        .unwrap_or(0..0)
}

pub(crate) fn single_line(text: &str) -> String {
    text.chars()
        .filter(|character| !character.is_control())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const DECOMPOSED_E_ACUTE: &str = "e\u{0301}";

    #[test]
    fn grapheme_motions_never_split_a_cluster() {
        let text = format!("a{DECOMPOSED_E_ACUTE}b");
        assert_eq!(
            Motion::NextGrapheme.target(&text, 1),
            1 + DECOMPOSED_E_ACUTE.len()
        );
        assert_eq!(
            Motion::PreviousGrapheme.target(&text, 1 + DECOMPOSED_E_ACUTE.len()),
            1
        );
        assert_eq!(Motion::PreviousGrapheme.target(&text, 0), 0);
        assert_eq!(Motion::NextGrapheme.target(&text, text.len()), text.len());
    }

    #[test]
    fn word_motions_land_on_word_edges() {
        let text = "one two  three";
        assert_eq!(Motion::NextWord.target(text, 0), 3);
        assert_eq!(Motion::NextWord.target(text, 3), 7);
        assert_eq!(Motion::NextWord.target(text, 14), 14);
        assert_eq!(Motion::PreviousWord.target(text, 14), 9);
        assert_eq!(Motion::PreviousWord.target(text, 9), 4);
        assert_eq!(Motion::PreviousWord.target(text, 1), 0);
    }

    #[test]
    fn arrows_collapse_a_selection_and_shift_extends_it() {
        let text = "hello";
        let selected = Selection {
            anchor: 1,
            caret: 4,
        };
        assert_eq!(
            selected.after(text, Motion::PreviousGrapheme, false),
            Selection::at(1)
        );
        assert_eq!(
            selected.after(text, Motion::NextGrapheme, false),
            Selection::at(4)
        );
        assert_eq!(
            selected.after(text, Motion::NextGrapheme, true),
            Selection {
                anchor: 1,
                caret: 5
            }
        );
        assert_eq!(selected.after(text, Motion::Start, false), Selection::at(0));
    }

    #[test]
    fn replacing_swaps_the_selection_for_the_inserted_text() {
        let mut text = String::from("hello world");
        let caret = Selection {
            anchor: 6,
            caret: 11,
        }
        .replace(&mut text, "there");
        assert_eq!(text, "hello there");
        assert_eq!(caret, Some(Selection::at(11)));
        assert_eq!(Selection::at(3).replace(&mut text, ""), None);
    }

    #[test]
    fn deleting_removes_the_selection_or_one_step() {
        let mut text = String::from("hello world");
        assert_eq!(
            Selection::at(11).delete(&mut text, Motion::PreviousWord),
            Some(Selection::at(6))
        );
        assert_eq!(text, "hello ");
        assert_eq!(
            Selection {
                anchor: 0,
                caret: 2
            }
            .delete(&mut text, Motion::NextGrapheme),
            Some(Selection::at(0))
        );
        assert_eq!(text, "llo ");
        assert_eq!(
            Selection::at(0).delete(&mut text, Motion::PreviousGrapheme),
            None
        );
    }

    #[test]
    fn surrounding_bytes_widen_to_character_boundaries() {
        let text = "aéb";
        assert_eq!(
            Selection::at(3).surrounding(text, 1, 0),
            Selection {
                anchor: 1,
                caret: 3
            }
        );
        assert_eq!(
            Selection::at(1).surrounding(text, 0, 1),
            Selection {
                anchor: 1,
                caret: 3
            }
        );
    }

    #[test]
    fn a_stale_selection_is_clamped_onto_the_text() {
        let text = "héllo";
        let stale = Selection {
            anchor: 2,
            caret: 99,
        };
        assert_eq!(
            stale.clamped(text),
            Selection {
                anchor: 1,
                caret: text.len()
            }
        );
    }

    #[test]
    fn a_fraction_picks_the_nearest_grapheme_boundary() {
        assert_eq!(boundary_at_fraction("abcd", 0.0), 0);
        assert_eq!(boundary_at_fraction("abcd", 0.5), 2);
        assert_eq!(boundary_at_fraction("abcd", 2.0), 4);
        assert_eq!(boundary_at_fraction("", 0.5), 0);
    }

    #[test]
    fn a_double_click_selects_the_word_under_it() {
        assert_eq!(word_range_at("one two", 5), 4..7);
        assert_eq!(word_range_at("one two", 7), 4..7);
        assert_eq!(word_range_at("", 0), 0..0);
    }

    #[test]
    fn pasted_line_breaks_are_dropped() {
        assert_eq!(single_line("a\nb\tc"), "abc");
    }
}
