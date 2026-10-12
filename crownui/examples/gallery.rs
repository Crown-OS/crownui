use crownui::prelude::*;

fn section(title: &'static str, content: impl View) -> impl View {
    vstack((
        text(title)
            .text_size(13.0)
            .font_semibold()
            .text_color(rgb(0x64748b)),
        content,
    ))
    .gap(8.0)
    .p(16.0)
    .rounded(14.0)
    .bg(WHITE)
    .shadow(Shadow::sm())
    .dark(|style| style.bg(rgb(0x1e293b)))
}

fn main() -> Result<(), crownui::RunError> {
    tracing_subscriber::fmt::init();
    crownui::run(
        component(|cx| {
            let wifi = cx.signal(true);
            let bluetooth = cx.signal(false);
            let name = cx.signal(String::new());
            let dark = cx.signal(false);
            scroll_view(
                vstack((
                    text("CrownUI gallery").text_size(28.0).font_bold(),
                    section(
                        "Buttons",
                        hstack((
                            button("Default"),
                            button("Primary")
                                .bg(rgb(0x3b82f6))
                                .text_color(WHITE)
                                .hover(|style| style.bg(rgb(0x2563eb)))
                                .active(|style| style.scale(0.96)),
                            button("Disabled")
                                .disabled(true)
                                .when_disabled(|style| style.opacity(0.4)),
                        ))
                        .gap(8.0),
                    ),
                    section(
                        "Toggles",
                        vstack((
                            hstack((text("Wi-Fi").grow(1.0), toggle(wifi).label("Wi-Fi")))
                                .items_center(),
                            hstack((
                                text("Bluetooth").grow(1.0),
                                toggle(bluetooth).label("Bluetooth"),
                            ))
                            .items_center(),
                            hstack((
                                text("Dark mode").grow(1.0),
                                toggle(dark).label("Dark mode").on_click(move |cx, _| {
                                    let scheme = if dark.get(cx) {
                                        ColorScheme::Dark
                                    } else {
                                        ColorScheme::Light
                                    };
                                    cx.color_scheme().set(cx, scheme);
                                }),
                            ))
                            .items_center(),
                        ))
                        .gap(10.0),
                    ),
                    section(
                        "Text field",
                        vstack((
                            text_field(name, "Your name").label("Name"),
                            text(bind(move |rt| {
                                let name = name.get(rt);
                                if name.is_empty() {
                                    "Hello!".to_string()
                                } else {
                                    format!("Hello, {name}!")
                                }
                            })),
                        ))
                        .gap(8.0),
                    ),
                    section(
                        "Grid",
                        vstack(
                            (0..6)
                                .map(|index| {
                                    text(format!("Cell {index}"))
                                        .p(12.0)
                                        .rounded(8.0)
                                        .bg(rgba(0x3b82f61a))
                                        .text_center()
                                })
                                .collect::<Vec<_>>(),
                        )
                        .grid_cols(3)
                        .gap(8.0),
                    ),
                ))
                .gap(16.0)
                .p(24.0),
            )
            .bg(rgb(0xf1f5f9))
            .dark(|style| style.bg(rgb(0x0f172a)).text_color(WHITE))
            .transition(Spring::smooth())
        }),
        AppConfig::new("Gallery").size(520, 760).mcp(true),
    )
}
