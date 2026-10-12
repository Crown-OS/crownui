use std::collections::HashSet;

use gesture::{ArenaDecision, ArenaDecisions, GestureArena, Vote};
use input::PointerId;
use proptest::prelude::*;

#[derive(Debug, Clone, Copy)]
enum Operation {
    Add(u64, u8),
    Close(u64),
    Resolve(u64, u8, Vote),
    Sweep(u64),
    Release(u64),
}

fn vote() -> impl Strategy<Value = Vote> {
    prop_oneof![Just(Vote::Pending), Just(Vote::Accept), Just(Vote::Reject)]
}

fn operation() -> impl Strategy<Value = Operation> {
    let pointer = 0..3u64;
    let member = 0..4u8;
    prop_oneof![
        3 => (pointer.clone(), member.clone()).prop_map(|(pointer, member)| Operation::Add(pointer, member)),
        1 => pointer.clone().prop_map(Operation::Close),
        4 => (pointer.clone(), member, vote()).prop_map(|(pointer, member, vote)| Operation::Resolve(pointer, member, vote)),
        1 => pointer.clone().prop_map(Operation::Sweep),
        1 => pointer.prop_map(Operation::Release),
    ]
}

#[derive(Default)]
struct Model {
    undecided: HashSet<(u64, u8)>,
}

impl Model {
    fn has_undecided(&self, pointer: u64) -> bool {
        self.undecided.iter().any(|(owner, _)| *owner == pointer)
    }

    fn apply(&mut self, pointer: u64, decisions: &ArenaDecisions<u8>) -> Result<(), TestCaseError> {
        let mut winners = 0;
        for decision in decisions {
            let member = match *decision {
                ArenaDecision::Won(member) => {
                    winners += 1;
                    member
                }
                ArenaDecision::Lost(member) => member,
            };
            prop_assert!(
                self.undecided.remove(&(pointer, member)),
                "decision {decision:?} for pointer {pointer} has no undecided membership"
            );
        }
        prop_assert!(winners <= 1, "several winners in {decisions:?}");
        if winners == 1 {
            prop_assert!(
                !self.has_undecided(pointer),
                "members left undecided after a win"
            );
        }
        Ok(())
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(2000))]

    #[test]
    fn arena_gives_every_member_exactly_one_decision(operations in prop::collection::vec(operation(), 1..60)) {
        let mut arena = GestureArena::<u8>::new();
        let mut model = Model::default();
        for operation in operations {
            match operation {
                Operation::Add(pointer, member) => {
                    if arena.add(PointerId(pointer), member) {
                        model.undecided.insert((pointer, member));
                    }
                }
                Operation::Close(pointer) => model.apply(pointer, &arena.close(PointerId(pointer)))?,
                Operation::Resolve(pointer, member, vote) => {
                    model.apply(pointer, &arena.resolve(PointerId(pointer), member, vote))?;
                }
                Operation::Sweep(pointer) => {
                    model.apply(pointer, &arena.sweep(PointerId(pointer)))?;
                    prop_assert!(!model.has_undecided(pointer), "sweep left members undecided");
                }
                Operation::Release(pointer) => {
                    model.apply(pointer, &arena.release(PointerId(pointer)))?;
                    prop_assert!(!model.has_undecided(pointer), "release left members undecided");
                    prop_assert!(arena.release(PointerId(pointer)).is_empty());
                    prop_assert!(arena.close(PointerId(pointer)).is_empty());
                    prop_assert!(arena.sweep(PointerId(pointer)).is_empty());
                    for member in 0..4 {
                        prop_assert!(arena.resolve(PointerId(pointer), member, Vote::Accept).is_empty());
                    }
                }
            }
        }
        for pointer in 0..3 {
            model.apply(pointer, &arena.release(PointerId(pointer)))?;
        }
        prop_assert!(model.undecided.is_empty());
    }
}
