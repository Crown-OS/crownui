use smallvec::SmallVec;

/// The arena's verdict for one member.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ArenaDecision<K> {
    /// The member owns the pointer's gesture.
    Won(K),
    /// The member is out of this pointer's arena.
    Lost(K),
}

/// Decisions produced by one arena call, losers before the winner.
pub type ArenaDecisions<K> = SmallVec<[ArenaDecision<K>; 4]>;
