use geometry::{Insets, Rect, Size};
use layouting::{
    Align, Dimension, Display, FlexDirection, GridTrack, Justify, LayoutEngine, LayoutStyle,
    Overflow, Position,
};
use taffy_layout::{TaffyEngine, TaffyNode};

fn sized(width: f32, height: f32) -> LayoutStyle {
    LayoutStyle {
        width: Dimension::Length(width),
        height: Dimension::Length(height),
        ..LayoutStyle::default()
    }
}

fn container(engine: &mut TaffyEngine, style: LayoutStyle, children: &[TaffyNode]) -> TaffyNode {
    let node = engine.create_node(&style, false);
    engine.set_children(node, children);
    node
}

fn leaves(engine: &mut TaffyEngine, style: &LayoutStyle, count: usize) -> Vec<TaffyNode> {
    (0..count)
        .map(|_| engine.create_node(style, false))
        .collect()
}

fn unmeasured(_: TaffyNode, _: layouting::MeasureInput) -> Size {
    Size::ZERO
}

#[test]
fn row_places_children_after_padding_separated_by_gap() {
    let mut engine = TaffyEngine::new();
    let children = leaves(&mut engine, &sized(30.0, 20.0), 3);
    let root = container(
        &mut engine,
        LayoutStyle {
            flex_direction: FlexDirection::Row,
            padding: Insets {
                left: 10.0,
                ..Insets::uniform(5.0)
            },
            gap: (8.0, 0.0),
            align_items: Some(Align::Start),
            ..sized(200.0, 50.0)
        },
        &children,
    );
    engine.compute_layout(root, Size::new(200.0, 50.0), unmeasured);

    let origins: Vec<_> = children
        .iter()
        .map(|&child| engine.layout(child).rect.origin)
        .map(|origin| (origin.x, origin.y))
        .collect();
    assert_eq!(origins, [(10.0, 5.0), (48.0, 5.0), (86.0, 5.0)]);
}

#[test]
fn column_child_with_flex_grow_fills_remaining_height() {
    let mut engine = TaffyEngine::new();
    let header = engine.create_node(&sized(100.0, 40.0), false);
    let body = engine.create_node(
        &LayoutStyle {
            flex_grow: 1.0,
            ..LayoutStyle::default()
        },
        false,
    );
    let root = container(&mut engine, sized(100.0, 300.0), &[header, body]);
    engine.compute_layout(root, Size::new(100.0, 300.0), unmeasured);

    assert_eq!(
        engine.layout(body).rect,
        Rect::from_xywh(0.0, 40.0, 100.0, 260.0)
    );
}

#[test]
fn space_between_pushes_outer_children_to_the_edges() {
    let mut engine = TaffyEngine::new();
    let children = leaves(&mut engine, &sized(20.0, 20.0), 3);
    let root = container(
        &mut engine,
        LayoutStyle {
            flex_direction: FlexDirection::Row,
            justify_content: Some(Justify::SpaceBetween),
            ..sized(100.0, 20.0)
        },
        &children,
    );
    engine.compute_layout(root, Size::new(100.0, 20.0), unmeasured);

    let xs: Vec<_> = children
        .iter()
        .map(|&child| engine.layout(child).rect.min_x())
        .collect();
    assert_eq!(xs, [0.0, 40.0, 80.0]);
}

#[test]
fn align_items_center_centers_children_on_the_cross_axis() {
    let mut engine = TaffyEngine::new();
    let child = engine.create_node(&sized(20.0, 10.0), false);
    let root = container(
        &mut engine,
        LayoutStyle {
            flex_direction: FlexDirection::Row,
            align_items: Some(Align::Center),
            ..sized(100.0, 50.0)
        },
        &[child],
    );
    engine.compute_layout(root, Size::new(100.0, 50.0), unmeasured);

    assert_eq!(
        engine.layout(child).rect,
        Rect::from_xywh(0.0, 20.0, 20.0, 10.0)
    );
}

#[test]
fn end_alignment_follows_reversed_flex_direction() {
    let mut engine = TaffyEngine::new();
    let child = engine.create_node(&sized(20.0, 10.0), false);
    let root = container(
        &mut engine,
        LayoutStyle {
            flex_direction: FlexDirection::RowReverse,
            justify_content: Some(Justify::End),
            ..sized(100.0, 10.0)
        },
        &[child],
    );
    engine.compute_layout(root, Size::new(100.0, 10.0), unmeasured);

    assert_eq!(engine.layout(child).rect.min_x(), 0.0);
}

#[test]
fn grid_splits_fractions_and_spans_both_columns() {
    let mut engine = TaffyEngine::new();
    let first = engine.create_node(&LayoutStyle::default(), false);
    let second = engine.create_node(&LayoutStyle::default(), false);
    let spanning = engine.create_node(
        &LayoutStyle {
            column_span: 2,
            ..LayoutStyle::default()
        },
        false,
    );
    let root = container(
        &mut engine,
        LayoutStyle {
            display: Display::Grid,
            grid_columns: vec![GridTrack::Fraction(1.0), GridTrack::Fraction(2.0)],
            grid_rows: vec![GridTrack::Length(30.0), GridTrack::Length(30.0)],
            ..sized(300.0, 60.0)
        },
        &[first, second, spanning],
    );
    engine.compute_layout(root, Size::new(300.0, 60.0), unmeasured);

    assert_eq!(
        engine.layout(first).rect,
        Rect::from_xywh(0.0, 0.0, 100.0, 30.0)
    );
    assert_eq!(
        engine.layout(second).rect,
        Rect::from_xywh(100.0, 0.0, 200.0, 30.0)
    );
    assert_eq!(
        engine.layout(spanning).rect,
        Rect::from_xywh(0.0, 30.0, 300.0, 30.0)
    );
}

#[test]
fn absolute_child_is_placed_by_inset_from_the_parent_edges() {
    let mut engine = TaffyEngine::new();
    let flowing = engine.create_node(&sized(50.0, 50.0), false);
    let badge = engine.create_node(
        &LayoutStyle {
            position: Position::Absolute,
            inset: [
                Dimension::Length(4.0),
                Dimension::Length(6.0),
                Dimension::Auto,
                Dimension::Auto,
            ],
            ..sized(10.0, 10.0)
        },
        false,
    );
    let root = container(&mut engine, sized(100.0, 100.0), &[flowing, badge]);
    engine.compute_layout(root, Size::new(100.0, 100.0), unmeasured);

    assert_eq!(
        engine.layout(badge).rect,
        Rect::from_xywh(84.0, 4.0, 10.0, 10.0)
    );
    assert_eq!(engine.layout(flowing).rect.origin.y, 0.0);
}

#[test]
fn percent_width_is_a_fraction_of_the_parent() {
    let mut engine = TaffyEngine::new();
    let child = engine.create_node(
        &LayoutStyle {
            width: Dimension::Percent(0.25),
            height: Dimension::Length(10.0),
            ..LayoutStyle::default()
        },
        false,
    );
    let root = container(&mut engine, sized(200.0, 100.0), &[child]);
    engine.compute_layout(root, Size::new(200.0, 100.0), unmeasured);

    assert_eq!(engine.layout(child).rect.size, Size::new(50.0, 10.0));
}

fn wrapping_text(_: TaffyNode, input: layouting::MeasureInput) -> Size {
    const TEXT_WIDTH: f32 = 100.0;
    const LINE_HEIGHT: f32 = 20.0;
    let offered = input
        .known_width
        .or(input.available_width.definite())
        .unwrap_or(TEXT_WIDTH);
    let width = offered.min(TEXT_WIDTH);
    let lines = (TEXT_WIDTH / width).ceil();
    Size::new(width, LINE_HEIGHT * lines)
}

#[test]
fn measured_text_wraps_to_the_width_on_offer() {
    let mut engine = TaffyEngine::new();
    let text = engine.create_node(&LayoutStyle::default(), true);
    let root = container(
        &mut engine,
        LayoutStyle {
            align_items: Some(Align::Start),
            ..sized(40.0, 200.0)
        },
        &[text],
    );
    engine.compute_layout(root, Size::new(40.0, 200.0), wrapping_text);

    assert_eq!(engine.layout(text).rect.size, Size::new(40.0, 60.0));
}

#[test]
fn unmeasured_leaf_ignores_the_measure_closure() {
    let mut engine = TaffyEngine::new();
    let leaf = engine.create_node(&LayoutStyle::default(), false);
    let root = container(
        &mut engine,
        LayoutStyle {
            align_items: Some(Align::Start),
            ..sized(40.0, 200.0)
        },
        &[leaf],
    );
    engine.compute_layout(root, Size::new(40.0, 200.0), |_, _| Size::new(999.0, 999.0));

    assert_eq!(engine.layout(leaf).rect.size, Size::ZERO);
}

#[test]
fn scroll_container_reports_tall_content_without_growing() {
    let mut engine = TaffyEngine::new();
    let tall = engine.create_node(
        &LayoutStyle {
            flex_shrink: 0.0,
            ..sized(100.0, 500.0)
        },
        false,
    );
    let scroller = container(
        &mut engine,
        LayoutStyle {
            overflow: Overflow::Scroll,
            flex_grow: 1.0,
            ..LayoutStyle::default()
        },
        &[tall],
    );
    let root = container(&mut engine, sized(100.0, 200.0), &[scroller]);
    engine.compute_layout(root, Size::new(100.0, 200.0), unmeasured);

    let layout = engine.layout(scroller);
    assert_eq!(layout.rect.size, Size::new(100.0, 200.0));
    assert_eq!(layout.content_size, Size::new(100.0, 500.0));
}

#[test]
fn restyling_a_node_updates_the_next_layout() {
    let mut engine = TaffyEngine::new();
    let child = engine.create_node(&sized(20.0, 20.0), false);
    let root = container(&mut engine, sized(100.0, 100.0), &[child]);
    engine.compute_layout(root, Size::new(100.0, 100.0), unmeasured);
    assert_eq!(engine.layout(child).rect.size, Size::new(20.0, 20.0));

    engine.set_style(child, &sized(60.0, 30.0));
    engine.compute_layout(root, Size::new(100.0, 100.0), unmeasured);

    assert_eq!(engine.layout(child).rect.size, Size::new(60.0, 30.0));
}

#[test]
fn marking_a_measured_leaf_dirty_measures_it_again() {
    let mut engine = TaffyEngine::new();
    let text = engine.create_node(&LayoutStyle::default(), true);
    let root = container(&mut engine, sized(100.0, 100.0), &[text]);
    engine.compute_layout(root, Size::new(100.0, 100.0), |_, _| Size::new(10.0, 10.0));

    engine.mark_dirty(text);
    engine.compute_layout(root, Size::new(100.0, 100.0), |_, _| Size::new(10.0, 30.0));

    assert_eq!(engine.layout(text).rect.size.height, 30.0);
}

#[test]
fn removing_a_child_reflows_its_siblings() {
    let mut engine = TaffyEngine::new();
    let children = leaves(&mut engine, &sized(20.0, 20.0), 2);
    let root = container(&mut engine, sized(100.0, 100.0), &children);
    engine.compute_layout(root, Size::new(100.0, 100.0), unmeasured);
    assert_eq!(engine.layout(children[1]).rect.min_y(), 20.0);

    engine.remove_node(children[0]);
    engine.compute_layout(root, Size::new(100.0, 100.0), unmeasured);

    assert_eq!(engine.layout(children[1]).rect.min_y(), 0.0);
    assert_eq!(engine.node_count(), 2);
}

#[test]
fn fractional_results_are_not_rounded() {
    let mut engine = TaffyEngine::new();
    let children = leaves(
        &mut engine,
        &LayoutStyle {
            flex_grow: 1.0,
            ..LayoutStyle::default()
        },
        3,
    );
    let root = container(
        &mut engine,
        LayoutStyle {
            flex_direction: FlexDirection::Row,
            ..sized(100.0, 10.0)
        },
        &children,
    );
    engine.compute_layout(root, Size::new(100.0, 10.0), unmeasured);

    let width = engine.layout(children[0]).rect.size.width;
    assert!((width - 100.0 / 3.0).abs() < 1e-4, "width was {width}");
}
