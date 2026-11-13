use gpui::{Context, IntoElement, Render, Window, div, prelude::*, px, rgb};

use crate::palette::{ACCENT, BACKGROUND, FOREGROUND};

pub struct Hello;

impl Render for Hello {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap(px(16.0))
            .items_center()
            .justify_center()
            .size_full()
            .bg(rgb(BACKGROUND))
            .child(
                div()
                    .text_size(px(32.0))
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(rgb(FOREGROUND))
                    .child("Hello, world"),
            )
            .child(
                div()
                    .id("button")
                    .px(px(16.0))
                    .py(px(8.0))
                    .rounded(px(6.0))
                    .bg(rgb(ACCENT))
                    .text_color(rgb(0xffffff))
                    .hover(|style| style.opacity(0.9))
                    .child("Click me"),
            )
    }
}
