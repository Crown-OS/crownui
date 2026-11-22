use std::time::{Duration, Instant};

use camera::{Camera, CameraOptions, CameraState, cameras};

#[test]
#[ignore = "opens the default camera through PipeWire; frames stay in memory"]
fn the_default_camera_previews_and_taps_frames_with_increasing_pts() {
    let (previewed, previews) = crossbeam_channel::unbounded();
    let stream = Camera::open(CameraOptions::default(), move |frame| {
        let _ = previewed.send((frame.pts(), frame.width(), frame.height()));
    })
    .expect("open the default camera");
    let tap = stream.tap(8);
    let deadline = Instant::now() + Duration::from_secs(3);
    let mut tapped = 0;
    while Instant::now() < deadline {
        if tap.recv_timeout(Duration::from_millis(100)).is_ok() {
            tapped += 1;
        }
    }
    let state = stream.state().borrow().clone();
    drop(stream);
    let frames: Vec<_> = previews.try_iter().collect();
    println!(
        "{} previewed, {tapped} tapped in 3 s, state {state:?}",
        frames.len()
    );
    assert!(matches!(state, CameraState::Live(_)), "{state:?}");
    assert!(frames.len() > 10, "only {} frames", frames.len());
    assert!(tapped > 0);
    assert!(frames.windows(2).all(|pair| pair[0].0 < pair[1].0));
    let (_, width, height) = frames[0];
    assert!(width > 0 && height > 0);
}

#[test]
#[ignore = "watches the PipeWire registry"]
fn cameras_lists_what_is_plugged_in() {
    let watched = cameras();
    let deadline = Instant::now() + Duration::from_secs(1);
    while Instant::now() < deadline && watched.borrow().is_empty() {
        std::thread::sleep(Duration::from_millis(10));
    }
    let listed = watched.borrow().clone();
    println!("{listed:#?}");
    assert!(!listed.is_empty());
}
