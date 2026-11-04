/// One folder of the path a [`path_bar`](super::path_bar) shows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathSegment {
    /// The folder's name.
    pub label: String,
    /// A Lucide icon from [`icons`](crate::kit::icons) before the name, e.g. a house for home.
    pub icon: Option<&'static str>,
}

impl PathSegment {
    /// A segment showing `label`.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            icon: None,
        }
    }

    /// The same segment with a leading icon.
    pub fn icon(self, body: &'static str) -> Self {
        Self {
            icon: Some(body),
            ..self
        }
    }
}
