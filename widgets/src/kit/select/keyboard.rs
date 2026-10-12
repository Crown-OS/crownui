use input::{Key, KeyboardEvent, NamedKey};
use view::{Cx, Event};

use super::{handles::SelectHandles, type_ahead::type_ahead};
use crate::util::row_navigation::{Step, step};

fn step_for(key: Key) -> Option<Step> {
    match key {
        Key::Named(NamedKey::ArrowDown) => Some(Step::Next),
        Key::Named(NamedKey::ArrowUp) => Some(Step::Previous),
        Key::Named(NamedKey::Home) => Some(Step::First),
        Key::Named(NamedKey::End) => Some(Step::Last),
        _ => None,
    }
}

fn typed_character(event: &KeyboardEvent) -> Option<char> {
    let plain = !event.modifiers.ctrl_key && !event.modifiers.meta_key && !event.modifiers.alt_key;
    match event.key {
        Key::Character(character) if plain && !character.is_control() => Some(character),
        _ => None,
    }
}

fn moved(
    cx: &mut Cx,
    handles: SelectHandles,
    from: Option<usize>,
    direction: Step,
) -> Option<usize> {
    let count = handles.option_count(cx);
    step(&vec![true; count], from, direction)
}

fn found_by_typing(
    cx: &mut Cx,
    handles: SelectHandles,
    character: char,
    from: Option<usize>,
) -> Option<usize> {
    let options = handles.options.get(cx);
    let mut buffer = handles.typed.get(cx);
    let found = type_ahead(&options, &mut buffer, character, from);
    handles.typed.set(cx, buffer);
    found
}

pub(super) fn handle_key(cx: &mut Cx, event: &mut Event<KeyboardEvent>, handles: SelectHandles) {
    let open = handles.popup.is_open(cx);
    let highlighted = handles.highlighted.get(cx);
    let selected = handles.selected.get(cx);
    let handled = match (open, event.key, highlighted) {
        (true, Key::Named(NamedKey::Escape), _) => {
            handles.close(cx);
            true
        }
        (true, Key::Named(NamedKey::Enter | NamedKey::Space), Some(index)) => {
            handles.pick(cx, index);
            true
        }
        (false, Key::Named(NamedKey::ArrowDown | NamedKey::ArrowUp), _) => {
            let direction = if event.key == Key::Named(NamedKey::ArrowDown) {
                Step::First
            } else {
                Step::Last
            };
            let start = selected.or_else(|| moved(cx, handles, None, direction));
            handles.open(cx, start);
            true
        }
        (true, key, _) if step_for(key).is_some() => {
            let target =
                step_for(key).and_then(|direction| moved(cx, handles, highlighted, direction));
            handles.highlighted.set(cx, target);
            true
        }
        (_, _, _) => match typed_character(event) {
            Some(character) if open => {
                let found = found_by_typing(cx, handles, character, highlighted);
                if found.is_some() {
                    handles.highlighted.set(cx, found);
                }
                found.is_some()
            }
            Some(character) => match found_by_typing(cx, handles, character, selected) {
                Some(index) => {
                    handles.choose(cx, index);
                    true
                }
                None => false,
            },
            None => false,
        },
    };
    if handled {
        event.prevent_default();
        event.stop_propagation();
    }
}
