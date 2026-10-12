/// One row of a [`choice_list`](super::choice_list).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Choice {
    pub(super) title: String,
    pub(super) subtitle: String,
    pub(super) icon: Option<&'static str>,
    pub(super) available: bool,
}

impl Choice {
    /// A row with a name and a line explaining it; an empty subtitle centers the title.
    pub fn new(title: impl Into<String>, subtitle: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            subtitle: subtitle.into(),
            icon: None,
            available: true,
        }
    }

    /// A leading tile showing the icon `body` from [`icons`](crate::kit::icons).
    pub fn icon(mut self, body: &'static str) -> Self {
        self.icon = Some(body);
        self
    }

    /// Keeps the row listed but unpickable, drawn faded; put the reason in the subtitle.
    pub fn disabled(mut self) -> Self {
        self.available = false;
        self
    }
}
