# taffy-layout

A [`layouting::LayoutEngine`] for CrownUI backed by [taffy](https://docs.rs/taffy)'s
flexbox and grid algorithms.

```rust
use layouting::{LayoutEngine, LayoutStyle, Dimension};
use geometry::Size;
use taffy_layout::TaffyEngine;

let mut engine = TaffyEngine::new();
let root = engine.create_node(
    &LayoutStyle { width: Dimension::Length(200.0), height: Dimension::Length(100.0), ..LayoutStyle::default() },
    false,
);
engine.compute_layout(root, Size::new(200.0, 100.0), |_, _| Size::ZERO);
assert_eq!(engine.layout(root).rect.size, Size::new(200.0, 100.0));
```

- `LayoutStyle` maps one to one onto taffy's `Style`. `Start`/`End` alignment maps to
  taffy's `FlexStart`/`FlexEnd`: they follow the flex direction (a `RowReverse` stack packs
  at its right edge) and behave exactly like `start`/`end` in grids.
- `Dimension::Percent` is a `0.0..=1.0` fraction, which is taffy's convention too.
- `Overflow::Scroll` makes a scroll container: its automatic minimum size is zero, so content
  does not grow it, and `content_size` reports the full extent of its children plus padding,
  so the scroll range is `content_size - rect.size`.
- Measured leaves (`create_node(style, true)`) ask the `compute_layout` closure for their size;
  other leaves size from their style alone.
- Taffy's pixel rounding is off: results stay fractional so the framework can snap to device
  pixels at the real scale factor (`layouting::snap_to_pixels`).
- An infinite available size lays the root out at its max-content size.
