# layouting

The layout contract. `LayoutStyle` speaks CSS flexbox/grid (the same vocabulary as the Tailwind
style methods: `p`, `gap`, `flex_row`, `items_center`, `grid_cols` …) and `LayoutEngine` is what
an engine implements (`taffy-layout` is the default).

- Leaves with content (text, images) are *measured*: the engine calls the measure closure passed
  to `compute_layout` with known dimensions and available space.
- Results are parent-relative `NodeLayout`s in logical pixels.
- `mirror_for_rtl` flips a child rect inside its parent for right-to-left layouts; `snap_to_pixels`
  aligns rects to the device grid at fractional scales.

Non-goals: text shaping (the measure closure does it), animation (layout jumps; the render
thread interpolates).
