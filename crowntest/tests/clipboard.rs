use crowntest::{Role, TestApp, role};
use view::{Listeners, View, bind, component};
use widgets::{button, text, vstack};

const URI_LIST: &str = "text/uri-list";
const GNOME_FILES: &str = "x-special/gnome-copied-files";
const PLAIN: &str = "text/plain;charset=utf-8";

fn copier() -> impl View {
    component(|cx| {
        let pasted = cx.signal(String::from("nothing"));
        vstack((
            text(bind(move |rt| pasted.get(rt))),
            button("copy").on_click(|cx, _| {
                cx.set_clipboard(vec![
                    (URI_LIST.to_owned(), b"file:///a".to_vec()),
                    (GNOME_FILES.to_owned(), b"cut\nfile:///a".to_vec()),
                    (PLAIN.to_owned(), b"/a".to_vec()),
                ]);
            }),
            button("paste").on_click(move |cx, _| {
                cx.read_clipboard(&[GNOME_FILES, URI_LIST], move |cx, data| {
                    let shown = data.map_or_else(
                        || "empty".to_owned(),
                        |data| {
                            format!("{} {}", data.mime_type, String::from_utf8_lossy(&data.data))
                        },
                    );
                    pasted.set(cx, shown);
                });
            }),
        ))
    })
}

fn shown(app: &mut TestApp) -> String {
    app.text_of(role(Role::Text))
}

#[test]
fn copying_offers_every_type_and_pasting_reads_the_first_one_held() {
    let mut app = TestApp::new(copier());
    app.click(role(Role::Button).name("copy"));
    assert_eq!(app.clipboard().len(), 3);
    app.click(role(Role::Button).name("paste"))
        .serve_clipboard();
    assert_eq!(
        shown(&mut app),
        "x-special/gnome-copied-files cut\nfile:///a"
    );
}

#[test]
fn a_read_falls_back_to_a_later_type_and_reports_an_empty_clipboard() {
    let mut app = TestApp::new(copier());
    app.set_clipboard(&[(URI_LIST, b"file:///b")]);
    app.click(role(Role::Button).name("paste"))
        .serve_clipboard();
    assert_eq!(shown(&mut app), "text/uri-list file:///b");
    app.set_clipboard(&[(PLAIN, b"just text")]);
    app.click(role(Role::Button).name("paste"))
        .serve_clipboard();
    assert_eq!(shown(&mut app), "empty");
}
