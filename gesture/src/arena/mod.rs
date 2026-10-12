mod decision;
mod pointer_arena;

use input::PointerId;
use smallvec::SmallVec;

use crate::Vote;

pub use decision::{ArenaDecision, ArenaDecisions};
use pointer_arena::{Phase, PointerArena};

/// Settles which member owns each pointer's gesture, with Flutter's gesture-arena rules.
///
/// Members (recognizers, identified by `K`) join a pointer's arena on pointer down; the caller
/// closes it once the down event reached every member. The first member to [`Vote::Accept`] wins
/// (an accept before closing wins at close), a member that rejects loses at once, and when a
/// closed arena has a single member left that member wins. [`sweep`](Self::sweep) on pointer up
/// hands an undecided arena to the member added first, which by convention is the deepest hit
/// target. Every member receives exactly one decision per arena.
#[derive(Debug, Clone)]
pub struct GestureArena<K> {
    arenas: SmallVec<[PointerArena<K>; 2]>,
}

impl<K> Default for GestureArena<K> {
    fn default() -> Self {
        Self {
            arenas: SmallVec::new(),
        }
    }
}

impl<K: Copy + Eq> GestureArena<K> {
    /// No arenas.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds `member` to `pointer`'s arena, opening a fresh arena when there is none or the old one
    /// is decided. Returns false, without adding, when the arena is already closed.
    pub fn add(&mut self, pointer: PointerId, member: K) -> bool {
        let arena = match self.position(pointer) {
            Some(index) => &mut self.arenas[index],
            None => {
                self.arenas.push(PointerArena::new(pointer));
                let last = self.arenas.len() - 1;
                &mut self.arenas[last]
            }
        };
        match arena.phase {
            Phase::Closed => return false,
            Phase::Decided { .. } => arena.reopen(),
            Phase::Open { .. } => {}
        }
        if !arena.members.contains(&member) {
            arena.members.push(member);
        }
        true
    }

    /// Closes `pointer`'s arena to new members once the down event was dispatched.
    #[must_use]
    pub fn close(&mut self, pointer: PointerId) -> ArenaDecisions<K> {
        self.arena_mut(pointer)
            .map_or_else(ArenaDecisions::new, PointerArena::close)
    }

    /// Applies `member`'s `vote` in `pointer`'s arena; ignored for non-members and decided arenas.
    #[must_use]
    pub fn resolve(&mut self, pointer: PointerId, member: K, vote: Vote) -> ArenaDecisions<K> {
        let Some(arena) = self
            .arena_mut(pointer)
            .filter(|arena| arena.members.contains(&member))
        else {
            return ArenaDecisions::new();
        };
        match vote {
            Vote::Pending => ArenaDecisions::new(),
            Vote::Accept => arena.accept(member),
            Vote::Reject => arena.reject(member),
        }
    }

    /// Decides `pointer`'s arena on pointer up: an undecided arena goes to its first member.
    #[must_use]
    pub fn sweep(&mut self, pointer: PointerId) -> ArenaDecisions<K> {
        self.arena_mut(pointer)
            .map_or_else(ArenaDecisions::new, PointerArena::sweep)
    }

    /// Forgets `pointer`'s arena; members of an undecided arena all lose.
    #[must_use]
    pub fn release(&mut self, pointer: PointerId) -> ArenaDecisions<K> {
        self.position(pointer)
            .map_or_else(ArenaDecisions::new, |index| {
                self.arenas.swap_remove(index).abandon()
            })
    }

    /// The member that won `pointer`'s arena, if it is decided in someone's favour.
    pub fn winner(&self, pointer: PointerId) -> Option<K> {
        self.arenas
            .iter()
            .find(|arena| arena.pointer == pointer)
            .and_then(|arena| match arena.phase {
                Phase::Decided { winner } => winner,
                Phase::Open { .. } | Phase::Closed => None,
            })
    }

    fn position(&self, pointer: PointerId) -> Option<usize> {
        self.arenas
            .iter()
            .position(|arena| arena.pointer == pointer)
    }

    fn arena_mut(&mut self, pointer: PointerId) -> Option<&mut PointerArena<K>> {
        self.arenas
            .iter_mut()
            .find(|arena| arena.pointer == pointer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MOUSE: PointerId = PointerId::MOUSE;
    const FINGER: PointerId = PointerId(7);

    fn arena_with(pointer: PointerId, members: &[u8]) -> GestureArena<u8> {
        let mut arena = GestureArena::new();
        for &member in members {
            assert!(arena.add(pointer, member));
        }
        arena
    }

    #[test]
    fn accepting_member_wins_and_others_lose() {
        let mut arena = arena_with(MOUSE, &[1, 2, 3]);
        assert!(arena.close(MOUSE).is_empty());
        assert_eq!(
            arena.resolve(MOUSE, 2, Vote::Accept).as_slice(),
            [
                ArenaDecision::Lost(1),
                ArenaDecision::Lost(3),
                ArenaDecision::Won(2)
            ]
        );
        assert_eq!(arena.winner(MOUSE), Some(2));
        assert!(arena.resolve(MOUSE, 1, Vote::Accept).is_empty());
    }

    #[test]
    fn accept_before_close_wins_at_close() {
        let mut arena = arena_with(MOUSE, &[1, 2]);
        assert!(arena.resolve(MOUSE, 2, Vote::Accept).is_empty());
        assert!(arena.resolve(MOUSE, 1, Vote::Accept).is_empty());
        assert_eq!(
            arena.close(MOUSE).as_slice(),
            [ArenaDecision::Lost(1), ArenaDecision::Won(2)]
        );
    }

    #[test]
    fn last_survivor_of_a_closed_arena_wins() {
        let mut arena = arena_with(MOUSE, &[1, 2, 3]);
        assert_eq!(
            arena.resolve(MOUSE, 1, Vote::Reject).as_slice(),
            [ArenaDecision::Lost(1)]
        );
        assert!(arena.close(MOUSE).is_empty());
        assert_eq!(
            arena.resolve(MOUSE, 3, Vote::Reject).as_slice(),
            [ArenaDecision::Lost(3), ArenaDecision::Won(2)]
        );
    }

    #[test]
    fn closing_with_a_single_member_decides_at_once() {
        let mut arena = arena_with(MOUSE, &[4]);
        assert_eq!(arena.close(MOUSE).as_slice(), [ArenaDecision::Won(4)]);
    }

    #[test]
    fn sweep_picks_the_first_added_member() {
        let mut arena = arena_with(MOUSE, &[5, 6, 7]);
        assert!(arena.close(MOUSE).is_empty());
        assert_eq!(
            arena.sweep(MOUSE).as_slice(),
            [
                ArenaDecision::Lost(6),
                ArenaDecision::Lost(7),
                ArenaDecision::Won(5)
            ]
        );
        assert!(arena.release(MOUSE).is_empty());
    }

    #[test]
    fn closed_arena_refuses_members_and_decided_arena_reopens() {
        let mut arena = arena_with(MOUSE, &[1, 2]);
        assert!(arena.close(MOUSE).is_empty());
        assert!(!arena.add(MOUSE, 3));
        let _ = arena.sweep(MOUSE);
        assert!(arena.add(MOUSE, 3));
        assert_eq!(arena.close(MOUSE).as_slice(), [ArenaDecision::Won(3)]);
    }

    #[test]
    fn releasing_an_undecided_arena_makes_everyone_lose() {
        let mut arena = arena_with(MOUSE, &[1, 2]);
        assert_eq!(
            arena.release(MOUSE).as_slice(),
            [ArenaDecision::Lost(1), ArenaDecision::Lost(2)]
        );
        assert!(arena.resolve(MOUSE, 1, Vote::Accept).is_empty());
    }

    #[test]
    fn pointers_have_independent_arenas() {
        let mut arena = arena_with(MOUSE, &[1, 2]);
        assert!(arena.add(FINGER, 1));
        assert!(arena.add(FINGER, 3));
        assert!(arena.close(MOUSE).is_empty());
        assert!(arena.close(FINGER).is_empty());
        assert_eq!(
            arena.resolve(FINGER, 1, Vote::Accept).as_slice(),
            [ArenaDecision::Lost(3), ArenaDecision::Won(1)]
        );
        assert_eq!(arena.winner(MOUSE), None);
        assert_eq!(
            arena.resolve(MOUSE, 2, Vote::Accept).as_slice(),
            [ArenaDecision::Lost(1), ArenaDecision::Won(2)]
        );
    }
}
