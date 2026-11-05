# CrownUI perf observations (from reading the code, branch files-app)

## UI thread, per frame (`WindowRunner::push_frame` -> `build_batch` -> `Ui::update/layout/take_frame`)
- `Ui::needs_frame` (view/src/ui.rs:309) scans every node's `dirty` flags: O(n) per `about_to_wait`/`proxy_wake_up`.
- `layout_pass` (ui.rs:349) ends with another O(n) scan for `dirty.style || dirty.children`.
- `ElementTree::sync_styles` (view/src/layout.rs:433) collects all NodeIds into a Vec and walks every node every layout pass (twice).
- `ElementTree::write_back` (layout.rs:489) walks the whole tree every layout pass, calls `engine.layout()` per node; every descendant of a scroll container gets `dirty.frame=true` on scroll because `window_rect` moves -> `outcome.frames_changed` -> `semantics_dirty`.
- `scroll_nearest` (view/src/dispatch.rs:564) sets `layout_needed=true` + dirty.frame on container & children: every wheel tick = full taffy compute_layout + full write_back + full semantics rebuild + `RenderOp::Visual` (content_offset) -> render thread re-records the container boundary (RenderOp::Visual sets needs_record unconditionally, tree.rs:365-371).
- `collect_render_ops` (view/src/render_op.rs:949) walks the whole tree every frame; clones `element.children` Vec per node (line 1042) and clones the canvas `DrawList` (line 1034) when content dirty.
- `material_shapes` (render_op.rs:1058) iterates all nodes every `take_frame`.
- `refresh_semantics` (ui.rs:452) rebuilds the full SemanticTree (`semantics_build::rebuild` allocates `path()` String per node, clones names), rebuilds `semantic_index` HashMap over all nodes, collects + sorts focusables. Triggered after ANY event (handle_event sets semantics_dirty=true) and after any ops.
- `key_down` (dispatch.rs:604) scans every node for a shortcut on every key press.
- `assign_semantic_ids` (cx.rs) walks the whole tree each take_frame, SipHash per node.

## Hit testing (dispatch.rs / tree.rs)
- `hit_test` (tree.rs:192): `top_layers` walks the whole tree; `hit_test_node` allocates a `Vec<NodeId>` and sorts by z_index at every visited node. `pointer_move` calls `hit_test` twice (update_hover + PointerMove dispatch). No cached hitbox list, no bounds tree.
- `bubble`/`dispatch_along` collects ancestors into a Vec; listeners are `mem::take`n and restored per node.

## Styles / variants
- `refresh_visual` (cx.rs:~700) clones the whole `NodeStyle` (LayoutStyle incl. Vec<GridTrack>, TextOverrides with Option<String>) on every hover/press/focus change, applies variant props, compares.
- `apply_style` with text impact does `mark_text_subtree` walk.

## Text (view/src/text_cache.rs, impeller/src/text/*)
- `TextCache::key` hashes the full text with SipHash (`DefaultHasher`) and `TextStyle::shaping_key` hashes 11 fields with SipHash on every `measure`/`acquire` call; taffy may call measure 2-3x per text leaf per layout (min-content, max-content, definite).
- `resolve_text_style` walks ancestors per dirty text node and allocates a `TextStyle` (Cow font_family -> owned String clone); `element.resolved_text.clone()` per measure call.
- `write_back` text branch calls `text.measure(.., None)` then `text.acquire` per dirty text node (2 hash lookups).
- `evict` is an O(n) scan per evicted entry.
- Impeller shapes a whole `Paragraph` per text node; no glyph atlas of our own, no line cache reuse across widths (quantized to 0.5px steps).
- `ImpellerTextSystem::shape` may build the paragraph twice (centered/trailing text) to fit width.

## Render thread (runtime/src/render/*)
- `RenderTree.nodes: HashMap<NodeKey, RenderNode>` with std SipHash; iteration order random -> `prepare` iterates all nodes each frame (ticks animations, compares `VisualValues` by value).
- `RenderInbox::wants_frame` calls `tree.is_animating()` = O(n) scan, called 2x per loop iteration.
- `compose` (render/compose.rs:685) clones + sorts `children` by z_index for every node every frame; builds a fresh `Vec<CompositeOp>` each frame; pushes a `PushLayer` per node (3 ops/node) even for plain nodes with identity transform & opacity 1.
- `compose_frame` (impeller/src/composite.rs:309) rebuilds a full Impeller `DisplayList` each frame via `DisplayListBuilder` and `builder.save()`/`restore` per node; a `draw_display_list` per boundary; nothing is retained between frames except the per-boundary display lists.
- `flush_releases` builds HashSets over all nodes whenever any release is pending.
- `RenderOp::SetChildren` lingering check is O(children^2) (`children.contains`).
- `record_node` re-records when ANY VisualValues paint field changes (radii, colors) — fine — but the record for a scroll container also happens on content_offset changes because the op path sets needs_record.
- No damage tracking: `SurfaceState.damage` is always empty -> `surface.damage(0,0,MAX,MAX)` every frame (windowing/src/wayland/window/render_surface.rs:149). Compositor repaints the whole window every frame.
- No opaque region is ever set (opaque_region None) -> compositor cannot skip blending behind the window.
- Frame pacing: `wait_for_frame` blocks on wl_callback (good), `FrameClock` steps one refresh at a time (good). Present blocks on Vulkan FIFO acquire (Impeller limitation).

## Icons / SVG (widgets/src/svg.rs, kit/icon.rs)
- Every icon mount formats an SVG string and parses it with `usvg::Tree::from_str` (no cache keyed by body/stroke); every repaint `draw()` clones each `Path` (Vec<PathCommand>) into the DrawList, then `Command::Canvas` moves the list, then `collect_render_ops` clones it again, then the render thread hashes the path commands with SipHash to find the segment texture (impeller/src/segment_textures.rs:1105), and the cache is cleared wholesale at 256 entries.

## Lists
- `lazy_for_each`/`lazy_index` are fixed-row-height only; items are absolutely positioned; rebuild on window change (good), no variable-height support, no measured-item sum tree.
- Row mount cost = full `Kind::mount` incl. taffy node creation, usvg parse for icons, text shaping.

## Hashing / allocations / tooling
- No FxHash/ahash/foldhash anywhere (grep empty); `std::collections::HashMap` everywhere incl. hot maps (TextCache, RenderTree, LayoutState.elements, Dispatcher).
- No `benches/` and no criterion/divan anywhere in the workspace; only `FrameStats` (layout_us/record_us/render_us) ring of 120 frames via inspect/MCP.
- `Element` modifiers: `SmallVec<[Modifier; 6]>` (good). `ElementNode` is large (listeners Vec, variants SmallVec, NodeStyle with Vec<GridTrack>, TextOverrides with String...).
- reactive `Runtime` stores every signal as `Box<dyn Any>` in a SlotMap; `notify` clones the subscriber SmallVec; `next_pass.contains` linear. Fine for now.

## Things crownui already does well (do not re-plan)
- Retained render tree on a dedicated thread; animations sampled on the render thread's clock; re-records only changed boundaries.
- Analytic SDF rounded-box shader + flattened-path shader for icons; MSAA fallback.
- Text shaping cached by (text hash, style hash, width quantum) with refcounted upload to the renderer.
- Frame callbacks pace the render thread; one Vulkan context; `select_vulkan_driver` avoids waking other GPUs; swapchain image count trimmed.
- Fixed-height virtualization (`lazy_index`) with overscan and index-based rebuild.
