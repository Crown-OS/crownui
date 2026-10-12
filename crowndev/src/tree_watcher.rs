/// Remembers the last tree printed so `watch` only reprints changes.
#[derive(Debug, Default)]
pub(crate) struct TreeWatcher {
    last: Option<String>,
}

impl TreeWatcher {
    /// The tree when it differs from the previous one.
    pub(crate) fn observe(&mut self, tree: String) -> Option<&str> {
        if self.last.as_ref() == Some(&tree) {
            return None;
        }
        Some(self.last.insert(tree).as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_changes_are_reported() {
        let mut watcher = TreeWatcher::default();
        assert_eq!(watcher.observe("a".into()), Some("a"));
        assert_eq!(watcher.observe("a".into()), None);
        assert_eq!(watcher.observe("b".into()), Some("b"));
    }
}
