mod change;
mod direction;

use geometry::Rect;
use input::{Key, KeyboardEvent, NamedKey};

pub use change::FocusChange;
pub use direction::Direction;

/// Keyboard focus over an ordered list of focusable members and their window rects.
///
/// Tab and Shift+Tab walk the list in order, wrapping around; arrow keys pick the member whose
/// rect center is nearest ahead by [`Direction::weighted_distance`]. Like CSS `:focus-visible`,
/// [`focus_visible`](Self::focus_visible) is true only while focus last moved by keyboard.
#[derive(Debug, Clone)]
pub struct FocusManager<K> {
    focusables: Vec<(K, Rect)>,
    focused: Option<K>,
    focus_visible: bool,
}

impl<K> Default for FocusManager<K> {
    fn default() -> Self {
        Self {
            focusables: Vec::new(),
            focused: None,
            focus_visible: false,
        }
    }
}

impl<K: Copy + Eq> FocusManager<K> {
    /// Nothing focusable, nothing focused.
    pub fn new() -> Self {
        Self::default()
    }

    /// Replaces the focusable members, in Tab order; focus survives when its member still exists
    /// and is dropped otherwise.
    pub fn set_focusables(&mut self, focusables: &[(K, Rect)]) -> Option<FocusChange<K>> {
        self.focusables.clear();
        self.focusables.extend_from_slice(focusables);
        match self.focused {
            Some(member) if self.index_of(member).is_none() => self.move_focus(None, false),
            _ => None,
        }
    }

    /// The focused member.
    pub fn focused(&self) -> Option<K> {
        self.focused
    }

    /// Whether focus last moved by keyboard, so a focus ring should show.
    pub fn focus_visible(&self) -> bool {
        self.focus_visible
    }

    /// Focuses `member` by pointer or program; `None` when it is not focusable or nothing changed.
    pub fn focus(&mut self, member: K) -> Option<FocusChange<K>> {
        self.index_of(member)?;
        self.move_focus(Some(member), false)
    }

    /// Removes focus.
    pub fn blur(&mut self) -> Option<FocusChange<K>> {
        self.move_focus(None, false)
    }

    /// Focuses the next member in Tab order, wrapping; the first when nothing is focused.
    pub fn focus_next(&mut self) -> Option<FocusChange<K>> {
        let count = self.focusables.len();
        let next = self.focused_index().map_or(0, |index| (index + 1) % count);
        self.move_focus_to_index(next)
    }

    /// Focuses the previous member in Tab order, wrapping; the last when nothing is focused.
    pub fn focus_previous(&mut self) -> Option<FocusChange<K>> {
        let count = self.focusables.len();
        let previous = self
            .focused_index()
            .map_or(count.checked_sub(1)?, |index| (index + count - 1) % count);
        self.move_focus_to_index(previous)
    }

    /// Focuses the nearest member in `direction`; the first member when nothing is focused.
    pub fn focus_direction(&mut self, direction: Direction) -> Option<FocusChange<K>> {
        let Some(index) = self.focused_index() else {
            return self.focus_next();
        };
        let origin = self.focusables[index].1.center();
        let (target, _) = self
            .focusables
            .iter()
            .enumerate()
            .filter(|(candidate, _)| *candidate != index)
            .filter_map(|(candidate, (_, rect))| {
                direction
                    .weighted_distance(origin, rect.center())
                    .map(|distance| (candidate, distance))
            })
            .min_by(|(_, left), (_, right)| left.total_cmp(right))?;
        self.move_focus_to_index(target)
    }

    /// Handles Tab, Shift+Tab and the arrow keys of a key press; other keys and chords with
    /// Ctrl, Alt or Meta are left alone. Returns the change when focus or its visibility moved.
    pub fn handle_key(&mut self, event: &KeyboardEvent) -> Option<FocusChange<K>> {
        let modifiers = event.modifiers;
        if modifiers.ctrl_key || modifiers.alt_key || modifiers.meta_key {
            return None;
        }
        match event.key {
            Key::Named(NamedKey::Tab) if modifiers.shift_key => self.focus_previous(),
            Key::Named(NamedKey::Tab) => self.focus_next(),
            Key::Named(arrow) if !modifiers.shift_key => {
                self.focus_direction(Direction::from_arrow(arrow)?)
            }
            _ => None,
        }
    }

    fn index_of(&self, member: K) -> Option<usize> {
        self.focusables
            .iter()
            .position(|(candidate, _)| *candidate == member)
    }

    fn focused_index(&self) -> Option<usize> {
        self.focused.and_then(|member| self.index_of(member))
    }

    fn move_focus_to_index(&mut self, index: usize) -> Option<FocusChange<K>> {
        let (member, _) = *self.focusables.get(index)?;
        self.move_focus(Some(member), true)
    }

    fn move_focus(&mut self, member: Option<K>, by_keyboard: bool) -> Option<FocusChange<K>> {
        let visible = by_keyboard && member.is_some();
        if member == self.focused && visible == self.focus_visible {
            return None;
        }
        let previous = std::mem::replace(&mut self.focused, member);
        self.focus_visible = visible;
        Some(FocusChange {
            previous,
            current: member,
        })
    }
}

#[cfg(test)]
mod tests {
    use input::{Modifiers, PhysicalKey};

    use super::*;

    fn grid() -> FocusManager<char> {
        let mut manager = FocusManager::new();
        let cells = [
            ('a', Rect::from_xywh(0.0, 0.0, 10.0, 10.0)),
            ('b', Rect::from_xywh(20.0, 0.0, 10.0, 10.0)),
            ('c', Rect::from_xywh(0.0, 20.0, 10.0, 10.0)),
            ('d', Rect::from_xywh(20.0, 20.0, 10.0, 10.0)),
        ];
        assert_eq!(manager.set_focusables(&cells), None);
        manager
    }

    fn key(key: NamedKey, shift_key: bool) -> KeyboardEvent {
        KeyboardEvent {
            key: Key::Named(key),
            code: PhysicalKey(0),
            text: None,
            repeat: false,
            modifiers: Modifiers {
                shift_key,
                ..Modifiers::NONE
            },
        }
    }

    #[test]
    fn tab_and_shift_tab_wrap_around() {
        let mut manager = grid();
        manager.handle_key(&key(NamedKey::Tab, false));
        assert_eq!(manager.focused(), Some('a'));
        assert_eq!(
            manager.handle_key(&key(NamedKey::Tab, true)),
            Some(FocusChange {
                previous: Some('a'),
                current: Some('d')
            })
        );
        manager.focus_next();
        assert_eq!(manager.focused(), Some('a'));
    }

    #[test]
    fn shift_tab_from_nothing_focuses_the_last() {
        let mut manager = grid();
        manager.focus_previous();
        assert_eq!(manager.focused(), Some('d'));
    }

    #[test]
    fn arrows_pick_the_nearest_member_ahead() {
        let mut manager = grid();
        manager.focus('a');
        manager.handle_key(&key(NamedKey::ArrowRight, false));
        assert_eq!(manager.focused(), Some('b'));
        manager.handle_key(&key(NamedKey::ArrowDown, false));
        assert_eq!(manager.focused(), Some('d'));
        manager.handle_key(&key(NamedKey::ArrowLeft, false));
        assert_eq!(manager.focused(), Some('c'));
        assert_eq!(manager.handle_key(&key(NamedKey::ArrowLeft, false)), None);
        assert_eq!(manager.focused(), Some('c'));
    }

    #[test]
    fn cross_axis_offset_costs_more_than_distance_ahead() {
        let mut manager = FocusManager::new();
        let _ = manager.set_focusables(&[
            (0, Rect::from_xywh(0.0, 0.0, 10.0, 10.0)),
            (1, Rect::from_xywh(15.0, 15.0, 10.0, 10.0)),
            (2, Rect::from_xywh(40.0, 0.0, 10.0, 10.0)),
        ]);
        manager.focus(0);
        manager.focus_direction(Direction::Right);
        assert_eq!(manager.focused(), Some(2));
    }

    #[test]
    fn focus_visible_follows_the_last_input() {
        let mut manager = grid();
        manager.focus('b');
        assert!(!manager.focus_visible());
        assert_eq!(
            manager.handle_key(&key(NamedKey::Tab, false)),
            Some(FocusChange {
                previous: Some('b'),
                current: Some('c')
            })
        );
        assert!(manager.focus_visible());
        assert_eq!(
            manager.focus('c'),
            Some(FocusChange {
                previous: Some('c'),
                current: Some('c')
            })
        );
        assert!(!manager.focus_visible());
    }

    #[test]
    fn focus_survives_rebuilds_only_while_its_member_exists() {
        let mut manager = grid();
        manager.focus('b');
        assert_eq!(
            manager.set_focusables(&[('b', Rect::from_xywh(5.0, 5.0, 1.0, 1.0))]),
            None
        );
        assert_eq!(manager.focused(), Some('b'));
        assert_eq!(
            manager.set_focusables(&[('a', Rect::ZERO)]),
            Some(FocusChange {
                previous: Some('b'),
                current: None
            })
        );
        assert_eq!(manager.focus('z'), None);
    }

    #[test]
    fn chords_and_other_keys_are_ignored() {
        let mut manager = grid();
        let mut ctrl_tab = key(NamedKey::Tab, false);
        ctrl_tab.modifiers.ctrl_key = true;
        assert_eq!(manager.handle_key(&ctrl_tab), None);
        assert_eq!(manager.handle_key(&key(NamedKey::Enter, false)), None);
        assert_eq!(FocusManager::<u8>::new().focus_next(), None);
    }
}
