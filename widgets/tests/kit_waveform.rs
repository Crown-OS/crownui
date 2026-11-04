use crowntest::{Role, TestApp, role, text as text_query};
use geometry::Color;
use view::{Listeners, Semantic, Styled, bind, component};
use widgets::{
    kit::{
        LevelHistory, Theme, WaveformOptions, WaveformPeaks, button, live_waveform, provide_theme,
        waveform,
    },
    text, vstack,
};

const LEFT: f32 = 24.0;
const WIDTH: f32 = 432.0;
const MIDDLE: f32 = 24.0 + 24.0;
const BARS: usize = 86;

fn stage(content: impl view::ViewSeq) -> TestApp {
    TestApp::with_size(
        component(move |cx| {
            provide_theme(cx, Theme::dark());
            vstack(content).p(24.0).gap(12.0)
        }),
        480.0,
        320.0,
    )
}

fn hex(color: Color) -> String {
    let [red, green, blue, _] = color.to_rgba8();
    format!("#{red:02x}{green:02x}{blue:02x}")
}

fn ramp() -> WaveformPeaks {
    WaveformPeaks::new((0..=255).collect::<Vec<u8>>())
}

fn seekable() -> TestApp {
    stage(component(|cx| {
        let position = cx.signal(0.25_f32);
        vstack((
            waveform(ramp(), position).on_seeking(move |cx, to| position.set(cx, to)),
            text(bind(move |runtime| format!("At {}", position.get(runtime)))),
        ))
        .gap(12.0)
    }))
}

fn position_of(app: &mut TestApp) -> f32 {
    let label = app.text_of(text_query("At"));
    let Ok(position) = label.trim_start_matches("At ").parse() else {
        panic!("unreadable position {label:?}");
    };
    position
}

fn live() -> TestApp {
    stage(component(|cx| {
        let history = cx.signal(LevelHistory::new());
        vstack((
            live_waveform(history),
            button("Loud").on_click(move |cx, _| history.update(cx, |history| history.push(1.0))),
        ))
        .gap(12.0)
    }))
}

#[test]
fn both_waveforms_fill_their_column_at_forty_eight_tall() {
    let mut app = stage(component(|cx| {
        let history = cx.signal(LevelHistory::new());
        vstack((
            waveform(ramp(), 0.0).label("Take"),
            live_waveform(history).label("Input"),
        ))
    }));
    for (kind, name) in [(Role::ProgressBar, "Take"), (Role::Image, "Input")] {
        let bounds = app.find(role(kind).name(name)).bounds;
        assert_eq!(
            (bounds.size.width, bounds.size.height),
            (WIDTH, 48.0),
            "{name}"
        );
    }
}

#[test]
fn a_recording_fits_its_peaks_to_the_width_and_colors_what_has_played() {
    let theme = Theme::dark();
    let app = stage(waveform(ramp(), 0.5));
    let frame = app.frame_snapshot();
    let muted = frame.matches(&hex(theme.text.muted)).count();
    let accent = frame.matches(&hex(theme.accent.start)).count();
    assert_eq!(muted, BARS / 2, "{frame}");
    assert!(accent > 0, "{frame}");
    assert!(frame.contains("fill 429,0 3x48 r1.5"), "{frame}");
    assert!(
        frame.contains(&format!("fill 215,0 2x48 r1 {}", hex(theme.text.primary))),
        "{frame}"
    );
}

#[test]
fn an_unseekable_recording_reads_as_progress() {
    let mut app = stage(waveform(ramp(), 0.42));
    let node = app.find(role(Role::ProgressBar));
    assert_eq!(node.value.as_deref(), Some("42%"));
    assert!(!node.states.focusable);
}

#[test]
fn clicking_and_dragging_seek_to_the_pointer_and_clamp_at_the_ends() {
    let mut app = seekable();
    assert_eq!(app.find(role(Role::Slider)).value.as_deref(), Some("25%"));
    app.pointer_down(LEFT + WIDTH / 2.0, MIDDLE);
    app.pointer_up(LEFT + WIDTH / 2.0, MIDDLE);
    assert_eq!(position_of(&mut app), 0.5);
    app.drag(role(Role::Slider), LEFT + WIDTH * 0.75, MIDDLE);
    assert_eq!(position_of(&mut app), 0.75);
    app.drag(role(Role::Slider), 2000.0, MIDDLE);
    assert_eq!(position_of(&mut app), 1.0);
    assert_eq!(app.find(role(Role::Slider)).value.as_deref(), Some("100%"));
}

#[test]
fn arrows_home_and_end_seek_a_focused_waveform() {
    let mut app = seekable();
    app.focus(role(Role::Slider));
    app.press_key("ArrowRight");
    assert!((position_of(&mut app) - 0.3).abs() < 1e-6);
    app.press_key("End");
    assert_eq!(position_of(&mut app), 1.0);
    app.press_key("ArrowRight");
    assert_eq!(position_of(&mut app), 1.0);
    app.press_key("Home");
    assert_eq!(position_of(&mut app), 0.0);
}

#[test]
fn the_newest_level_enters_at_the_right_edge_with_a_fast_attack_and_nothing_animates() {
    let mut app = live();
    let quiet = app.frame_snapshot();
    assert_eq!(quiet.matches("3x3 r1.5").count(), BARS, "{quiet}");
    app.click(role(Role::Button).name("Loud"));
    let frame = app.frame_snapshot();
    assert!(frame.contains("fill 429,13.5 3x21 r1.5"), "{frame}");
    assert!(frame.contains("fill 424,22.5 3x3 r1.5"), "{frame}");
    app.click(role(Role::Button).name("Loud"));
    let frame = app.frame_snapshot();
    assert!(frame.contains("fill 424,13.5 3x21 r1.5"), "{frame}");
    assert!(!app.is_animating());
}

#[test]
fn waveforms_compose_in_a_column() {
    let mut app = stage(component(|cx| {
        let mut levels = LevelHistory::new();
        (0..40).for_each(|step| levels.push(if step % 8 < 3 { 0.9 } else { 0.1 }));
        let history = cx.signal(levels);
        vstack((
            waveform(WaveformPeaks::from_levels(&[0.2, 0.9, 0.4, 1.0, 0.1]), 0.3),
            live_waveform(history),
        ))
        .gap(12.0)
    }));
    insta::assert_snapshot!(app.snapshot());
}
