use view::ClipboardRequest;
use windowing::{ClipboardData, WindowEvent};

use graphics::GraphicsBackend;

use crate::TestApp;

/// A system clipboard shared with the app under test.
impl<G: GraphicsBackend> TestApp<G> {
    /// Puts `items` on the clipboard, as another app copying them would.
    pub fn set_clipboard(&mut self, items: &[(&str, &[u8])]) -> &mut Self {
        self.clipboard = items
            .iter()
            .map(|(mime_type, data)| ((*mime_type).to_owned(), data.to_vec()))
            .collect();
        self
    }

    /// What the clipboard holds once the app's pending clipboard requests are served.
    pub fn clipboard(&mut self) -> &[(String, Vec<u8>)] {
        self.serve_clipboard();
        &self.clipboard
    }

    /// Serves the clipboard requests the app made since the last call, as the platform would:
    /// copies replace the clipboard, pastes and reads are answered from it.
    pub fn serve_clipboard(&mut self) -> &mut Self {
        for request in self.ui().take_clipboard_requests() {
            match request {
                ClipboardRequest::Copy(text) => {
                    self.clipboard = vec![(TEXT_MIME_TYPE.to_owned(), text.into_bytes())];
                }
                ClipboardRequest::Offer(items) => self.clipboard = items,
                ClipboardRequest::Paste => {
                    let text = self
                        .clipboard
                        .iter()
                        .find(|(mime_type, _)| mime_type.starts_with("text/plain"))
                        .map(|(_, data)| String::from_utf8_lossy(data).into_owned())
                        .unwrap_or_default();
                    self.deliver(WindowEvent::ClipboardText(text));
                }
                ClipboardRequest::Read(request, mime_types) => {
                    let data = mime_types.into_iter().find_map(|wanted| {
                        let (_, data) = self
                            .clipboard
                            .iter()
                            .find(|(mime_type, _)| *mime_type == wanted)?;
                        Some(ClipboardData {
                            data: data.clone(),
                            mime_type: wanted,
                        })
                    });
                    self.deliver(WindowEvent::ClipboardRead { request, data });
                }
            }
        }
        self
    }
}

const TEXT_MIME_TYPE: &str = "text/plain;charset=utf-8";
