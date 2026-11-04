use windowing::{ClipboardData, ClipboardReadId};

use crate::{Cx, cx::ClipboardRequest};

type OnRead = Box<dyn FnOnce(&mut Cx, Option<ClipboardData>)>;

/// Clipboard reads waiting for the platform's answer.
#[derive(Default)]
pub(crate) struct ClipboardReads {
    next: u64,
    pending: Vec<(ClipboardReadId, OnRead)>,
}

impl Cx {
    /// Puts the same content on the system clipboard in several MIME types, e.g. copied files
    /// as `text/uri-list`, `x-special/gnome-copied-files` and `text/plain;charset=utf-8`. An
    /// item in a text type is also offered under the other text types.
    pub fn set_clipboard(&mut self, items: Vec<(String, Vec<u8>)>) {
        self.clipboard_requests.push(ClipboardRequest::Offer(items));
    }

    /// Reads the system clipboard in the first of `mime_types` it holds, best first, and hands
    /// it to `on_read`; `None` when it holds none of them.
    pub fn read_clipboard(
        &mut self,
        mime_types: &[&str],
        on_read: impl FnOnce(&mut Cx, Option<ClipboardData>) + 'static,
    ) {
        let reads = &mut self.clipboard_reads;
        let request = ClipboardReadId(reads.next);
        reads.next += 1;
        reads.pending.push((request, Box::new(on_read)));
        let wanted = mime_types.iter().map(|mime| (*mime).to_owned()).collect();
        self.clipboard_requests
            .push(ClipboardRequest::Read(request, wanted));
    }

    pub(crate) fn answer_clipboard_read(
        &mut self,
        request: ClipboardReadId,
        data: Option<ClipboardData>,
    ) {
        let pending = &mut self.clipboard_reads.pending;
        let Some(index) = pending.iter().position(|(id, _)| *id == request) else {
            return;
        };
        let (_, on_read) = pending.swap_remove(index);
        on_read(self, data);
        self.apply_pending();
    }
}
