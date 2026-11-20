use std::ops::BitOr;

/// The shell surfaces the compositor wants on an output, with the user's defaults applied.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct ShellSurfaces(u8);

impl ShellSurfaces {
    /// No shell surface, as on a hidden output.
    pub const NONE: Self = Self(0);
    /// The bar.
    pub const BAR: Self = Self(1);
    /// The dock.
    pub const DOCK: Self = Self(2);
    /// Notification popups.
    pub const NOTIFICATIONS: Self = Self(4);

    const ALL_BITS: u32 = 7;

    pub(crate) fn from_protocol_bits(bits: u32) -> Self {
        Self((bits & Self::ALL_BITS) as u8)
    }

    /// Whether every surface in `other` is wanted.
    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    /// Whether the output shows the bar.
    pub const fn bar(self) -> bool {
        self.contains(Self::BAR)
    }

    /// Whether the output shows the dock.
    pub const fn dock(self) -> bool {
        self.contains(Self::DOCK)
    }

    /// Whether the output shows notification popups.
    pub const fn notifications(self) -> bool {
        self.contains(Self::NOTIFICATIONS)
    }
}

impl BitOr for ShellSurfaces {
    type Output = Self;

    fn bitor(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protocol_bits_map_to_their_surfaces() {
        let surfaces = ShellSurfaces::from_protocol_bits(5);
        assert!(surfaces.bar());
        assert!(!surfaces.dock());
        assert!(surfaces.notifications());
        assert_eq!(surfaces, ShellSurfaces::BAR | ShellSurfaces::NOTIFICATIONS);
    }

    #[test]
    fn bits_from_a_newer_compositor_are_dropped() {
        assert_eq!(
            ShellSurfaces::from_protocol_bits(8 | 2),
            ShellSurfaces::DOCK
        );
    }
}
