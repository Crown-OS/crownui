use std::{
    fs,
    process::{Child, Command, Stdio},
};

use anyhow::{Context, Result};

use crate::{
    environment::{self, Placement},
    frame_rate::FrameRate,
    framework::Framework,
    paths::Paths,
    scenario::Scenario,
    spec::Spec,
};

/// A CrownUI window on the user's workspace that holds an idle inhibitor while the apps run on
/// the hidden output: crownpositor locks the screen after its idle timeout, which hides every
/// window, and logind inhibitors do not stop it. It runs a copy of the CrownUI app, so the
/// measured app shares no file pages with it and its PSS stays what it would be alone.
pub struct KeepAwake(Child);

impl KeepAwake {
    pub fn start(paths: &Paths, spec: &Spec) -> Result<Self> {
        let app = Framework::CrownUi.executable(paths, FrameRate::Display)?;
        let copy = paths.target.join("keep-awake");
        fs::create_dir_all(&copy).with_context(|| format!("creating {}", copy.display()))?;
        let copy = copy.join("crownui-bench");
        fs::copy(&app, &copy).with_context(|| format!("copying {}", app.display()))?;
        let mut command = Command::new(copy);
        environment::apply(
            &mut command,
            spec,
            Scenario::Hello,
            Placement::Desktop,
            FrameRate::Display,
        );
        command
            .env_remove("WAYLAND_DEBUG")
            .env("BENCH_KEEP_AWAKE", "1")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        let child = command
            .spawn()
            .with_context(|| format!("starting the keep-awake window {command:?}"))?;
        Ok(Self(child))
    }
}

impl Drop for KeepAwake {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}
