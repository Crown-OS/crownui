/// One row of a search bar's suggestions: a label, and optionally a leading icon and a muted
/// detail at the trailing edge.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Suggestion {
    pub(super) label: String,
    pub(super) detail: Option<String>,
    pub(super) icon: Option<&'static str>,
}

/// A suggestion that is just its label; plain strings convert into one too.
pub fn suggestion(label: impl Into<String>) -> Suggestion {
    Suggestion {
        label: label.into(),
        detail: None,
        icon: None,
    }
}

impl Suggestion {
    /// Muted text at the row's trailing edge, shown whole beside the label or not at all.
    pub fn detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    /// A Lucide icon from [`icons`](crate::kit::icons) before the label.
    pub fn icon(mut self, body: &'static str) -> Self {
        self.icon = Some(body);
        self
    }

    /// The text the row shows, and what picking it writes into the query.
    pub fn label(&self) -> &str {
        &self.label
    }
}

impl From<&str> for Suggestion {
    fn from(label: &str) -> Self {
        suggestion(label)
    }
}

impl From<String> for Suggestion {
    fn from(label: String) -> Self {
        suggestion(label)
    }
}
