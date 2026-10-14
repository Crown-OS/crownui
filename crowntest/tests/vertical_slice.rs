use crowntest::{Role, TestApp, role, text};
use geometry::rgb;
use motion::Spring;
use view::{
    ColorScheme, Listeners, Semantic, Styled, View, bind, component, error_boundary, for_each,
    keyed, show,
};
use widgets::{button, hstack, image, scroll_view, text as label, text_field, toggle, vstack};

fn counter() -> impl View {
    component(|cx| {
        let count = cx.signal(0);
        vstack((
            label(bind(move |rt| format!("Count: {}", count.get(rt)))).text_size(20.0),
            hstack((
                button("-1").on_click(move |cx, _| count.update(cx, |value| *value -= 1)),
                button("+1").on_click(move |cx, _| count.update(cx, |value| *value += 1)),
            ))
            .gap(8.0),
        ))
        .p(16.0)
        .gap(12.0)
    })
}

#[test]
fn clicking_buttons_updates_bound_text() {
    let mut app = TestApp::new(counter());
    assert_eq!(app.text_of(role(Role::Text).text("Count")), "Count: 0");
    app.click(role(Role::Button).name("+1"));
    app.click(role(Role::Button).name("+1"));
    app.click(role(Role::Button).name("-1"));
    assert_eq!(app.text_of(role(Role::Text).text("Count")), "Count: 1");
    assert!(
        app.frame_snapshot().contains("text \"Count: 1\""),
        "{}",
        app.frame_snapshot()
    );
}

#[test]
fn real_pointer_clicks_go_through_the_gesture_arena() {
    let mut app = TestApp::new(counter());
    let plus = app.find(role(Role::Button).name("+1")).bounds;
    let center = plus.center();
    app.pointer_down(center.x, center.y)
        .advance_ms(50)
        .pointer_up(center.x, center.y);
    assert_eq!(app.text_of(role(Role::Text).text("Count")), "Count: 1");
}

#[derive(Clone, PartialEq)]
struct Todo {
    id: u32,
    title: String,
}

fn todo_app() -> impl View {
    component(|cx| {
        let todos = cx.signal(Vec::<Todo>::new());
        let draft = cx.signal(String::new());
        let next_id = cx.signal(1_u32);
        let add = move |cx: &mut view::Cx| {
            let title = draft.get(cx);
            if title.trim().is_empty() {
                return;
            }
            let id = next_id.get(cx);
            next_id.set(cx, id + 1);
            todos.update(cx, |list| list.push(Todo { id, title }));
            draft.set(cx, String::new());
        };
        vstack((
            hstack((
                text_field(draft, "What needs doing?")
                    .grow(1.0)
                    .label("New todo")
                    .on_submit(move |cx, _| add(cx)),
                button("Add").on_click(move |cx, _| add(cx)),
            ))
            .gap(8.0),
            for_each(
                todos,
                |todo: &Todo| todo.id,
                move |_cx, todo| {
                    hstack((
                        label(bind(move |rt| todo.with(rt, |todo| todo.title.clone()))).grow(1.0),
                        button("Remove")
                            .label(bind(move |rt| {
                                todo.with(rt, |todo| format!("Remove {}", todo.title))
                            }))
                            .on_click(move |cx, _| {
                                let id = todo.with(cx, |todo| todo.id);
                                todos.update(cx, |list| list.retain(|item| item.id != id));
                            }),
                    ))
                    .gap(8.0)
                },
            )
            .gap(4.0),
            label(bind(move |rt| format!("{} left", todos.with(rt, Vec::len)))),
        ))
        .p(16.0)
        .gap(12.0)
    })
}

#[test]
fn todos_can_be_typed_added_and_removed() {
    let mut app = TestApp::new(todo_app());
    app.click(role(Role::TextField).name("New todo"))
        .type_text("milk");
    app.click(role(Role::Button).name("Add"));
    app.click(role(Role::TextField).name("New todo"))
        .type_text("eggs")
        .press_key("Enter");
    assert_eq!(app.text_of(text("left")), "2 left");
    assert!(app.exists(role(Role::Text).name("milk")));
    app.click(role(Role::Button).name("Remove milk"));
    assert_eq!(app.text_of(text("left")), "1 left");
    assert!(!app.exists(role(Role::Text).name("milk")));
    assert!(app.exists(role(Role::Text).name("eggs")));
}

#[test]
fn toggle_knob_springs_on_the_render_thread() {
    let mut app = TestApp::new(component(|cx| {
        let on = cx.signal(false);
        hstack((label("Wi-Fi"), toggle(on).label("Wi-Fi")))
            .gap(8.0)
            .p(8.0)
    }));
    app.click(role(Role::Switch).name("Wi-Fi"));
    assert_eq!(app.find(role(Role::Switch)).states.checked, Some(true));
    assert!(
        app.is_animating(),
        "the knob should be mid-spring right after the click"
    );
    app.advance_ms(50);
    assert!(app.is_animating());
    app.settle();
    assert!(!app.is_animating());
}

#[test]
fn hover_and_dark_variants_restyle_without_rebuilding() {
    let mut app = TestApp::new(
        button("Hover me")
            .bg(rgb(0x222222))
            .hover(|style| style.bg(rgb(0x444444)))
            .dark(|style| style.text_color(rgb(0xffffff)))
            .transition(Spring::crisp()),
    );
    app.hover(role(Role::Button));
    app.settle();
    assert!(
        app.frame_snapshot().contains("#444444ff"),
        "{}",
        app.frame_snapshot()
    );
    app.set_color_scheme(ColorScheme::Dark);
    app.settle();
    assert!(app.frame_snapshot().contains("text \"Hover me\""));
}

#[test]
fn show_swaps_branches_and_frees_the_hidden_one() {
    let mut app = TestApp::new(component(|cx| {
        let open = cx.signal(false);
        vstack((
            button("Toggle details").on_click(move |cx, _| open.update(cx, |open| *open = !*open)),
            show(open, || label("Details"), || label("Collapsed")),
        ))
    }));
    assert!(app.exists(role(Role::Text).name("Collapsed")));
    let nodes_before = app.ui().node_count();
    app.click(role(Role::Button).name("Toggle details"));
    assert!(app.exists(role(Role::Text).name("Details")));
    assert!(!app.exists(role(Role::Text).name("Collapsed")));
    assert_eq!(app.ui().node_count(), nodes_before);
}

#[test]
fn scroll_views_clamp_their_offset() {
    let rows: Vec<_> = (0..50)
        .map(|index| label(format!("Row {index}")).h(20.0))
        .collect();
    let mut app = TestApp::with_size(
        scroll_view(vstack(rows))
            .h(100.0)
            .label("List")
            .role(Role::ScrollView),
        300.0,
        200.0,
    );
    app.scroll(role(Role::ScrollView), 10_000.0);
    let last = app.find(role(Role::Text).name("Row 49")).bounds;
    assert!(
        last.max_y() <= 100.5 && last.max_y() >= 99.5,
        "last row should end at the bottom edge: {last:?}"
    );
}

#[test]
fn error_boundaries_replace_a_panicking_subtree() {
    let mut app = TestApp::new(component(|cx| {
        let explode = cx.signal(false);
        vstack((
            button("Break").on_click(move |cx, _| explode.set(cx, true)),
            error_boundary(
                move || {
                    label(bind(move |rt| {
                        if explode.get(rt) {
                            panic!("boom");
                        }
                        "Fine".to_string()
                    }))
                },
                |message| label(format!("Recovered: {message}")),
            ),
        ))
    }));
    assert!(app.exists(role(Role::Text).name("Fine")));
    app.click(role(Role::Button).name("Break"));
    assert!(app.exists(role(Role::Text).name("Recovered: boom")));
}

#[test]
fn counter_snapshot_is_readable_text() {
    let mut app = TestApp::new(counter());
    app.click(role(Role::Button).name("+1"));
    insta::assert_snapshot!(app.snapshot());
}

#[test]
fn lazy_lists_mount_only_visible_rows() {
    use widgets::lazy_vstack;
    let mut app = TestApp::with_size(
        component(|cx| {
            let items = cx.signal((0..10_000).collect::<Vec<u32>>());
            lazy_vstack(
                items,
                |item: &u32| *item,
                20.0,
                |_cx, item| label(bind(move |rt| format!("Item {}", item.get(rt)))),
            )
            .h(200.0)
        }),
        300.0,
        200.0,
    );
    app.settle();
    let mounted = app.count(role(Role::Text).text("Item"));
    assert!(
        mounted > 5 && mounted < 30,
        "only visible rows plus overscan should mount, got {mounted}"
    );
    assert!(app.exists(role(Role::Text).name("Item 0")));
    app.scroll(role(Role::ScrollView), 1_000_000.0);
    app.settle();
    assert!(
        app.exists(role(Role::Text).name("Item 9999")),
        "{}",
        app.semantics_snapshot()
    );
    assert!(!app.exists(role(Role::Text).name("Item 0")));
    assert!(app.ui().node_count() < 100);
}

#[test]
fn components_expose_tools_to_agents() {
    use view::AppTool;
    let mut app = TestApp::new(component(|cx| {
        let count = cx.signal(0_i64);
        cx.register_tool(AppTool::new(
            "add",
            "Adds a number to the counter",
            r#"{"type":"object","properties":{"amount":{"type":"integer"}}}"#,
            move |cx, arguments| {
                let amount: i64 = arguments
                    .split(|character: char| !character.is_ascii_digit() && character != '-')
                    .find_map(|part| part.parse().ok())
                    .unwrap_or(0);
                count.update(cx, |value| *value += amount);
                Ok(format!("{{\"count\":{}}}", count.get(cx)))
            },
        ));
        label(bind(move |rt| format!("Total {}", count.get(rt))))
    }));
    let tools = app.ui().tools();
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0].name, "add");
    let result = app.ui().call_tool("add", r#"{"amount":5}"#);
    assert_eq!(result, Ok("{\"count\":5}".to_string()));
    app.settle();
    assert!(app.exists(role(Role::Text).name("Total 5")));
}

#[test]
fn popovers_open_below_their_anchor_and_close_on_escape() {
    use widgets::popover;
    let mut app = TestApp::new(component(|cx| {
        let open = cx.signal(false);
        vstack((popover(
            button("Menu").on_click(move |cx, _| open.update(cx, |open| *open = !*open)),
            open,
            || vstack((label("Rename"), label("Delete"))).p(8.0),
        ),))
        .p(16.0)
    }));
    assert!(!app.exists(role(Role::Text).name("Rename")));
    app.click(role(Role::Button).name("Menu"));
    let anchor = app.find(role(Role::Button).name("Menu")).bounds;
    let item = app.find(role(Role::Text).name("Rename")).bounds;
    assert!(
        item.min_y() >= anchor.max_y(),
        "menu should sit below its anchor: {anchor:?} {item:?}"
    );
    app.press_key("Escape");
    assert!(
        !app.exists(role(Role::Text).name("Rename")),
        "{}",
        app.semantics_snapshot()
    );
}

#[test]
fn materials_fall_back_to_in_app_backdrop_blur() {
    use windowing::Material;
    let mut app = TestApp::new(vstack((label("Glass"),)).p(24.0).material(Material::thin()));
    let surface = app.inline().last_surface().cloned();
    let shapes = surface
        .and_then(|state| state.material_shapes)
        .unwrap_or_default();
    assert_eq!(
        shapes.len(),
        1,
        "the compositor receives the material shape"
    );
    assert!(!app.frame_snapshot().contains("backdrop-blur"));
    app.ui().set_emulate_materials(true);
    app.settle();
    assert!(
        app.frame_snapshot().contains("backdrop-blur24"),
        "{}",
        app.frame_snapshot()
    );
}

#[test]
fn with_animation_animates_the_layout_it_causes() {
    let mut app = TestApp::new(component(|cx| {
        let wide = cx.signal(false);
        vstack((
            button("Grow").on_click(move |cx, _| {
                cx.with_animation(Spring::smooth(), |cx| wide.set(cx, true))
            }),
            label("Box")
                .w(bind(move |rt| if wide.get(rt) { 300.0 } else { 100.0 }))
                .h(20.0),
        ))
    }));
    app.click(role(Role::Button).name("Grow"));
    assert!(app.is_animating(), "the width change should animate");
    app.settle();
    assert!(!app.is_animating());
}

#[test]
fn right_to_left_mirrors_rows() {
    let mut app = TestApp::with_size(
        hstack((button("First"), button("Second"))).gap(8.0),
        400.0,
        100.0,
    );
    let first_ltr = app.find(role(Role::Button).name("First")).bounds;
    app.ui().set_right_to_left(true);
    app.settle();
    let first_rtl = app.find(role(Role::Button).name("First")).bounds;
    let second_rtl = app.find(role(Role::Button).name("Second")).bounds;
    assert_eq!(first_ltr.min_x(), 0.0);
    assert!(
        (first_rtl.max_x() - 400.0).abs() < 0.01,
        "first item hugs the right edge: {first_rtl:?}"
    );
    assert!(second_rtl.max_x() <= first_rtl.min_x());
}

#[test]
fn exit_animations_keep_nodes_drawn_until_they_finish() {
    let mut app = TestApp::new(component(|cx| {
        let visible = cx.signal(true);
        vstack((
            button("Dismiss").on_click(move |cx, _| visible.set(cx, false)),
            show(
                visible,
                || {
                    label("Toast")
                        .exit(|style| style.opacity(0.0))
                        .transition(Spring::smooth())
                },
                || vstack(()),
            ),
        ))
    }));
    app.click(role(Role::Button).name("Dismiss"));
    assert!(
        !app.exists(role(Role::Text).name("Toast")),
        "gone from the semantic tree immediately"
    );
    assert!(
        app.frame_snapshot().contains("Toast"),
        "still drawn while fading out"
    );
    app.settle();
    assert!(
        !app.frame_snapshot().contains("Toast"),
        "{}",
        app.frame_snapshot()
    );
}

#[test]
fn text_fields_edit_at_the_caret() {
    let mut app = TestApp::new(component(|cx| {
        let value = cx.signal(String::new());
        vstack((
            text_field(value, "Name").label("Name"),
            label(bind(move |rt| format!("Value: {}", value.get(rt)))),
        ))
    }));
    app.click(role(Role::TextField).name("Name"))
        .type_text("helo");
    app.press_key("ArrowLeft").type_text("l");
    app.press_key("Home").press_key("Delete").type_text("H");
    app.press_key("End").press_key("Backspace");
    assert_eq!(app.text_of(text("Value:")), "Value: Hell");
}

#[test]
fn variants_follow_bound_values() {
    let mut app = TestApp::new(component(|cx| {
        let accent = cx.signal(rgb(0x111111));
        vstack((
            button("Restyle").on_click(move |cx, _| accent.set(cx, rgb(0x00ff00))),
            button("Target").hover(move |style| style.bg(bind(move |rt| accent.get(rt)))),
        ))
    }));
    app.hover(role(Role::Button).name("Target")).settle();
    assert!(
        app.frame_snapshot().contains("#111111ff"),
        "{}",
        app.frame_snapshot()
    );
    app.click(role(Role::Button).name("Restyle"));
    app.hover(role(Role::Button).name("Target")).settle();
    assert!(
        app.frame_snapshot().contains("#00ff00ff"),
        "{}",
        app.frame_snapshot()
    );
}

#[test]
fn text_fields_advertise_set_value_and_expose_their_value() {
    let mut app = TestApp::new(component(|cx| {
        let value = cx.signal(String::from("Ada"));
        text_field(value, "Name").label("Name")
    }));
    let field = app.find(role(Role::TextField).name("Name"));
    assert!(field.supports(semantics::ActionKind::SetValue));
    assert_eq!(field.value.as_deref(), Some("Ada"));
    assert!(
        field.children.is_empty(),
        "text fields are atomic for assistive technology"
    );
}

#[test]
fn keyed_rebuilds_only_when_its_key_changes() {
    let mut app = TestApp::new(component(|cx| {
        let count = cx.signal(0_u32);
        let builds = cx.store(0_u32);
        vstack((
            button("Add").on_click(move |cx, _| count.update(cx, |count| *count += 1)),
            keyed(
                move |rt| count.get(rt) >= 2,
                move |cx, many| {
                    builds.update(cx, |builds| *builds += 1);
                    let total = builds.with(cx, |builds| *builds);
                    label(format!(
                        "{} build {total}",
                        if many { "Many" } else { "Few" }
                    ))
                },
            ),
        ))
    }));
    assert!(app.exists(role(Role::Text).name("Few build 1")));
    app.click(role(Role::Button).name("Add"));
    assert!(
        app.exists(role(Role::Text).name("Few build 1")),
        "same key, no rebuild"
    );
    app.click(role(Role::Button).name("Add"));
    assert!(app.exists(role(Role::Text).name("Many build 2")));
}

#[test]
fn unmounted_images_are_freed_from_the_renderer() {
    let mut app = TestApp::new(component(|cx| {
        let visible = cx.signal(true);
        vstack((
            button("Hide").on_click(move |cx, _| visible.set(cx, false)),
            show(visible, || image(1, 1, vec![255; 4]), || vstack(())),
        ))
    }));
    assert_eq!(app.inline().renderer().image_count(), 1);
    app.click(role(Role::Button).name("Hide"));
    app.settle();
    assert_eq!(app.inline().renderer().image_count(), 0);
}

#[test]
fn a_collapsed_branch_leaves_no_gap() {
    let mut app = TestApp::new(component(|cx| {
        let open = cx.signal(false);
        vstack((
            button("Open").on_click(move |cx, _| open.set(cx, true)),
            show(open, || label("Middle"), || vstack(()).display_none()),
            label("Last"),
        ))
        .gap(10.0)
    }));
    let button_bottom = app.find(role(Role::Button).name("Open")).bounds.max_y();
    let last_top = app.find(role(Role::Text).name("Last")).bounds.min_y();
    assert!(
        (last_top - button_bottom - 10.0).abs() < 0.5,
        "one gap, not two: {button_bottom} → {last_top}"
    );
    app.click(role(Role::Button).name("Open"));
    let middle = app.find(role(Role::Text).name("Middle")).bounds;
    assert!((middle.min_y() - button_bottom - 10.0).abs() < 0.5);
}
