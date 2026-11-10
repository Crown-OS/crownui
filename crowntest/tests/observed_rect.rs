use crowntest::{Role, TestApp, role};
use reactive::Signal;
use view::{Cx, Listeners, NodeId, Styled, View, bind, component};
use widgets::{button, hstack, text as label, vstack};

struct ReportLeft<V> {
    view: V,
    left: Signal<f32>,
}

impl<V: View> View for ReportLeft<V> {
    fn build(self, cx: &mut Cx) -> NodeId {
        let node = self.view.build(cx);
        let rect = cx.observe_rect(node);
        let left = self.left;
        cx.effect(move |runtime| {
            let x = rect.get(runtime).min_x();
            left.set(runtime, x);
        });
        node
    }
}

#[test]
fn an_observed_rect_follows_its_node_when_a_sibling_grows() {
    let mut app = TestApp::new(component(|cx| {
        let gap = cx.signal(10.0_f32);
        let left = cx.signal(0.0_f32);
        vstack((
            button("grow").on_click(move |cx, _| gap.set(cx, 30.0)),
            hstack((
                vstack(()).w(bind(move |runtime| gap.get(runtime))),
                ReportLeft {
                    view: label("tracked"),
                    left,
                },
            )),
            label(bind(move |runtime| format!("left {}", left.get(runtime)))),
        ))
    }));
    assert_eq!(app.text_of(role(Role::Text).text("left")), "left 10");
    app.click(role(Role::Button).name("grow"));
    assert_eq!(app.text_of(role(Role::Text).text("left")), "left 30");
}
