use std::time::Instant;

use geometry::Size;
use layouting::{Dimension, FlexDirection, LayoutEngine, LayoutStyle};
use taffy_layout::TaffyEngine;

const ROWS: usize = 100;
const CELLS_PER_ROW: usize = 99;

#[test]
#[ignore = "timing report; run with --ignored --nocapture"]
fn lays_out_ten_thousand_nodes() {
    let mut engine = TaffyEngine::new();
    let cell_style = LayoutStyle {
        flex_grow: 1.0,
        height: Dimension::Length(12.0),
        ..LayoutStyle::default()
    };
    let row_style = LayoutStyle {
        flex_direction: FlexDirection::Row,
        gap: (2.0, 0.0),
        ..LayoutStyle::default()
    };
    let rows: Vec<_> = (0..ROWS)
        .map(|_| {
            let cells: Vec<_> = (0..CELLS_PER_ROW)
                .map(|_| engine.create_node(&cell_style, false))
                .collect();
            let row = engine.create_node(&row_style, false);
            engine.set_children(row, &cells);
            row
        })
        .collect();
    let root = engine.create_node(&LayoutStyle::default(), false);
    engine.set_children(root, &rows);
    let available = Size::new(1920.0, 1080.0);

    let cold_start = Instant::now();
    engine.compute_layout(root, available, |_, _| Size::ZERO);
    let cold = cold_start.elapsed();

    let warm_start = Instant::now();
    engine.compute_layout(root, available, |_, _| Size::ZERO);
    let warm = warm_start.elapsed();

    println!(
        "{} nodes: cold layout {cold:?}, cached relayout {warm:?}",
        engine.node_count()
    );
    assert_eq!(engine.node_count(), ROWS * (CELLS_PER_ROW + 1) + 1);
}
