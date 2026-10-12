/// One segment of a [`toggle_group`](super::toggle_group): a text label, or a Lucide icon whose
/// label is only spoken.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToggleItem {
    pub(super) label: String,
    pub(super) icon: Option<&'static str>,
}

impl ToggleItem {
    /// A segment showing `label`.
    pub fn text(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            icon: None,
        }
    }

    /// A segment showing the icon `body` from [`icons`](crate::kit::icons), named `label` for
    /// assistive technology.
    pub fn icon(body: &'static str, label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            icon: Some(body),
        }
    }
}
