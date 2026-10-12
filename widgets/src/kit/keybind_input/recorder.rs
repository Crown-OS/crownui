use crownconfig::{KeyCode, Keybind, Mods};
use input::{KeyboardEvent, NamedKey, PhysicalKey};

use super::metrics::{PENDING_HINT, RECORDING_HINT, UNBOUND_HINT};
use crate::util::keybind::{from_toolkit, key_code, merge, modifier_of, mods_of, written};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum KeyPhase {
    Down,
    Up,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Outcome {
    Pending,
    Commit(Keybind),
    Cancel,
    Ignored,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) enum Piece {
    Cap(String),
    Hint(&'static str),
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct Recorder {
    recording: bool,
    held: Vec<(PhysicalKey, NamedKey)>,
}

impl Recorder {
    pub(super) fn is_recording(&self) -> bool {
        self.recording
    }

    pub(super) fn arm(&mut self) {
        self.recording = true;
        self.held.clear();
    }

    pub(super) fn disarm(&mut self) {
        self.recording = false;
        self.held.clear();
    }

    fn tracked_mods(&self) -> Mods {
        self.held.iter().fold(Mods::NONE, |mods, (_, modifier)| {
            merge(mods, mods_of(*modifier))
        })
    }

    pub(super) fn shown(&self, bound: Keybind) -> Keybind {
        if self.recording {
            Keybind::new(self.tracked_mods(), None)
        } else {
            bound
        }
    }

    pub(super) fn pieces(&self, bound: Keybind) -> Vec<Piece> {
        let mut pieces: Vec<Piece> = written(self.shown(bound))
            .split('+')
            .filter(|part| !part.is_empty())
            .map(|part| Piece::Cap(part.to_owned()))
            .collect();
        let hint = match (self.recording, pieces.is_empty()) {
            (true, true) => Some(RECORDING_HINT),
            (true, false) => Some(PENDING_HINT),
            (false, true) => Some(UNBOUND_HINT),
            (false, false) => None,
        };
        pieces.extend(hint.map(Piece::Hint));
        pieces
    }

    pub(super) fn on_key(&mut self, event: &KeyboardEvent, phase: KeyPhase) -> Outcome {
        match (phase, modifier_of(event.key)) {
            (KeyPhase::Down, Some(modifier)) => {
                let held = (event.code, modifier);
                if !self.held.contains(&held) {
                    self.held.push(held);
                }
                Outcome::Pending
            }
            (KeyPhase::Up, Some(modifier)) => {
                let mods = self.tracked_mods();
                self.held.retain(|held| *held != (event.code, modifier));
                if self.held.is_empty() && !mods.is_empty() {
                    Outcome::Commit(Keybind::new(mods, None))
                } else {
                    Outcome::Pending
                }
            }
            (KeyPhase::Down, None) => {
                let mods = merge(self.tracked_mods(), from_toolkit(event.modifiers));
                match (key_code(event), mods.is_empty()) {
                    (Some(KeyCode::Escape), true) => Outcome::Cancel,
                    (Some(KeyCode::Backspace | KeyCode::Delete), true) => {
                        Outcome::Commit(Keybind::NONE)
                    }
                    (Some(key), _) => Outcome::Commit(Keybind::new(mods, Some(key))),
                    (None, _) => Outcome::Ignored,
                }
            }
            (KeyPhase::Up, None) => Outcome::Ignored,
        }
    }
}

#[cfg(test)]
mod tests {
    use input::{Key, Modifiers};

    use super::*;

    fn key(key: Key, code: u32) -> KeyboardEvent {
        KeyboardEvent {
            key,
            code: PhysicalKey(code),
            text: None,
            repeat: false,
            modifiers: Modifiers::NONE,
        }
    }

    const LEFT_CTRL: (Key, u32) = (Key::Named(NamedKey::Control), 29);
    const LEFT_SHIFT: (Key, u32) = (Key::Named(NamedKey::Shift), 42);
    const RIGHT_SHIFT: (Key, u32) = (Key::Named(NamedKey::Shift), 54);
    const LEFT_META: (Key, u32) = (Key::Named(NamedKey::Meta), 125);
    const KEY_D: (Key, u32) = (Key::Character('d'), 32);
    const ESCAPE: (Key, u32) = (Key::Named(NamedKey::Escape), 1);
    const BACKSPACE: (Key, u32) = (Key::Named(NamedKey::Backspace), 14);

    fn armed() -> Recorder {
        let mut recorder = Recorder::default();
        recorder.arm();
        recorder
    }

    fn strike(recorder: &mut Recorder, (logical, code): (Key, u32), phase: KeyPhase) -> Outcome {
        recorder.on_key(&key(logical, code), phase)
    }

    #[test]
    fn striking_a_key_commits_the_chord_with_the_modifiers_held() {
        let mut recorder = armed();
        strike(&mut recorder, LEFT_CTRL, KeyPhase::Down);
        assert_eq!(
            strike(&mut recorder, KEY_D, KeyPhase::Down),
            Outcome::Commit(Keybind::new(
                Mods {
                    ctrl: true,
                    ..Mods::NONE
                },
                Some(KeyCode::D)
            ))
        );
    }

    #[test]
    fn a_modifier_held_before_arming_comes_in_with_the_key_event() {
        let mut recorder = armed();
        let mut event = key(KEY_D.0, KEY_D.1);
        event.modifiers.alt_key = true;
        assert_eq!(
            recorder.on_key(&event, KeyPhase::Down),
            Outcome::Commit(Keybind::new(
                Mods {
                    alt: true,
                    ..Mods::NONE
                },
                Some(KeyCode::D)
            ))
        );
    }

    #[test]
    fn releasing_the_last_modifier_commits_a_modifier_only_chord() {
        let mut recorder = armed();
        assert_eq!(
            strike(&mut recorder, LEFT_META, KeyPhase::Down),
            Outcome::Pending
        );
        assert_eq!(
            strike(&mut recorder, LEFT_META, KeyPhase::Up),
            Outcome::Commit(Keybind::new(Mods::META, None))
        );
    }

    #[test]
    fn a_modifier_stays_held_until_every_key_that_sets_it_is_up() {
        let mut recorder = armed();
        strike(&mut recorder, LEFT_SHIFT, KeyPhase::Down);
        strike(&mut recorder, RIGHT_SHIFT, KeyPhase::Down);
        assert_eq!(
            strike(&mut recorder, LEFT_SHIFT, KeyPhase::Up),
            Outcome::Pending
        );
        assert_eq!(
            strike(&mut recorder, RIGHT_SHIFT, KeyPhase::Up),
            Outcome::Commit(Keybind::new(
                Mods {
                    shift: true,
                    ..Mods::NONE
                },
                None
            ))
        );
    }

    #[test]
    fn bare_escape_cancels_and_bare_backspace_clears_but_a_modifier_makes_them_keys() {
        let mut recorder = armed();
        assert_eq!(
            strike(&mut recorder, ESCAPE, KeyPhase::Down),
            Outcome::Cancel
        );
        assert_eq!(
            strike(&mut recorder, BACKSPACE, KeyPhase::Down),
            Outcome::Commit(Keybind::NONE)
        );
        strike(&mut recorder, LEFT_CTRL, KeyPhase::Down);
        assert_eq!(
            strike(&mut recorder, ESCAPE, KeyPhase::Down),
            Outcome::Commit(Keybind::new(
                Mods {
                    ctrl: true,
                    ..Mods::NONE
                },
                Some(KeyCode::Escape)
            ))
        );
    }

    #[test]
    fn a_key_no_shortcut_may_use_is_ignored() {
        let mut recorder = armed();
        assert_eq!(
            strike(
                &mut recorder,
                (Key::Named(NamedKey::Enter), 96),
                KeyPhase::Down
            ),
            Outcome::Ignored
        );
    }

    #[test]
    fn an_ordinary_key_coming_up_says_nothing() {
        let mut recorder = armed();
        assert_eq!(strike(&mut recorder, KEY_D, KeyPhase::Up), Outcome::Ignored);
    }

    #[test]
    fn arming_forgets_what_was_held_before() {
        let mut recorder = armed();
        strike(&mut recorder, LEFT_CTRL, KeyPhase::Down);
        recorder.arm();
        assert_eq!(recorder.shown(Keybind::NONE), Keybind::NONE);
    }

    #[test]
    fn the_pieces_follow_the_state() {
        let mut recorder = Recorder::default();
        assert_eq!(
            recorder.pieces(Keybind::SUPER_SPACE),
            [Piece::Cap("Super".into()), Piece::Cap("Space".into())]
        );
        assert_eq!(recorder.pieces(Keybind::NONE), [Piece::Hint(UNBOUND_HINT)]);
        recorder.arm();
        assert_eq!(
            recorder.pieces(Keybind::SUPER_SPACE),
            [Piece::Hint(RECORDING_HINT)]
        );
        strike(&mut recorder, LEFT_META, KeyPhase::Down);
        assert_eq!(
            recorder.pieces(Keybind::SUPER_SPACE),
            [Piece::Cap("Super".into()), Piece::Hint(PENDING_HINT)]
        );
    }
}
