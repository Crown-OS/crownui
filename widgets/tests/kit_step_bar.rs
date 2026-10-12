use crowntest::{Role, TestApp, role};
use view::{Listeners, Styled, bind, component};
use widgets::{
    kit::{Theme, button, provide_theme, step_bar},
    vstack,
};

fn bar_app(total: usize, current: usize, width: f32) -> TestApp {
    TestApp::new(component(move |cx| {
        provide_theme(cx, Theme::dark());
        vstack(step_bar(total, current)).p(24.0).w(width + 48.0)
    }))
}

fn wipe_offsets(frame: &str) -> Vec<f32> {
    const TRACK_ORIGINS: [f32; 4] = [0.0, 117.5, 235.0, 352.5];
    frame
        .lines()
        .filter_map(|line| line.trim().strip_prefix("layer at ")?.strip_suffix(",0"))
        .filter_map(|offset| offset.parse::<f32>().ok())
        .filter(|offset| !TRACK_ORIGINS.contains(offset))
        .collect()
}

#[test]
fn the_tracks_divide_the_width_evenly_with_ten_pixel_gaps() {
    let mut app = bar_app(4, 0, 460.0);
    let layout = app.layout_snapshot();
    for index in 0..4 {
        let left = 24.0 + index as f32 * 107.5 + index as f32 * 10.0;
        let suffix = if index == 0 {
            String::new()
        } else {
            format!("[{index}]")
        };
        let expected = format!("kit_step_bar > stack{suffix} @{left},24 107.5x5");
        assert!(layout.contains(&expected), "{expected}\n{layout}");
    }
}

#[test]
fn the_bar_fills_its_column_and_is_five_pixels_tall() {
    let mut app = bar_app(4, 0, 460.0);
    let bar = app.find(role(Role::ProgressBar)).bounds;
    assert_eq!((bar.size.width, bar.size.height), (460.0, 5.0));
}

#[test]
fn a_single_step_is_one_full_width_track() {
    let mut app = bar_app(1, 0, 200.0);
    assert!(
        app.layout_snapshot()
            .contains("kit_step_bar > stack @24,24 200x5")
    );
}

#[test]
fn the_current_step_counts_as_reached_and_overshoot_fills_the_bar() {
    for (total, current, step) in [(4, 0, 1), (4, 3, 4), (4, 9, 4)] {
        let mut app = bar_app(total, current, 460.0);
        let value = app.find(role(Role::ProgressBar)).value;
        assert_eq!(value, Some(format!("step {step} of {total}")));
    }
}

#[test]
fn a_bar_with_no_steps_is_empty() {
    let app = bar_app(0, 0, 460.0);
    assert!(!app.frame_snapshot().contains("gradient"));
}

#[test]
fn reached_tracks_paint_the_accent_over_the_whole_track_and_the_rest_stay_clipped_away() {
    let mut app = bar_app(4, 1, 460.0);
    app.settle();
    insta::assert_snapshot!(app.snapshot());
}

#[test]
fn advancing_a_step_wipes_the_new_track_in_with_a_crisp_spring() {
    let mut app = TestApp::new(component(|cx| {
        provide_theme(cx, Theme::dark());
        let step = cx.signal(0_usize);
        vstack((
            step_bar(4, step),
            button("Next").on_click(move |cx, _| step.update(cx, |step| *step += 1)),
        ))
        .p(24.0)
        .w(508.0)
    }));
    app.settle();
    let before = app.frame_snapshot();
    app.click(role(Role::Button).name("Next"));
    app.advance_ms(16);
    assert!(app.is_animating());
    let midway = app.frame_snapshot();
    assert_ne!(before, midway);
    let offsets = wipe_offsets(&midway);
    assert_eq!(offsets.len(), 6, "{midway}");
    assert!(offsets[0] < 0.0 && offsets[0] > -107.5, "{midway}");
    assert_eq!(offsets[0] + offsets[1], 0.0, "{midway}");
    app.settle();
    let after = app.frame_snapshot();
    assert_ne!(midway, after);
    assert!(!app.is_animating());
    assert_eq!(
        app.find(role(Role::ProgressBar)).value.as_deref(),
        Some("step 2 of 4")
    );
}

#[test]
fn going_back_wipes_the_track_out_again() {
    let mut app = TestApp::new(component(|cx| {
        provide_theme(cx, Theme::dark());
        let step = cx.signal(2_usize);
        vstack((
            step_bar(4, bind(move |runtime| step.get(runtime))),
            button("Back").on_click(move |cx, _| step.update(cx, |step| *step -= 1)),
        ))
        .p(24.0)
        .w(508.0)
    }));
    app.settle();
    let before = app.frame_snapshot();
    app.click(role(Role::Button).name("Back"));
    app.settle();
    assert_ne!(before, app.frame_snapshot());
    assert_eq!(
        app.find(role(Role::ProgressBar)).value.as_deref(),
        Some("step 2 of 4")
    );
}

#[test]
fn mounting_does_not_animate() {
    let mut app = bar_app(4, 1, 460.0);
    assert!(!app.is_animating());
    app.advance_ms(16);
    assert!(!app.is_animating());
}
