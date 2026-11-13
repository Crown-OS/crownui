use crownui::prelude::*;

use crate::palette::{BACKGROUND, FOREGROUND};

pub fn view() -> impl View {
    vstack((
        text("Hello, world")
            .text_size(32.0)
            .font_semibold()
            .text_color(rgb(FOREGROUND)),
        button("Click me"),
    ))
    .gap(16.0)
    .items_center()
    .justify_center()
    .w_full()
    .h_full()
    .bg(rgb(BACKGROUND))
}
