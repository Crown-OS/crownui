use crowntest::{TestApp, text as text_query};
use view::{Styled, component};
use widgets::{
    kit::{CalloutOptions, Theme, Tone, callout, icons, provide_theme},
    vstack,
};

const LONG_BODY: &str = "Suspend BitLocker before resizing this volume, or Windows will ask for your recovery key on the next boot.";

fn stage(content: impl view::ViewSeq, width: f32) -> TestApp {
    TestApp::new(component(move |cx| {
        provide_theme(cx, Theme::dark());
        vstack(content).p(24.0).w(width).items_start()
    }))
}

#[test]
fn a_title_only_callout_is_a_single_line_in_a_padded_box() {
    let mut app = stage(callout(Tone::Warning, "BitLocker is on"), 800.0);
    let title = app.find(text_query("BitLocker is on")).bounds;
    assert_eq!(title.origin.x, 24.0 + 13.0 + 16.0 + 10.0);
    assert_eq!(title.origin.y, 24.0 + 13.0);
    assert!((title.size.height - 14.0 * 1.2).abs() < 0.01);
}

#[test]
fn the_body_sits_three_pixels_under_the_title_at_thirteen_pixels() {
    let mut app = stage(
        callout(Tone::Neutral, "Heads up").body("Short body."),
        800.0,
    );
    let title = app.find(text_query("Heads up")).bounds;
    let body = app.find(text_query("Short body.")).bounds;
    assert_eq!(body.origin.y, title.origin.y + title.size.height + 3.0);
    assert!((body.size.height - 13.0 * 1.2).abs() < 0.01);
}

#[test]
fn the_body_wraps_inside_a_narrow_column() {
    let mut app = stage(
        callout(Tone::Warning, "BitLocker is on").body(LONG_BODY),
        320.0,
    );
    let body = app.find(text_query(LONG_BODY)).bounds;
    assert!(body.size.height > 2.0 * 13.0 * 1.2, "{body:?}");
    assert!(body.max_x() <= 24.0 + 272.0 - 13.0 + 0.01, "{body:?}");
}

#[test]
fn the_glyph_sits_on_the_titles_line_not_the_blocks_middle() {
    let mut app = stage(
        callout(Tone::Warning, "BitLocker is on").body(LONG_BODY),
        320.0,
    );
    let layout = app.layout_snapshot();
    assert!(
        layout.contains("kit_callout > kit_icon @37,37 16x16"),
        "{layout}"
    );
}

#[test]
fn the_wash_border_and_glyph_come_from_the_tone() {
    let app = stage(callout(Tone::Danger, "Couldn't join network"), 800.0);
    let frame = app.frame_snapshot();
    assert!(frame.contains("r8 #f871711a"), "{frame}");
    assert!(frame.contains("stroke1 0.5,0.5"), "{frame}");
    assert!(frame.contains("#f8717152"), "{frame}");
    assert!(frame.contains("#f87171ff"), "{frame}");
}

#[test]
fn each_tone_picks_its_own_glyph_and_an_explicit_icon_wins() {
    let glyph_of = |callout| {
        let app = stage(callout, 800.0);
        app.frame_snapshot()
    };
    let success = glyph_of(callout(Tone::Success, "Saved"));
    let warning = glyph_of(callout(Tone::Warning, "Saved"));
    let danger = glyph_of(callout(Tone::Danger, "Saved"));
    let neutral = glyph_of(callout(Tone::Neutral, "Saved"));
    let overridden = glyph_of(callout(Tone::Success, "Saved").icon(icons::LOCK));
    let glyphs = [&success, &warning, &danger, &neutral];
    for (index, first) in glyphs.iter().enumerate() {
        let path = |frame: &str| {
            frame
                .lines()
                .find(|line| line.contains("path["))
                .map(str::to_owned)
        };
        for second in &glyphs[index + 1..] {
            assert_ne!(path(first), path(second));
        }
    }
    assert_ne!(overridden, success);
}

#[test]
fn a_callout_composes_every_tone() {
    let mut app = stage(
        vec![
            callout(Tone::Success, "Backup finished").body("Everything was copied."),
            callout(Tone::Warning, "BitLocker is on").body(LONG_BODY),
            callout(Tone::Danger, "Couldn't join network"),
            callout(Tone::Neutral, "Heads up").body("Nothing to do."),
        ],
        360.0,
    );
    insta::assert_snapshot!(app.snapshot());
}
