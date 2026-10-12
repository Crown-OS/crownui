use geometry::Color;
use input::{Key, NamedKey, TextInputEvent};
use layouting::Align;
use reactive::Signal;
use semantics::Role;
use view::{Corners, Cx, Edges, Element, Kind, Listener, NodeId, StyleProp, Styled, bind, show};
use windowing::CursorIcon;

use crate::{stack::hstack, text::text};

const FIELD_BACKGROUND: Color = Color::new(0.5, 0.5, 0.5, 0.12);
const PLACEHOLDER_COLOR: Color = Color::new(0.5, 0.5, 0.5, 0.8);
const CARET_COLOR: Color = Color::new(0.23, 0.51, 0.96, 1.0);
const CARET_WIDTH: f32 = 2.0;
const CARET_HEIGHT: f32 = 16.0;

/// The kind behind [`text_field`].
pub struct TextFieldKind {
    value: Signal<String>,
    placeholder: String,
}

/// Byte offset of the `chars`-th character of `text`.
fn byte_index(text: &str, chars: usize) -> usize {
    text.char_indices()
        .nth(chars)
        .map_or(text.len(), |(index, _)| index)
}

/// Edits `value` at the caret and fires DOM `input` when the text changed.
fn edit(
    cx: &mut Cx,
    node: NodeId,
    value: Signal<String>,
    caret: Signal<usize>,
    change: impl FnOnce(&mut String, &mut usize),
) {
    let mut position = caret.get(cx);
    let mut edited = None;
    value.update(cx, |text| {
        position = position.min(text.chars().count());
        let before = text.clone();
        change(text, &mut position);
        if *text != before {
            edited = Some(text.clone());
        }
    });
    caret.set(cx, position);
    if let Some(text) = edited {
        cx.emit_input(node, text);
    }
}

fn insert(text: &mut String, caret: &mut usize, inserted: &str) {
    text.insert_str(byte_index(text, *caret), inserted);
    *caret += inserted.chars().count();
}

fn handle_key(
    cx: &mut Cx,
    node: NodeId,
    value: Signal<String>,
    caret: Signal<usize>,
    event: &mut view::Event<input::KeyboardEvent>,
) {
    let length = value.with(cx, |text| text.chars().count());
    let handled = match event.key {
        Key::Named(NamedKey::ArrowLeft) => {
            caret.update(cx, |position| *position = position.saturating_sub(1));
            true
        }
        Key::Named(NamedKey::ArrowRight) => {
            caret.update(cx, |position| *position = (*position + 1).min(length));
            true
        }
        Key::Named(NamedKey::Home) => {
            caret.set(cx, 0);
            true
        }
        Key::Named(NamedKey::End) => {
            caret.set(cx, length);
            true
        }
        Key::Named(NamedKey::Backspace) => {
            edit(cx, node, value, caret, |text, position| {
                if *position > 0 {
                    let start = byte_index(text, *position - 1);
                    text.replace_range(start..byte_index(text, *position), "");
                    *position -= 1;
                }
            });
            true
        }
        Key::Named(NamedKey::Delete) => {
            edit(cx, node, value, caret, |text, position| {
                if *position < text.chars().count() {
                    let start = byte_index(text, *position);
                    text.replace_range(start..byte_index(text, *position + 1), "");
                }
            });
            true
        }
        Key::Named(NamedKey::Enter) => {
            let submitted = value.get(cx);
            cx.emit_change(node, submitted.clone());
            cx.emit_submit(node, submitted);
            true
        }
        Key::Character(character) if event.modifiers.ctrl_key || event.modifiers.meta_key => {
            match character.to_ascii_lowercase() {
                'c' => {
                    let copied = value.get(cx);
                    cx.copy_to_clipboard(copied);
                    true
                }
                'x' => {
                    let cut = value.get(cx);
                    cx.copy_to_clipboard(cut);
                    edit(cx, node, value, caret, |text, position| {
                        text.clear();
                        *position = 0;
                    });
                    true
                }
                'v' => {
                    cx.paste_from_clipboard();
                    true
                }
                _ => false,
            }
        }
        Key::Character(_) | Key::Named(NamedKey::Space) => match event
            .text
            .clone()
            .filter(|typed| !typed.chars().any(char::is_control))
        {
            Some(typed) => {
                edit(cx, node, value, caret, |text, position| {
                    insert(text, position, &typed)
                });
                true
            }
            None => false,
        },
        _ => false,
    };
    if handled {
        event.prevent_default();
    }
}

impl Kind for TextFieldKind {
    const NAME: &'static str = "text_field";

    fn mount(self, cx: &mut Cx, node: NodeId) {
        let Self { value, placeholder } = self;
        let focused = cx.signal(false);
        let initial_length = value.with(cx, |text| text.chars().count());
        let caret = cx.signal(initial_length);
        cx.set_role(node, Role::TextField);
        cx.bind_value(node, move |runtime| value.get(runtime));
        for prop in [
            StyleProp::FlexDirection(layouting::FlexDirection::Row),
            StyleProp::AlignItems(Align::Center),
            StyleProp::Padding(Edges::Horizontal, 10.0),
            StyleProp::Padding(Edges::Vertical, 6.0),
            StyleProp::Radius(Corners::All, 8.0),
            StyleProp::Background(Some(FIELD_BACKGROUND)),
            StyleProp::Cursor(CursorIcon::Text),
        ] {
            cx.set_default_style(node, prop);
        }
        let prefix = move |runtime: &mut reactive::Runtime| {
            let position = caret.get(runtime);
            value.with(runtime, |text| {
                text[..byte_index(text, position)].to_owned()
            })
        };
        let suffix = move |runtime: &mut reactive::Runtime| {
            let position = caret.get(runtime);
            value.with(runtime, |text| {
                text[byte_index(text, position)..].to_owned()
            })
        };
        let content = hstack((
            text(bind(prefix)),
            show(
                bind(move |runtime| focused.get(runtime)),
                || hstack(()).w(CARET_WIDTH).h(CARET_HEIGHT).bg(CARET_COLOR),
                || hstack(()),
            ),
            show(
                bind(move |runtime| value.with(runtime, String::is_empty)),
                move || text(placeholder.clone()).text_color(PLACEHOLDER_COLOR),
                move || text(bind(suffix)),
            ),
        ))
        .items_center();
        cx.set_children(node, content);
        cx.add_listener(
            node,
            Listener::TextInput(Box::new(move |cx, event| match event.detail() {
                TextInputEvent::Commit(inserted) => {
                    let inserted = inserted.clone();
                    edit(cx, node, value, caret, |text, position| {
                        insert(text, position, &inserted)
                    });
                }
                TextInputEvent::DeleteSurrounding { before, after } => {
                    let (before, after) = (*before, *after);
                    edit(cx, node, value, caret, |text, position| {
                        let at = byte_index(text, *position);
                        let start = (0..=at.saturating_sub(before))
                            .rev()
                            .find(|index| text.is_char_boundary(*index))
                            .unwrap_or(0);
                        let end = ((at + after).min(text.len())..=text.len())
                            .find(|index| text.is_char_boundary(*index))
                            .unwrap_or(text.len());
                        let removed_before = text[start..at].chars().count();
                        text.replace_range(start..end, "");
                        *position -= removed_before;
                    });
                }
                TextInputEvent::Preedit { .. } => {}
            })),
        );
        cx.add_listener(
            node,
            Listener::KeyDown(Box::new(move |cx, event| {
                handle_key(cx, node, value, caret, event)
            })),
        );
        cx.add_listener(
            node,
            Listener::Focus(Box::new(move |cx, _| focused.set(cx, true))),
        );
        cx.add_listener(
            node,
            Listener::Blur(Box::new(move |cx, _| {
                focused.set(cx, false);
                let committed = value.get(cx);
                cx.emit_change(node, committed);
            })),
        );
        cx.add_listener(
            node,
            Listener::Click(Box::new(move |cx, _| cx.request_focus(node))),
        );
    }
}

/// A single-line editable field bound to `value`: typing and IME commits insert at the caret,
/// arrows/Home/End move it, Backspace/Delete edit around it, Enter submits. Listen with
/// `.on_input`, `.on_change` and `.on_submit`.
pub fn text_field(value: Signal<String>, placeholder: impl Into<String>) -> Element<TextFieldKind> {
    Element::new(TextFieldKind {
        value,
        placeholder: placeholder.into(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn caret_offsets_respect_multibyte_characters() {
        assert_eq!(byte_index("héllo", 2), 3);
        assert_eq!(byte_index("héllo", 10), 6);
        let mut text = String::from("hllo");
        let mut caret = 1;
        insert(&mut text, &mut caret, "é");
        assert_eq!((text.as_str(), caret), ("héllo", 2));
    }
}
