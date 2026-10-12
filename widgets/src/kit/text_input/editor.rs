use geometry::Point;
use input::{Key, KeyboardEvent, NamedKey, TextInputEvent};
use reactive::{Runtime, Signal};
use view::{Cx, Event, NodeId};

use crate::util::{
    mask::mask,
    text_edit::{Motion, Selection, boundary_at_fraction, single_line, word_range_at},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Segment {
    Before,
    Selected,
    After,
}

#[derive(Clone, Copy)]
pub(super) struct Editor {
    pub(super) node: NodeId,
    pub(super) value: Signal<String>,
    pub(super) selection: Signal<Selection>,
    pub(super) secret: bool,
    pub(super) read_only: bool,
}

impl Editor {
    pub(super) fn selection_now(self, runtime: &mut Runtime) -> Selection {
        let selection = self.selection.get(runtime);
        self.value.with(runtime, |text| selection.clamped(text))
    }

    pub(super) fn segment(self, runtime: &mut Runtime, segment: Segment) -> String {
        let range = self.selection_now(runtime).range();
        self.value.with(runtime, |text| {
            let part = match segment {
                Segment::Before => &text[..range.start],
                Segment::Selected => &text[range],
                Segment::After => &text[range.end..],
            };
            if self.secret {
                mask(part)
            } else {
                part.to_owned()
            }
        })
    }

    fn edit(self, cx: &mut Cx, apply: impl FnOnce(&mut String, Selection) -> Option<Selection>) {
        if self.read_only {
            return;
        }
        let selection = self.selection_now(cx);
        let mut text = self.value.get(cx);
        let Some(next) = apply(&mut text, selection) else {
            return;
        };
        self.value.set(cx, text.clone());
        self.selection.set(cx, next);
        cx.emit_input(self.node, text);
    }

    pub(super) fn insert(self, cx: &mut Cx, inserted: &str) {
        let inserted = single_line(inserted);
        self.edit(cx, |text, selection| selection.replace(text, &inserted));
    }

    fn delete(self, cx: &mut Cx, motion: Motion) {
        self.edit(cx, |text, selection| selection.delete(text, motion));
    }

    pub(super) fn clear(self, cx: &mut Cx) {
        self.edit(cx, |text, _| Selection::all(text).replace(text, ""));
    }

    fn move_caret(self, cx: &mut Cx, motion: Motion, extend: bool) {
        let selection = self.selection_now(cx);
        let next = self
            .value
            .with(cx, |text| selection.after(text, motion, extend));
        self.selection.set(cx, next);
    }

    fn select_all(self, cx: &mut Cx) {
        let all = self.value.with(cx, |text| Selection::all(text));
        self.selection.set(cx, all);
    }

    fn selected_text(self, cx: &mut Cx) -> Option<String> {
        let range = self.selection_now(cx).range();
        if self.secret || range.is_empty() {
            return None;
        }
        Some(self.value.with(cx, |text| text[range].to_owned()))
    }

    fn copy(self, cx: &mut Cx) {
        if let Some(selected) = self.selected_text(cx) {
            cx.copy_to_clipboard(selected);
        }
    }

    fn cut(self, cx: &mut Cx) {
        if let Some(selected) = self.selected_text(cx) {
            cx.copy_to_clipboard(selected);
            self.delete(cx, Motion::NextGrapheme);
        }
    }

    pub(super) fn submit(self, cx: &mut Cx) {
        let value = self.value.get(cx);
        cx.emit_change(self.node, value.clone());
        cx.emit_submit(self.node, value);
    }

    pub(super) fn commit_change(self, cx: &mut Cx) {
        let value = self.value.get(cx);
        cx.emit_change(self.node, value);
    }

    pub(super) fn text_input(self, cx: &mut Cx, event: &TextInputEvent) {
        match event {
            TextInputEvent::Commit(inserted) => self.insert(cx, inserted),
            TextInputEvent::DeleteSurrounding { before, after } => {
                self.edit(cx, |text, selection| {
                    selection
                        .surrounding(text, *before, *after)
                        .replace(text, "")
                });
            }
            TextInputEvent::Preedit { .. } => {}
        }
    }

    pub(super) fn key_down(self, cx: &mut Cx, event: &mut Event<KeyboardEvent>) {
        let modifiers = event.modifiers;
        let command = modifiers.ctrl_key || modifiers.meta_key;
        let by_word = modifiers.ctrl_key || modifiers.alt_key;
        let extend = modifiers.shift_key;
        let (previous, next) = if by_word {
            (Motion::PreviousWord, Motion::NextWord)
        } else {
            (Motion::PreviousGrapheme, Motion::NextGrapheme)
        };
        let handled = match event.key {
            Key::Named(NamedKey::ArrowLeft) => self.moved(cx, previous, extend),
            Key::Named(NamedKey::ArrowRight) => self.moved(cx, next, extend),
            Key::Named(NamedKey::Home | NamedKey::ArrowUp) => self.moved(cx, Motion::Start, extend),
            Key::Named(NamedKey::End | NamedKey::ArrowDown) => self.moved(cx, Motion::End, extend),
            Key::Named(NamedKey::Backspace) => self.deleted(cx, previous),
            Key::Named(NamedKey::Delete) => self.deleted(cx, next),
            Key::Named(NamedKey::Enter) => {
                self.submit(cx);
                true
            }
            Key::Character(character) if command => self.shortcut(cx, character),
            Key::Character(_) => event
                .text
                .as_deref()
                .filter(|typed| !typed.chars().any(char::is_control))
                .is_some_and(|typed| {
                    self.insert(cx, typed);
                    true
                }),
            Key::Named(NamedKey::Space) => {
                self.insert(cx, event.text.as_deref().unwrap_or(" "));
                true
            }
            _ => false,
        };
        if handled {
            event.prevent_default();
        }
    }

    fn moved(self, cx: &mut Cx, motion: Motion, extend: bool) -> bool {
        self.move_caret(cx, motion, extend);
        true
    }

    fn deleted(self, cx: &mut Cx, motion: Motion) -> bool {
        self.delete(cx, motion);
        true
    }

    fn shortcut(self, cx: &mut Cx, character: char) -> bool {
        match character.to_ascii_lowercase() {
            'a' => self.select_all(cx),
            'c' => self.copy(cx),
            'x' => self.cut(cx),
            'v' => cx.paste_from_clipboard(),
            _ => return false,
        }
        true
    }

    pub(super) fn place_caret(self, cx: &mut Cx, fraction: f32, clicks: u32) {
        let next = self.value.with(cx, |text| {
            let index = boundary_at_fraction(text, fraction);
            match clicks {
                1 => Selection::at(index),
                2 => {
                    let word = word_range_at(text, index);
                    Selection {
                        anchor: word.start,
                        caret: word.end,
                    }
                }
                _ => Selection::all(text),
            }
        });
        self.selection.set(cx, next);
    }

    pub(super) fn drag_to(self, cx: &mut Cx, fraction: f32) {
        let anchor = self.selection_now(cx).anchor;
        let caret = self
            .value
            .with(cx, |text| boundary_at_fraction(text, fraction));
        self.selection.set(cx, Selection { anchor, caret });
    }
}

pub(super) fn fraction_along(position: Point, inset: f32, scroll: f32, content_width: f32) -> f32 {
    if content_width <= 0.0 {
        return 0.0;
    }
    (position.x - inset + scroll) / content_width
}
