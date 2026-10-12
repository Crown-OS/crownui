use input::TextInputEvent;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct Preedit {
    text: String,
    cursor: Option<(usize, usize)>,
}

impl Preedit {
    fn event(&self) -> TextInputEvent {
        TextInputEvent::Preedit {
            text: self.text.clone(),
            cursor: self.cursor,
        }
    }
}

/// The double-buffered half of text-input-v3: preedit, commit and delete events pile up
/// and are applied together, in protocol order, when `done` arrives.
#[derive(Debug, Default)]
pub(crate) struct TextInputTransaction {
    pending_preedit: Option<Preedit>,
    pending_commit: Option<String>,
    pending_delete: Option<(usize, usize)>,
    shown_preedit: Option<Preedit>,
}

impl TextInputTransaction {
    pub(crate) fn set_preedit(&mut self, text: Option<String>, cursor_begin: i32, cursor_end: i32) {
        self.pending_preedit = text.filter(|text| !text.is_empty()).map(|text| {
            let offset = |position: i32| {
                usize::try_from(position)
                    .ok()
                    .map(|offset| offset.min(text.len()))
            };
            let cursor = offset(cursor_begin).zip(offset(cursor_end));
            Preedit { text, cursor }
        });
    }

    pub(crate) fn set_commit(&mut self, text: Option<String>) {
        self.pending_commit = text.filter(|text| !text.is_empty());
    }

    pub(crate) fn set_delete(&mut self, before: u32, after: u32) {
        self.pending_delete =
            (before != 0 || after != 0).then_some((before as usize, after as usize));
    }

    /// Applies the pending state: clear the old preedit, delete, commit, then show the new
    /// preedit; only the events that change something are emitted.
    pub(crate) fn done(&mut self, mut emit: impl FnMut(TextInputEvent)) {
        let delete = self.pending_delete.take();
        let commit = self.pending_commit.take();
        let preedit = self.pending_preedit.take();
        if (delete.is_some() || commit.is_some()) && self.shown_preedit.take().is_some() {
            emit(TextInputEvent::Preedit {
                text: String::new(),
                cursor: None,
            });
        }
        if let Some((before, after)) = delete {
            emit(TextInputEvent::DeleteSurrounding { before, after });
        }
        if let Some(text) = commit {
            emit(TextInputEvent::Commit(text));
        }
        if preedit != self.shown_preedit {
            emit(preedit.as_ref().map_or(
                TextInputEvent::Preedit {
                    text: String::new(),
                    cursor: None,
                },
                Preedit::event,
            ));
            self.shown_preedit = preedit;
        }
    }

    /// Drops everything pending; a visible preedit is cleared, as focus left the field.
    pub(crate) fn reset(&mut self) -> Option<TextInputEvent> {
        self.pending_preedit = None;
        self.pending_commit = None;
        self.pending_delete = None;
        self.shown_preedit.take().map(|_| TextInputEvent::Preedit {
            text: String::new(),
            cursor: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn apply(transaction: &mut TextInputTransaction) -> Vec<TextInputEvent> {
        let mut events = Vec::new();
        transaction.done(|event| events.push(event));
        events
    }

    #[test]
    fn nothing_reaches_the_app_before_done() {
        let mut transaction = TextInputTransaction::default();
        transaction.set_preedit(Some("ni".to_owned()), 2, 2);
        assert_eq!(
            apply(&mut transaction),
            [TextInputEvent::Preedit {
                text: "ni".to_owned(),
                cursor: Some((2, 2))
            }]
        );
    }

    #[test]
    fn a_commit_clears_the_preedit_then_deletes_then_inserts() {
        let mut transaction = TextInputTransaction::default();
        transaction.set_preedit(Some("ni".to_owned()), -1, -1);
        apply(&mut transaction);
        transaction.set_delete(1, 0);
        transaction.set_commit(Some("你".to_owned()));
        assert_eq!(
            apply(&mut transaction),
            [
                TextInputEvent::Preedit {
                    text: String::new(),
                    cursor: None
                },
                TextInputEvent::DeleteSurrounding {
                    before: 1,
                    after: 0
                },
                TextInputEvent::Commit("你".to_owned()),
            ]
        );
    }

    #[test]
    fn an_unchanged_preedit_is_not_repeated_and_a_missing_one_clears() {
        let mut transaction = TextInputTransaction::default();
        transaction.set_preedit(Some("a".to_owned()), 1, 1);
        apply(&mut transaction);
        transaction.set_preedit(Some("a".to_owned()), 1, 1);
        assert!(apply(&mut transaction).is_empty());
        assert_eq!(
            apply(&mut transaction),
            [TextInputEvent::Preedit {
                text: String::new(),
                cursor: None
            }]
        );
    }

    #[test]
    fn reset_clears_a_visible_preedit_only() {
        let mut transaction = TextInputTransaction::default();
        assert_eq!(transaction.reset(), None);
        transaction.set_preedit(Some("x".to_owned()), 0, 0);
        apply(&mut transaction);
        assert!(transaction.reset().is_some());
    }
}
