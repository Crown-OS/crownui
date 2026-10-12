use crowntest::{Role, TestApp, role, text as text_query};
use view::{Listeners, Styled, component};
use widgets::{
    kit::{StatusOptions, Theme, Tone, button, provide_theme, status},
    vstack,
};

const ALL_TONES: [Tone; 4] = [Tone::Success, Tone::Warning, Tone::Danger, Tone::Neutral];

fn stage(theme: Theme) -> TestApp {
    TestApp::new(component(move |cx| {
        provide_theme(cx, theme);
        let lines: Vec<_> = ALL_TONES
            .into_iter()
            .map(|tone| status(format!("{tone:?}")).tone(tone))
            .collect();
        vstack(lines).p(24.0).gap(8.0)
    }))
}

#[test]
fn the_line_is_a_seven_pixel_dot_then_a_six_pixel_gap_then_the_label() {
    let mut app = stage(Theme::dark());
    let label = app.find(text_query("Success")).bounds;
    assert_eq!(label.origin.x, 24.0 + 7.0 + 6.0);
    let label_width = label.size.width;
    let line = app.layout_snapshot();
    assert!(
        line.contains(&format!(
            "kit_status @24,24 {}x14.4",
            7.0 + 6.0 + label_width
        )),
        "{line}"
    );
}

#[test]
fn the_dot_is_centred_on_the_label() {
    let mut app = stage(Theme::dark());
    let label = app.find(text_query("Success")).bounds;
    let layout = app.layout_snapshot();
    let dot_top = 24.0 + (label.size.height - 7.0) / 2.0;
    assert!(
        layout.contains(&format!("kit_status > stack @24,{dot_top} 7x7")),
        "{layout}"
    );
}

#[test]
fn the_label_is_twelve_pixels_tall_text() {
    let mut app = stage(Theme::dark());
    let label = app.find(text_query("Neutral")).bounds;
    assert!((label.size.height - 12.0 * 1.2).abs() < 0.01, "{label:?}");
}

#[test]
fn each_tone_paints_its_palette_dot() {
    let theme = Theme::dark();
    let app = stage(theme);
    let frame = app.frame_snapshot();
    for dot in [
        "fill 0,0 7x7 r3.5 #4ade80ff",
        "#fbbf24ff",
        "#f87171ff",
        "#8a8a93ff",
    ] {
        assert!(frame.contains(dot), "{dot}\n{frame}");
    }
}

#[test]
fn light_dots_use_the_light_status_colors() {
    let app = stage(Theme::light());
    let frame = app.frame_snapshot();
    for dot in ["#159c4eff", "#b45409ff", "#c22727ff", "#8a8a8fff"] {
        assert!(frame.contains(dot), "{dot}\n{frame}");
    }
}

#[test]
fn the_default_tone_is_neutral() {
    assert_eq!(Tone::default(), Tone::Neutral);
}

#[test]
fn neutral_text_recedes_to_muted_and_other_tones_use_their_own_color() {
    let theme = Theme::dark();
    assert_eq!(Tone::Neutral.text_color(&theme), theme.text.muted);
    for tone in [Tone::Success, Tone::Warning, Tone::Danger] {
        assert_eq!(tone.text_color(&theme), tone.color(&theme));
    }
}

#[test]
fn switching_the_theme_fades_the_dot_instead_of_snapping() {
    let mut app = TestApp::new(component(|cx| {
        let theme = provide_theme(cx, Theme::light());
        vstack((
            status("Connected").tone(Tone::Success),
            button("Dark").on_click(move |cx, _| theme.set(cx, Theme::dark())),
        ))
        .p(24.0)
    }));
    assert!(app.frame_snapshot().contains("#159c4eff"));
    app.click(role(Role::Button).name("Dark"));
    app.advance_ms(16);
    assert!(app.is_animating());
    app.settle();
    let frame = app.frame_snapshot();
    assert!(
        frame.contains("#4ade80ff") && !frame.contains("#159c4eff"),
        "{frame}"
    );
}

#[test]
fn every_tone_composes_into_one_frame() {
    let mut app = stage(Theme::dark());
    insta::assert_snapshot!(app.snapshot());
}
