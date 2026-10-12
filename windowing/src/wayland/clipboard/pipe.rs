use std::{
    fs::File,
    io::{ErrorKind, Read, Write},
    os::fd::OwnedFd,
};

use sctk::{
    data_device_manager::{ReadPipe, WritePipe},
    reexports::calloop::PostAction,
};

use crate::wayland::event_loop::WaylandPlatform;

const READ_CHUNK_BYTES: usize = 16 * 1024;

impl WaylandPlatform {
    /// Reads `pipe` to its end without blocking the loop, then hands the bytes to `done`.
    /// Returns `false`, without calling `done`, when the pipe cannot be watched.
    pub(crate) fn read_pipe(
        &mut self,
        pipe: ReadPipe,
        done: impl FnOnce(&mut Self, Vec<u8>) + 'static,
    ) -> bool {
        let mut contents = Vec::new();
        let mut done = Some(done);
        let inserted = self.loop_handle.insert_source(pipe, move |_, file, state| {
            let mut chunk = [0; READ_CHUNK_BYTES];
            let mut reader: &File = file;
            match reader.read(&mut chunk) {
                Err(error)
                    if matches!(error.kind(), ErrorKind::Interrupted | ErrorKind::WouldBlock) =>
                {
                    PostAction::Continue
                }
                Ok(0) | Err(_) => {
                    if let Some(done) = done.take() {
                        done(state, std::mem::take(&mut contents));
                    }
                    PostAction::Remove
                }
                Ok(read) => {
                    contents.extend_from_slice(&chunk[..read]);
                    PostAction::Continue
                }
            }
        });
        inserted.is_ok()
    }
}

/// Writes `bytes` into `pipe` from a short-lived thread, so a slow reader never stalls the loop.
pub(crate) fn write_in_background(pipe: WritePipe, bytes: Vec<u8>) {
    let mut file = File::from(OwnedFd::from(pipe));
    let _ = std::thread::Builder::new()
        .name("clipboard-writer".to_owned())
        .spawn(move || file.write_all(&bytes));
}
