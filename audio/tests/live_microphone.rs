use std::time::{Duration, Instant};

use audio::{Enhancement, Microphone, MicrophoneOptions, StreamState, sources};

#[test]
#[ignore = "opens the default microphone through PipeWire"]
fn the_default_microphone_delivers_frames_with_increasing_pts() {
    let mut stream =
        Microphone::open(MicrophoneOptions::default().enhancement(Enhancement::NoiseSuppression))
            .expect("open the default microphone");
    let mut frames = stream.take_frames().expect("the ring is handed out once");
    assert!(stream.take_frames().is_none());
    let deadline = Instant::now() + Duration::from_secs(1);
    let mut timestamps = Vec::new();
    while Instant::now() < deadline {
        while let Ok(frame) = frames.pop() {
            timestamps.push(frame.pts);
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    let state = stream.state().borrow().clone();
    println!("{} frames in 1 s, state {state:?}", timestamps.len());
    assert!(
        matches!(state, StreamState::Live { .. } | StreamState::Unenhanced(_)),
        "{state:?}"
    );
    assert!(timestamps.len() > 50, "only {} frames", timestamps.len());
    assert!(timestamps.windows(2).all(|pair| pair[0] < pair[1]));
}

#[test]
#[ignore = "watches the PipeWire registry"]
fn sources_lists_the_default_microphone() {
    let watched = sources();
    let deadline = Instant::now() + Duration::from_secs(1);
    while Instant::now() < deadline && !watched.borrow().iter().any(|source| source.is_default) {
        std::thread::sleep(Duration::from_millis(10));
    }
    let listed = watched.borrow().clone();
    println!("{listed:#?}");
    assert!(listed.iter().any(|source| source.is_default));
    assert!(
        listed
            .iter()
            .all(|source| !source.id.as_str().starts_with("crownos-voice."))
    );
}
