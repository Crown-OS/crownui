use std::borrow::Cow;

/// The UI toolkit name and version reported to assistive technology.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Toolkit {
    /// Toolkit name.
    pub name: Cow<'static, str>,
    /// Toolkit version.
    pub version: Cow<'static, str>,
}

impl Toolkit {
    /// CrownUI at this crate's version.
    pub const CROWNUI: Self = Self {
        name: Cow::Borrowed("CrownUI"),
        version: Cow::Borrowed(env!("CARGO_PKG_VERSION")),
    };

    /// A toolkit with the given name and version.
    pub fn new(name: impl Into<Cow<'static, str>>, version: impl Into<Cow<'static, str>>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
        }
    }
}

impl Default for Toolkit {
    fn default() -> Self {
        Self::CROWNUI
    }
}
