use crownui::prelude::*;

fn main() -> Result<(), crownui::RunError> {
    tracing_subscriber::fmt::init();
    crownui::run(
        component(|cx| {
            let count = cx.signal(0);
            vstack((
                text(bind(move |rt| format!("Count: {}", count.get(rt))))
                    .text_size(32.0)
                    .font_semibold(),
                hstack((
                    button("−1").on_click(move |cx, _| count.update(cx, |count| *count -= 1)),
                    button("+1")
                        .bg(rgb(0x3b82f6))
                        .text_color(WHITE)
                        .on_click(move |cx, _| count.update(cx, |count| *count += 1))
                        .hover(|style| style.bg(rgb(0x2563eb)).scale(1.05))
                        .active(|style| style.scale(0.95)),
                ))
                .gap(12.0),
            ))
            .p(32.0)
            .gap(16.0)
            .items_center()
            .justify_center()
            .bg(rgb(0xf8fafc))
            .transition(Spring::snappy())
        }),
        AppConfig::new("Counter").size(420, 300).mcp(true),
    )
}
