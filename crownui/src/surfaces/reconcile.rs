use std::{collections::HashMap, hash::Hash};

/// The outputs whose surfaces must close, and those that must open with their new key. An output
/// whose key changed is in both, so its surfaces are built again.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Reconciliation<Id> {
    pub(crate) close: Vec<Id>,
    pub(crate) open: Vec<(Id, u64)>,
}

/// Compares the surfaces placed on each output, by placement key, with the placements wanted now.
pub(crate) fn reconcile<Id, Surfaces>(
    placed: &HashMap<Id, (u64, Surfaces)>,
    wanted: &HashMap<Id, u64>,
) -> Reconciliation<Id>
where
    Id: Copy + Ord + Hash,
{
    let mut close: Vec<Id> = placed
        .iter()
        .filter(|(id, (key, _))| wanted.get(id) != Some(key))
        .map(|(id, _)| *id)
        .collect();
    let mut open: Vec<(Id, u64)> = wanted
        .iter()
        .filter(|(id, key)| placed.get(id).map(|(placed_key, _)| placed_key) != Some(key))
        .map(|(id, key)| (*id, *key))
        .collect();
    close.sort_unstable();
    open.sort_unstable();
    Reconciliation { close, open }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn placed(entries: &[(u32, u64)]) -> HashMap<u32, (u64, ())> {
        entries.iter().map(|&(id, key)| (id, (key, ()))).collect()
    }

    fn wanted(entries: &[(u32, u64)]) -> HashMap<u32, u64> {
        entries.iter().copied().collect()
    }

    #[test]
    fn new_outputs_open_their_surfaces() {
        assert_eq!(
            reconcile(&placed(&[]), &wanted(&[(2, 7), (1, 7)])),
            Reconciliation {
                close: vec![],
                open: vec![(1, 7), (2, 7)],
            }
        );
    }

    #[test]
    fn unchanged_placements_do_nothing() {
        assert_eq!(
            reconcile(&placed(&[(1, 7), (2, 9)]), &wanted(&[(1, 7), (2, 9)])),
            Reconciliation {
                close: vec![],
                open: vec![],
            }
        );
    }

    #[test]
    fn a_changed_key_rebuilds_only_that_output() {
        assert_eq!(
            reconcile(&placed(&[(1, 7), (2, 7)]), &wanted(&[(1, 7), (2, 8)])),
            Reconciliation {
                close: vec![2],
                open: vec![(2, 8)],
            }
        );
    }

    #[test]
    fn outputs_gone_or_no_longer_wanted_close() {
        assert_eq!(
            reconcile(&placed(&[(1, 7), (2, 7), (3, 7)]), &wanted(&[(2, 7)])),
            Reconciliation {
                close: vec![1, 3],
                open: vec![],
            }
        );
    }
}
