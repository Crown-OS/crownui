use input::{Key, KeyboardEvent, NamedKey};
use view::{Cx, Event};

use super::{entry::RowSpec, handles::MenuHandles, panel::panel_height};
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

fn move_highlight(cx: &mut Cx, handles: MenuHandles, rows: &[RowSpec], direction: Step) {
    let pickable: Vec<bool> = rows
        .iter()
        .map(|row| !row.separator && !row.disabled.resolve(cx))
        .collect();
    let current = handles.highlighted.get(cx);
    handles
        .highlighted
        .set(cx, step(&pickable, current, direction));
}

fn open_highlighting(cx: &mut Cx, handles: MenuHandles, rows: &[RowSpec], direction: Step) {
    handles.popup.reveal(cx, panel_height(rows));
    move_highlight(cx, handles, rows, direction);
}

pub(super) fn handle_key(
    cx: &mut Cx,
    event: &mut Event<KeyboardEvent>,
    handles: MenuHandles,
    rows: &[RowSpec],
) {
    let open = handles.popup.is_open(cx);
    let highlighted = handles.highlighted.get(cx);
    let handled = match (open, event.key, highlighted) {
        (true, Key::Named(NamedKey::Escape), _) => {
            handles.close(cx);
            true
        }
        (true, Key::Named(NamedKey::Enter | NamedKey::Space), Some(index)) => {
            handles.pick(cx, index);
            true
        }
        (false, Key::Named(NamedKey::ArrowDown), _) => {
            open_highlighting(cx, handles, rows, Step::First);
            true
        }
        (false, Key::Named(NamedKey::ArrowUp), _) => {
            open_highlighting(cx, handles, rows, Step::Last);
            true
        }
        (true, key, _) => step_for(key).is_some_and(|direction| {
            move_highlight(cx, handles, rows, direction);
            true
        }),
        _ => false,
    };
    if handled {
        event.prevent_default();
        event.stop_propagation();
    }
}
