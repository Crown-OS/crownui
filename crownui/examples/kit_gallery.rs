use crownui::{
    kit::{
        self, ButtonOptions, ButtonVariant, StatusOptions, Tone, button, card, choice_list,
        provide_configured_theme, radio_group, section, setting_row, slider, spinner, status,
        toggle, toggle_group,
    },
    prelude::*,
};

fn main() -> Result<(), crownui::RunError> {
    tracing_subscriber::fmt::init();
    crownui::run(
        component(|cx| {
            let theme = provide_configured_theme(cx);
            let wifi = cx.signal(true);
            let volume = cx.signal(0.4);
            let layout = cx.signal(0);
            let theme_name = cx.signal(None::<usize>);
            let background = theme.with(cx, |theme| theme.surface.bg);
            scroll_view(
                vstack((
                    text("CrownOS kit").text_size(24.0).font_semibold(),
                    section(
                        "Network",
                        (
                            setting_row("Wi-Fi", toggle(wifi)),
                            setting_row("Volume", slider(volume).w(180.0)),
                            setting_row(
                                "Layout",
                                toggle_group(
                                    [kit::ToggleItem::text("Grid"), kit::ToggleItem::text("List")],
                                    layout,
                                ),
                            ),
                        ),
                    ),
                    card((
                        status("Connected").tone(Tone::Success),
                        hstack((
                            button("Save").variant(ButtonVariant::Primary),
                            button("Cancel"),
                            button("Forget…").variant(ButtonVariant::Destructive),
                            spinner(),
                        ))
                        .gap(8.0)
                        .items_center(),
                        radio_group(["Light", "Dark", "Auto"], theme_name),
                    )),
                    choice_list(
                        [
                            kit::Choice::new("Erase disk", "Use the whole disk"),
                            kit::Choice::new("Alongside", "Keep your files"),
                        ],
                        theme_name,
                    ),
                ))
                .p(24.0)
                .gap(16.0)
                .bg(background),
            )
        }),
        AppConfig::new("CrownOS kit")
            .size(520, 760)
            .font(kit::inter_font().to_vec(), Some(kit::INTER_FAMILY)),
    )
}
