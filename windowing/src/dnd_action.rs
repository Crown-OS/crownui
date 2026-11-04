use std::ops::BitOr;

/// What a drop does with the dragged data (HTML `dropEffect`).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum DndAction {
    /// The target copies the data; the source keeps it.
    #[default]
    Copy,
    /// The target takes the data; the source removes its original once the drag ends.
    Move,
    /// The target asks the user (e.g. with a menu) what to do after the drop.
    Ask,
}

/// A set of [`DndAction`]s (HTML `effectAllowed`): what a drag source allows, or what a drop
/// target is willing to do.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DndActions(u8);

impl DndActions {
    /// No action.
    pub const NONE: Self = Self(0);
    /// Only copying.
    pub const COPY: Self = Self(1);
    /// Only moving.
    pub const MOVE: Self = Self(1 << 1);
    /// Only asking.
    pub const ASK: Self = Self(1 << 2);
    /// Copying or moving, what most drags allow.
    pub const COPY_MOVE: Self = Self(Self::COPY.0 | Self::MOVE.0);
    /// Every action.
    pub const ALL: Self = Self(Self::COPY_MOVE.0 | Self::ASK.0);

    /// Whether `action` is in the set.
    pub const fn contains(self, action: DndAction) -> bool {
        self.0 & Self::of(action).0 != 0
    }

    /// Whether the set holds no action.
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Both sets together.
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    /// The set holding only `action`.
    pub const fn of(action: DndAction) -> Self {
        match action {
            DndAction::Copy => Self::COPY,
            DndAction::Move => Self::MOVE,
            DndAction::Ask => Self::ASK,
        }
    }

    /// The actions in the set, copy first.
    pub fn iter(self) -> impl Iterator<Item = DndAction> {
        [DndAction::Copy, DndAction::Move, DndAction::Ask]
            .into_iter()
            .filter(move |action| self.contains(*action))
    }
}

impl Default for DndActions {
    fn default() -> Self {
        Self::COPY_MOVE
    }
}

impl From<DndAction> for DndActions {
    fn from(action: DndAction) -> Self {
        Self::of(action)
    }
}

impl BitOr for DndActions {
    type Output = Self;

    fn bitor(self, other: Self) -> Self {
        self.union(other)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_set_holds_exactly_the_actions_joined_into_it() {
        let set = DndActions::COPY | DndActions::ASK;
        assert!(set.contains(DndAction::Copy) && set.contains(DndAction::Ask));
        assert!(!set.contains(DndAction::Move));
        assert_eq!(
            set.iter().collect::<Vec<_>>(),
            [DndAction::Copy, DndAction::Ask]
        );
        assert!(DndActions::NONE.is_empty());
        assert_eq!(DndActions::default(), DndActions::COPY_MOVE);
    }
}
