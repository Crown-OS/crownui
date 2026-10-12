use reactive::Signal;
use view::{Element, IntoProp, Prop, Styled, View};

use super::metrics::{
    HEADER_GAP, HERO_COLUMN_WIDTH, SUBTITLE_TEXT_SIZE, TITLE_GAP, TITLE_TEXT_SIZE,
};
use crate::{
    kit::{IntoStep, StepBarKind, Theme, step_bar},
    stack::vstack,
    text::text,
    util::themed::themed,
};

/// The right column's heading: a title, an optional explanation and an optional step bar.
#[must_use]
pub struct HeroStep {
    title: Prop<String>,
    subtitle: Option<Prop<String>>,
    steps: Option<Element<StepBarKind>>,
}

impl HeroStep {
    /// A heading with nothing under it.
    pub fn new(title: impl IntoProp<String>) -> Self {
        Self {
            title: title.into_prop(),
            subtitle: None,
            steps: None,
        }
    }

    /// A line or two under the heading saying what to do; it wraps at the column width.
    pub fn subtitle(mut self, subtitle: impl IntoProp<String>) -> Self {
        self.subtitle = Some(subtitle.into_prop());
        self
    }

    /// A step bar of `total` steps with the zero-based `current` reached.
    pub fn steps(mut self, total: usize, current: impl IntoStep) -> Self {
        self.steps = Some(step_bar(total, current));
        self
    }

    pub(super) fn header(self, theme: Signal<Theme>) -> impl View + Styled {
        let title = text(self.title)
            .text_size(TITLE_TEXT_SIZE)
            .text_color(themed(theme, |theme| theme.text.primary));
        let subtitle = self.subtitle.map(|subtitle| {
            text(subtitle)
                .text_size(SUBTITLE_TEXT_SIZE)
                .text_color(themed(theme, |theme| theme.text.muted))
        });
        let bar = self
            .steps
            .map(|bar| bar.w(HERO_COLUMN_WIDTH).mt(HEADER_GAP));
        vstack((title, subtitle, bar))
            .gap(TITLE_GAP)
            .items_start()
            .shrink(0.0)
    }
}
