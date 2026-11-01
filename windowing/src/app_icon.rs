use geometry::Rect;

/// Where a launcher shows an application's icon, in window-local logical pixels, when the window
/// is fully shown. The compositor grows the application's windows out of it and shrinks them back
/// into it.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct AppIconPlacement {
    /// The icon's bounds.
    pub rect: Rect,
    /// The radius of the icon's rounded corners.
    pub corner_radius: f32,
}

/// How many windows an application has and whether one of them is focused, as the compositor
/// reports it to launchers.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AppWindows {
    /// The application's id, compared without regard to ASCII case.
    pub app_id: String,
    /// Its windows, minimized ones included; 0 when it has none left.
    pub count: u32,
    /// Whether one of its windows has keyboard focus.
    pub focused: bool,
}
