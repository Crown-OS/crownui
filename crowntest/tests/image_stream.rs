use std::sync::mpsc;

use crowntest::TestApp;
use geometry::Size;
use graphics::{GraphicsError, ImageId, StreamFrame};
use view::{ImageStream, Listeners, PushError, Styled, View, component, show};
use widgets::{button, cover_image, vstack};

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn live_view(streams: mpsc::Sender<ImageStream>) -> impl View {
    component(move |cx| {
        let visible = cx.signal(true);
        let streams = streams.clone();
        let preview = move || {
            let streams = streams.clone();
            component(move |cx| {
                let stream = cx.image_stream();
                let image = stream.image();
                let _ = streams.send(stream);
                cover_image(image, Size::new(4.0, 2.0), 0.0).w(80.0).h(40.0)
            })
        };
        vstack((
            button("hide").on_click(move |cx, _| visible.set(cx, false)),
            show(visible, preview, || vstack(())),
        ))
    })
}

fn frame(width: u32, height: u32) -> Result<StreamFrame, GraphicsError> {
    let rgba = [40, 80, 120, 255].repeat(width as usize * height as usize);
    StreamFrame::new(width, height, rgba)
}

fn image_size(app: &mut TestApp, image: ImageId) -> Option<(u32, u32)> {
    app.inline().renderer().image_size(image)
}

#[test]
fn a_pushed_frame_replaces_the_image_and_redraws_what_shows_it() -> TestResult {
    let (sender, streams) = mpsc::channel();
    let mut app = TestApp::new(live_view(sender));
    let stream = streams.try_recv().expect("the preview made a stream");
    assert_eq!(image_size(&mut app, stream.image()), Some((1, 1)));

    stream.push(frame(4, 2)?)?;
    assert!(app.inline().needs_frame());
    let stats = app.inline().frame()?;

    assert_eq!(image_size(&mut app, stream.image()), Some((4, 2)));
    assert!(stats.recorded >= 1);
    Ok(())
}

#[test]
fn only_the_latest_of_several_frames_is_uploaded() -> TestResult {
    let (sender, streams) = mpsc::channel();
    let mut app = TestApp::new(live_view(sender));
    let stream = streams.try_recv().expect("the preview made a stream");

    stream.push(frame(2, 2)?)?;
    stream.push(frame(8, 4)?)?;
    app.inline().frame()?;

    assert_eq!(image_size(&mut app, stream.image()), Some((8, 4)));
    Ok(())
}

#[test]
fn frames_after_the_preview_unmounts_are_dropped() -> TestResult {
    let (sender, streams) = mpsc::channel();
    let mut app = TestApp::new(live_view(sender));
    let stream = streams.try_recv().expect("the preview made a stream");

    app.click(crowntest::role(semantics::Role::Button).name("hide"));
    app.settle();
    assert_eq!(image_size(&mut app, stream.image()), None);

    stream.push(frame(4, 2)?)?;
    app.inline().frame()?;
    assert_eq!(image_size(&mut app, stream.image()), None);
    Ok(())
}

#[test]
fn a_closed_window_tells_the_producer_to_stop() -> TestResult {
    let (sender, streams) = mpsc::channel();
    let app = TestApp::new(live_view(sender));
    let stream = streams.try_recv().expect("the preview made a stream");
    drop(app);
    assert!(matches!(stream.push(frame(1, 1)?), Err(PushError::Closed)));
    Ok(())
}
