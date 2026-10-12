/// Light or dark appearance, provided at the root and read by `dark:` variants.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum ColorScheme {
    /// Light backgrounds.
    #[default]
    Light,
    /// Dark backgrounds.
    Dark,
}
