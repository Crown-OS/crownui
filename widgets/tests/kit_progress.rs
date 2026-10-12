use crowntest::{Role, TestApp, role};
use view::{Listeners, Styled, bind, component};
use widgets::{
    kit::{Theme, button, progress, provide_theme},
    vstack,
};

fn stage(content: impl view::ViewSeq) -> TestApp {
    TestApp::new(component(move |cx| {
        provide_theme(cx, Theme::dark());
        vstack(content).p(24.0).gap(12.0)
    }))
}

#[test]
fn the_bar_is_a_two_hundred_forty_by_eight_gauge_that_does_not_stretch() {
    let mut app = stage(progress(0.5));
    let bar = app.find(role(Role::ProgressBar)).bounds;
    assert_eq!((bar.size.width, bar.size.height), (240.0, 8.0));
}

#[test]
fn the_width_is_overridable() {
    let mut app = stage(progress(0.5).w(320.0));
    assert_eq!(app.find(role(Role::ProgressBar)).bounds.size.width, 320.0);
}

#[test]
fn the_track_is_the_palettes_and_the_fill_stops_where_the_fraction_says() {
    let mut app = stage(progress(0.5));
    let layout = app.layout_snapshot();
    assert!(layout.contains("bg-#3a3a3dff rounded-4"), "{layout}");
    let frame = app.frame_snapshot();
    assert!(
        frame.contains("fill 0,0 120x8 r4 gradient(#8f6dfbff@0,#6d48e8ff@1)"),
        "{frame}"
    );
}

#[test]
fn a_bar_that_cannot_yet_be_its_own_pill_paints_no_fill() {
    let app = stage(progress(7.0 / 240.0));
    assert!(!app.frame_snapshot().contains("gradient"));
}

#[test]
fn nonsense_fractions_are_clamped() {
    for (fraction, fill) in [
        (-1.0, None),
        (2.0, Some("fill 0,0 240x8")),
        (f32::NAN, None),
    ] {
        let app = stage(progress(fraction));
        let frame = app.frame_snapshot();
        match fill {
            Some(fill) => assert!(frame.contains(fill), "{fraction}\n{frame}"),
            None => assert!(!frame.contains("gradient"), "{fraction}\n{frame}"),
        }
    }
}

#[test]
fn the_value_reads_as_a_percentage() {
    let mut app = stage(progress(0.42));
    assert_eq!(
        app.find(role(Role::ProgressBar)).value.as_deref(),
        Some("42%")
    );
}

#[test]
fn a_signal_drives_the_fill_without_animating_it() {
    let mut app = TestApp::new(component(|cx| {
        provide_theme(cx, Theme::dark());
        let fraction = cx.signal(0.25_f32);
        vstack((
            progress(fraction),
            button("Advance").on_click(move |cx, _| fraction.set(cx, 0.75)),
        ))
        .p(24.0)
    }));
    assert!(app.frame_snapshot().contains("fill 0,0 60x8"));
    app.click(role(Role::Button).name("Advance"));
    assert!(!app.is_animating());
    assert!(app.frame_snapshot().contains("fill 0,0 180x8"));
}

#[test]
fn switching_the_theme_fades_the_track_and_repaints_the_accent() {
    let mut app = TestApp::new(component(|cx| {
        let theme = provide_theme(cx, Theme::light());
        vstack((
            progress(bind(|_| 0.5)),
            button("Dark").on_click(move |cx, _| theme.set(cx, Theme::dark())),
        ))
        .p(24.0)
    }));
    assert!(app.layout_snapshot().contains("bg-#e7e5f1ff"));
    app.click(role(Role::Button).name("Dark"));
    app.advance_ms(16);
    assert!(app.is_animating());
    app.settle();
    assert!(app.layout_snapshot().contains("bg-#3a3a3dff"));
}

#[test]
fn a_progress_bar_composes_in_a_column() {
    let mut app = stage((progress(0.0), progress(0.42), progress(1.0).w(320.0)));
    insta::assert_snapshot!(app.snapshot());
}
