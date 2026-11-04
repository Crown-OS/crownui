use std::{
    os::fd::OwnedFd,
    path::Path,
    time::{Duration, Instant},
};

use audio::{Enhancement, Microphone, MicrophoneOptions, StreamState, sources};
use crownos_ipc::{RemoteError, ServiceBuilder, blocking};
use voice_protocol::{EnhancedNode, JobId, ServiceStatus, StreamId, voice};

const BACKEND: &str = "fake: passthrough";

/// A stand-in crown-voiced that "enhances" by pointing at the raw default source.
struct PassthroughVoice {
    node_name: String,
}

impl PassthroughVoice {
    fn node(&self, enhancement: Enhancement) -> EnhancedNode {
        EnhancedNode {
            stream: StreamId(7),
            node_name: self.node_name.clone(),
            backend: format!("{BACKEND} {enhancement:?}"),
            latency_samples: 0,
        }
    }
}

impl voice::Handler for PassthroughVoice {
    fn open_stream(
        &mut self,
        _source: Option<String>,
        enhancement: Enhancement,
    ) -> Result<EnhancedNode, RemoteError> {
        Ok(self.node(enhancement))
    }

    fn set_enhancement(
        &mut self,
        _stream: StreamId,
        enhancement: Enhancement,
    ) -> Result<EnhancedNode, RemoteError> {
        Ok(self.node(enhancement))
    }

    fn close_stream(&mut self, _stream: StreamId) -> Result<(), RemoteError> {
        Ok(())
    }

    fn enhance_pcm(
        &mut self,
        _enhancement: Enhancement,
        _input: OwnedFd,
        _output: OwnedFd,
    ) -> Result<JobId, RemoteError> {
        Err(RemoteError::handler("offline jobs are not faked"))
    }

    fn cancel_job(&mut self, _job: JobId) -> Result<(), RemoteError> {
        Ok(())
    }

    fn status(&mut self) -> Result<ServiceStatus, RemoteError> {
        Ok(ServiceStatus {
            backends: Vec::new(),
            streams: 1,
            jobs: 0,
        })
    }
}

fn wait_for(state: &tokio::sync::watch::Receiver<StreamState>, wanted: &StreamState) -> bool {
    let deadline = Instant::now() + Duration::from_secs(2);
    while Instant::now() < deadline {
        if *state.borrow() == *wanted {
            return true;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    false
}

fn default_source_name() -> String {
    let watched = sources();
    let deadline = Instant::now() + Duration::from_secs(1);
    loop {
        if let Some(source) = watched.borrow().iter().find(|source| source.is_default) {
            return source.id.as_str().to_owned();
        }
        assert!(Instant::now() < deadline, "no default source");
        std::thread::sleep(Duration::from_millis(10));
    }
}

fn serve_fake_voice(runtime: &Path, node_name: String) -> Result<(), crownos_ipc::Error> {
    std::fs::create_dir_all(runtime.join("crownos"))?;
    let mut server = ServiceBuilder::new(voice::SERVICE)
        .directory(runtime.join("crownos"))
        .build()?;
    std::thread::spawn(move || {
        let mut handler = PassthroughVoice { node_name };
        let _ = blocking::serve(&mut server, |server, peer, message| {
            voice::dispatch(&mut handler, server, peer, message)?;
            Ok(true)
        });
    });
    Ok(())
}

#[test]
#[ignore = "opens the default microphone through PipeWire and a fake crown-voiced"]
fn enhancement_captures_the_node_crown_voiced_returns() {
    let real_runtime = std::env::var("XDG_RUNTIME_DIR").expect("XDG_RUNTIME_DIR");
    let fake_runtime = std::env::temp_dir().join(format!("crownui-audio-{}", std::process::id()));
    // SAFETY: this binary holds a single test and sets the environment before any thread starts.
    unsafe {
        std::env::set_var("PIPEWIRE_RUNTIME_DIR", &real_runtime);
        std::env::set_var("XDG_RUNTIME_DIR", &fake_runtime);
    }
    serve_fake_voice(&fake_runtime, default_source_name()).expect("serve a fake crown-voiced");

    let mut stream =
        Microphone::open(MicrophoneOptions::default().enhancement(Enhancement::StudioVoice))
            .expect("open the microphone");
    let state = stream.state();
    assert!(wait_for(
        &state,
        &StreamState::Live {
            enhanced_by: Some(format!("{BACKEND} StudioVoice"))
        }
    ));
    stream.set_enhancement(Enhancement::Light);
    assert!(wait_for(
        &state,
        &StreamState::Live {
            enhanced_by: Some(format!("{BACKEND} Light"))
        }
    ));
    stream.set_enhancement(Enhancement::Off);
    assert!(wait_for(&state, &StreamState::Live { enhanced_by: None }));

    let mut frames = stream.take_frames().expect("frames");
    std::thread::sleep(Duration::from_millis(200));
    assert!(frames.pop().is_ok());
    drop(stream);
    let _ = std::fs::remove_dir_all(fake_runtime);
}
