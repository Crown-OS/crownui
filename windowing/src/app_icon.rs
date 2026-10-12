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
