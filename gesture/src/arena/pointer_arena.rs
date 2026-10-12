use input::PointerId;
use smallvec::SmallVec;

use super::{ArenaDecision, ArenaDecisions};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Phase<K> {
    Open { eager_winner: Option<K> },
    Closed,
    Decided { winner: Option<K> },
}

#[derive(Debug, Clone)]
pub(super) struct PointerArena<K> {
    pub pointer: PointerId,
    pub members: SmallVec<[K; 4]>,
    pub phase: Phase<K>,
}

impl<K: Copy + Eq> PointerArena<K> {
    pub fn new(pointer: PointerId) -> Self {
        Self {
            pointer,
            members: SmallVec::new(),
            phase: Phase::Open { eager_winner: None },
        }
    }

    pub fn reopen(&mut self) {
        self.members.clear();
        self.phase = Phase::Open { eager_winner: None };
    }

    pub fn is_decided(&self) -> bool {
        matches!(self.phase, Phase::Decided { .. })
    }

    pub fn close(&mut self) -> ArenaDecisions<K> {
        match self.phase {
            Phase::Open {
                eager_winner: Some(winner),
            } => self.decide(Some(winner)),
            Phase::Open { eager_winner: None } => {
                self.phase = Phase::Closed;
                self.decide_if_settled()
            }
            Phase::Closed | Phase::Decided { .. } => ArenaDecisions::new(),
        }
    }

    pub fn accept(&mut self, member: K) -> ArenaDecisions<K> {
        match &mut self.phase {
            Phase::Open { eager_winner } => {
                eager_winner.get_or_insert(member);
                ArenaDecisions::new()
            }
            Phase::Closed => self.decide(Some(member)),
            Phase::Decided { .. } => ArenaDecisions::new(),
        }
    }

    pub fn reject(&mut self, member: K) -> ArenaDecisions<K> {
        self.members.retain(|candidate| *candidate != member);
        let lost = ArenaDecision::Lost(member);
        match &mut self.phase {
            Phase::Open { eager_winner } => {
                if *eager_winner == Some(member) {
                    *eager_winner = None;
                }
                ArenaDecisions::from_elem(lost, 1)
            }
            Phase::Closed => {
                let mut decisions = self.decide_if_settled();
                decisions.insert(0, lost);
                decisions
            }
            Phase::Decided { .. } => ArenaDecisions::new(),
        }
    }

    pub fn sweep(&mut self) -> ArenaDecisions<K> {
        match self.phase {
            Phase::Open { eager_winner } => {
                self.decide(eager_winner.or_else(|| self.members.first().copied()))
            }
            Phase::Closed => self.decide(self.members.first().copied()),
            Phase::Decided { .. } => ArenaDecisions::new(),
        }
    }

    pub fn abandon(&mut self) -> ArenaDecisions<K> {
        if self.is_decided() {
            return ArenaDecisions::new();
        }
        self.decide(None)
    }

    fn decide_if_settled(&mut self) -> ArenaDecisions<K> {
        match self.members.as_slice() {
            [] => self.decide(None),
            [only] => self.decide(Some(*only)),
            _ => ArenaDecisions::new(),
        }
    }

    fn decide(&mut self, winner: Option<K>) -> ArenaDecisions<K> {
        let mut decisions: ArenaDecisions<K> = self
            .members
            .iter()
            .filter(|member| Some(**member) != winner)
            .map(|&member| ArenaDecision::Lost(member))
            .collect();
        decisions.extend(winner.map(ArenaDecision::Won));
        self.members.clear();
        self.phase = Phase::Decided { winner };
        decisions
    }
}
