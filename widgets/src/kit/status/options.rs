use view::Element;

use super::kind::StatusKind;
use crate::kit::Tone;

/// The options of a [`status`](super::status), chained before its modifiers.
pub trait StatusOptions: Sized {
    /// What the line is saying; the default is [`Tone::Neutral`].
    fn tone(self, tone: Tone) -> Self;
}

impl StatusOptions for Element<StatusKind> {
    fn tone(self, tone: Tone) -> Self {
        self.configure(|kind| kind.tone = tone)
    }
}
