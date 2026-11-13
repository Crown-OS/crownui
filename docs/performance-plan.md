# Performance plan: what CrownUI takes from GPUI

Study of Zed's GPUI (zed-industries/zed @ `c7bea34`, 2026-11-04; sparse clone at `/tmp/gpui`, crates
`gpui`, `gpui_platform`, `gpui_linux`, `gpui_wgpu`, `gpui_apple`, `scheduler`, `sum_tree`, `collections`,
`refineable`, `bench_metrics`) against the CrownUI pipeline on branch `files-app`. File references below
are `crate/path:line` for CrownUI and `/tmp/gpui/crates/...:line` for GPUI.

## 1. The two architectures, and what that means for porting

| | GPUI | CrownUI |
|---|---|---|
| Element tree | Rebuilt every frame from `render()`, bump-allocated in a 1 MiB-chunk arena (`gpui/src/arena.rs`, `window.rs:338`) | Built once, retained `ElementTree` in a `SlotMap` (`view/src/tree.rs`) |
| Invalidation | `cx.notify(entity)` marks one view and its ancestors dirty; unchanged views replay their recorded prepaint/paint ranges (`view.rs:310-657`, `window.rs:4244-4356`) | Per-node `Dirty` flags set by signals and commands (`view/src/node.rs:104`) |
| Layout | Taffy tree cleared and rebuilt every frame (`window.rs:3743`); cached views contribute one leaf | Persistent taffy tree, dirty nodes re-styled (`view/src/layout.rs`) |
| Scene | Flat per-kind primitive arrays sorted by a BoundsTree draw order, one draw call per batch, re-uploaded every frame (`scene.rs`, `gpui_wgpu/src/wgpu_renderer.rs`) | Per-boundary Impeller display lists, re-recorded only when a node changes, composed each frame (`runtime/src/render/*`, `impeller/src/composite.rs`) |
| Threads | One thread: draw + present on the platform frame callback | UI thread + render thread per window, channels only |

GPUI's headline tricks (arena, double-buffered frames, scene replay, BoundsTree ordering, instance
buffers) exist to make a full per-frame rebuild cheap. CrownUI does not rebuild, so those do not
transfer. What transfers is everything GPUI does to keep each hot path proportional to what changed,
plus caches and platform behaviour CrownUI lacks. The plan is therefore: keep the retained model, remove
the O(n) scans that currently defeat it, and port the specific caches, indexes and compositor
behaviours below.

## 2. Where CrownUI spends time today

Everything below runs per frame or per input event and is proportional to the whole tree, not to
the change.

| Hot spot | Where | Cost shape |
|---|---|---|
| `needs_frame` scans every node's dirty flags on every loop turn | `view/src/ui.rs:334`, called from `runtime/src/window_runner.rs:140` | O(n) per `about_to_wait` |
| `sync_styles` collects all ids and walks all nodes twice per layout pass | `view/src/layout.rs:146` | O(n) |
| `write_back` reads every node's layout from taffy and marks every descendant of a scrolled container `dirty.frame` | `view/src/layout.rs:489` (fn `write_back`) | O(n) |
| A wheel tick sets `layout_needed`, forcing taffy, `write_back` and a semantics rebuild | `view/src/dispatch.rs:564-602` | full layout per scroll event |
| `collect_render_ops` walks the whole tree, clones each node's `children` Vec and each canvas `DrawList` | `view/src/render_op.rs:88,173,181` | O(n) + allocations |
| `refresh_semantics` rebuilds the semantic tree, a `HashMap` of every node, and sorts focusables; `handle_event` marks it dirty on every event | `view/src/ui.rs:251,405,437`, `view/src/semantics_build.rs` | O(n log n) per event |
| `hit_test` walks the tree to find top layers, then allocates and sorts children at every visited node; `pointer_move` hit-tests twice | `view/src/tree.rs:192-286`, `view/src/dispatch.rs:437` | O(n) per pointer move |
| `key_down` scans every node for a shortcut | `view/src/dispatch.rs:604` | O(n) per key |
| `material_shapes` and `assign_semantic_ids` walk every node per `take_frame` | `view/src/render_op.rs:197`, `view/src/cx.rs` (`assign_semantic_ids`) | O(n) |
| `refresh_visual` clones the whole `NodeStyle` (grid tracks, font family String) on each hover or press | `view/src/cx.rs:730` | allocation per interaction |
| `TextCache` hashes the full text and 11 style fields with SipHash on every measure; taffy measures a leaf two or three times; eviction is a linear scan | `view/src/text_cache.rs:56-89,228`, `graphics/src/text.rs:83` | per measure |
| Render thread iterates every node per frame to tick animations and compare values; `is_animating` scans twice per loop | `runtime/src/render/tree.rs:235-289`, `runtime/src/render_inbox.rs:24` | O(n) per frame |
| `compose` clones and sorts every node's children by `z_index` and pushes a layer per node | `runtime/src/render/compose.rs:55-120` | O(n log k) + allocations per frame |
| Every frame damages the whole surface and never sets an opaque region | `windowing/src/wayland/window/render_surface.rs:142,174` | compositor repaints everything |
| Every icon instance formats and parses an SVG with usvg; every repaint deep-clones each path, which is cloned again into the op and hashed again on the render thread; the segment cache clears wholesale at 256 | `widgets/src/kit/icon.rs:33`, `widgets/src/svg.rs:92-127`, `impeller/src/segment_textures.rs:14-32` | per mount, per repaint |
| No `FxHash`, no `SmallVec` for children, no benchmarks anywhere in the workspace | `grep` of `Cargo.toml` files | no regression gate |

Already done, do not re-plan: analytic SDF corners and icon paths with MSAA fallback, text shaping
cached by content/style/width with refcounted uploads, frame-callback pacing with a one-refresh-step
clock, `select_vulkan_driver`, trimmed swapchain image count, Mailbox present mode through the Vulkan
loader shim (`impeller/src/vulkan/surface_extents.rs:125-141`, in the working tree), wheel events
coalesced per pointer frame by sctk.

## 3. GPUI techniques and how they map

Status: **have** = CrownUI already does it; **port** = adopt; **adapt** = same idea, different form;
**skip** = does not apply (reason in §6).

| GPUI technique | Reference | Status |
|---|---|---|
| Dirty set of views + ancestors, early exit when an ancestor is already dirty | `window.rs:3830-3836`, `key_dispatch.rs:588-596` | port (§4.1) |
| Wake the platform only on the dirty false→true transition | `window.rs:171-226` | port (§4.1) |
| Scroll offset applied after layout, never touches taffy | `div.rs:2083`, `window.rs:5536` | port (§4.4) |
| Flat hitbox list built during prepaint with clip captured, scanned in reverse, only for elements that consume input, tested once per frame and once per event | `window.rs:1169-1191,5545-5559`, `div.rs:2461-2486` | port (§4.5) |
| Hover change dirties only the owning view | `div.rs:2905-2930` | have |
| `FxHashMap` everywhere, `SmallVec` for children and paths, precomputed path hashes | `collections/src/collections.rs`, `element.rs:218-283` | port (§4.2) |
| `Vec::clear()` not `Vec::new()` across frames | `window.rs:1126-1150`, `scene.rs:57-69` | port (§4.2, §4.7) |
| Text element returns the cached size when the wrap width is unchanged so repeated taffy measures do not reshape | `elements/text.rs:686-700` | port (§4.3) |
| Two-generation line layout cache, borrowed key lookups with zero allocation | `text_system/line_layout.rs:479-611` | adapt (§4.3) |
| Font resolution cache with negative entries; pooled char-width tables | `text_system.rs:245-262,505-519` | skip (Impeller owns fonts) |
| Scene culls primitives whose clipped bounds are empty | `scene.rs:87-95` | port (§4.7) |
| Layers group geometry under one order | `window.rs:4783-4800` | adapt (§4.7) |
| Opaque region only when the background is opaque and decorations are server side | `gpui_linux/src/linux/wayland/window.rs:2459-2476` | port (§4.8) |
| Interactive resize throttled to one configure per vblank | `wayland/window.rs:1121-1129` | verify (§4.8) |
| Minimum frame interval for inactive, low-power and thermal states | `window.rs:1360-1383` | port (§4.9) |
| Sum-tree list of measured/unmeasured items, seek by pixel, anchor by (item, offset) | `elements/list.rs:62-75,1036-1268` | adapt (§4.11) |
| Uniform list measures one item | `elements/uniform_list.rs:661-684` | port (§4.11) |
| SVG parsed once, rasterised once per size, keyed in an `FxHashMap` | `svg_renderer.rs`, `platform.rs:1826` | adapt (§4.10) |
| Erf-based analytic box shadow in one quad, no blur passes | `gpui_wgpu/src/shaders.wgsl:325-338,950-1046` | port (§4.12) |
| Border-only quads split into four strips | `window.rs:4886-5000` | optional (§4.12) |
| Randomised element-tree benchmarks per change kind, foreground-instruction metric, hdr histograms | `gpui_platform/benches/randomized_element_tree.rs`, `bench_metrics` | port (§4.0) |
| Hang monitor on its own OS thread | `profiler/hang/monitor.rs:78-111` | port (§4.0) |
| Frame-time overlay painted straight into the scene | `debug_overlay.rs` | port (§4.0) |
| Arena element tree, scene replay, BoundsTree, instance buffers, glyph atlas, subpixel text, entity lease model, priority executor | §6 | skip |

## 4. The plan

Each item names the GPUI reference, the CrownUI change, and how it is verified. Phases are ordered
by payoff over risk. Measure before and after every phase with §4.0.

Progress (2026-11-05): 4.0 items 1 and 2 (`cargo xtask bench`, `UiCounters`, 9aab750), 4.1 items
1-7 (dirty queues, pruned write-back, queue-driven ops, semantics on demand with bounds patching,
shortcut and material indexes, `refresh_visual` on a `Visual` copy; 9047dea, 0555544), 4.4 items
1-2 (scroll by translating the subtree, composite-only ops never re-record; 7aefeab) and 4.5
items 2-3 (one hit test per pointer move, subtree-bounds pruning, top-layer index; 810f2c5) are
done. Against the saved `before` baseline, retired instructions per frame fell 38-72% in every
interactive scenario and 94-99.8% for an unchanged frame. Still open in those phases: 4.0 items
3-5, 4.2, 4.3, 4.5 item 1 (the flat hitbox list), 4.6; everything from 4.7 on.

Progress (2026-11-07), measured against GPUI, Qt, Electron and Tauri with `cargo xtask compare`
(`benchmarks/`): parts of 4.2 (FxHash for the text cache, layout lookups and the render tree; O(1)
resubscription in `reactive`), 4.3 (one shaping per unwrapped text instead of two, a shaped layout
kept two frames after its last node lets go, width-only min-content queries answered from the
unbounded metrics, eviction in batches) and 4.7 (the render thread ticks and re-records only nodes
that changed or animate). New beyond the plan: compose drops rounded clips nothing reaches into,
since each one cost Impeller an offscreen layer that was reallocated whenever scrolling changed its
size, and `AppConfig::frame_pacing(FramePacing::Unpaced)` draws without waiting for the display.
Result (`benchmarks/results/2026-11-07T07-43Z.md`, `…T08-08Z.md` uncapped): fewer instructions per
frame than GPUI in every scenario (list 2.35 M vs 5.23 M, animation 29.5 M vs 30.0 M, settings
3.7 M vs 19.7 M); uncapped, settings draws 1 092 fps against GPUI's 581 and the list 2 303 against
2 356 at 0.31 vs 0.42 CPU ms per frame, while animation draws 329 against 423.
At 60 Hz it takes more CPU ms per frame than GPUI in list (2.36 vs 1.95) and animation (6.01 vs
4.78), and Qt does less in list and settings. What is left: the render thread's time is Impeller's text
path (glyph atlas lookups, vertex generation) and per-op dispatch; the UI→render→fence-wait
handoff costs about 13 context switches per frame against GPUI's 2, and splitting work over two
cores lets the `powersave` governor clock them lower. Next levers: a composite-only render op so
transform changes stop shipping and comparing the whole `Visual`, and fewer thread handoffs per
frame.
Memory since: text measured but never drawn (wrapping descriptions probed at several widths) kept
its shaped paragraph until the cache's byte budget evicted it; it now drops it two frames on like a
released layout (settings: 29 → 21.5 MiB anonymous after a full page cycle). In-flight animations
are boxed, so a resting `RenderNode` no longer carries three copies of its visual values.

### 4.0 Phase 0: measure first

No behaviour changes. Without this nothing below can be shown to help.

1. **Frame benchmarks** in `runtime/benches/` (dev-dependency on `divan` or `criterion`; `runtime`
   already owns `InlineApp` and can use `graphics::RecordingBackend` + `taffy_layout::TaffyEngine`).
   Model on `gpui_platform/benches/randomized_element_tree.rs`: seeded tree families (wide, deep,
   mixed) at 256, 1 000 and 10 000 nodes; one bench per change kind: `unchanged`, `hover`,
   `scroll_tick`, `text_change`, `insert_remove_rows`, `root_resize`, `full_rebuild`; plus
   `lazy_list_100k_scroll` and `icon_grid_1k_mount`. Report wall time and retired instructions
   (`perf_event_open` through `criterion-perf-events` or `iai-callgrind`); GPUI uses the
   instruction count as the regression gate because it is stable across machines
   (`bench_metrics.rs:1400-1449`). Add `cargo xtask bench [--save-baseline NAME | --baseline NAME]`
   in `xtask/src/command/bench.rs`.
2. **Counters, not just timers.** Expose from `Ui` and `RenderTree` the numbers the benches assert
   on: taffy `compute_layout` calls, nodes visited by `write_back`, hit tests per event, semantics
   rebuilds, boundaries recorded, composite ops. These are the acceptance criteria below.
3. **`FrameStats`** (`inspect/src/lib.rs:64`): fill `update_us`, add `hit_test_us`, `semantics_us`,
   `compose_us`, `ops`, `recorded`, `nodes`, and `dirty_to_present_us` (UI-thread dirty time to
   render-thread present, both already have timestamps). `crowndev` prints p50/p95/p99 over the
   120-frame ring, GPUI style (`profiler.rs:1002-1011` bucket edges are a good default).
4. **Hang watchdog**: the UI thread sends a heartbeat on a channel each loop turn; a `crownui-watchdog`
   thread does `recv_timeout` and logs `CUI0409: UI thread stalled N ms (last event: …)` when it
   misses (GPUI `HangMonitor`, `monitor.rs:78-111`). Channels only, no shared state.
5. **Frame-time overlay** behind `CROWNUI_OVERLAY=1`: the render thread appends one extra boundary
   with a bar per recent frame and a budget line, recorded from its own `RenderReport` history. It
   bypasses layout and the UI thread entirely (GPUI `debug_overlay.rs:1-3`).

Exit: baselines saved for every bench; counters wired.

### 4.1 Phase 1: make the UI thread O(changes)

GPUI reference: `dirty_views: FxHashSet<EntityId>` plus `mark_view_dirty` that stops at an already
dirty ancestor (`window.rs:2479-2490`), and a single `dirty` flag that wakes the platform once
(`window.rs:204-226`).

1. **Dirty queues in `Cx`** (`view/src/dirty.rs`): `layout`, `measure`, `visual`, `content`,
   `children` as `Vec<NodeId>` with a per-node `queued` bitset to avoid duplicates, plus a
   `subtree_dirty: bool` on `ElementNode` propagated up the parent chain until it hits a node already
   set. `apply_style`, `set_text`, `set_children`, `Command::Canvas` and friends push to the queues
   instead of only flipping flags. `Ui::needs_frame` (`ui.rs:334`) becomes a flag read.
2. **`sync_styles` consumes the queues** instead of collecting every id (`layout.rs:146`).
3. **`write_back` prunes clean subtrees** (`layout.rs`, fn `write_back`): a node that is not
   `subtree_dirty` and whose rect and `content_size` are unchanged is not descended into. Taffy's
   own cache guarantees its descendants are unchanged too. Scroll offsets stop flowing through
   `write_back` (see §4.4), so scrolling no longer dirties descendants' frames.
4. **`collect_render_ops` visits only `subtree_dirty` nodes** (`render_op.rs:88`); iterate children
   by index with a reborrow instead of `children.clone()` (`:181`); move the canvas list out with
   `mem::take` instead of cloning it (`:173`) and verify nothing on the UI thread reads it after
   collection (devtools read the render thread's copy).
5. **Semantics on demand.** Stop setting `semantics_dirty` in `handle_event` (`ui.rs:251`). Mark it
   from the operations that change the semantic tree (children, role, label, value, focus, disabled)
   and track frame-only changes separately. `refresh_semantics` runs only when a consumer asks
   (`semantic_tree()`, `perform`, the accessibility bridge when a screen reader is connected, MCP
   when a client is connected, `crowntest`) and otherwise at idle. Frame-only changes patch
   `bounds` in place through a new `SemanticTree::update_bounds(id, rect)`; structural changes
   rebuild the dirty subtree only. `semantic_index` and the sorted focusables update
   incrementally. `assign_semantic_ids` runs for subtrees whose `children` changed.
6. **Indexes replace scans**: `material_nodes: Vec<NodeId>` maintained when `StyleProp::Material`
   is applied or a node is removed (replaces `material_shapes` iterating everything,
   `render_op.rs:197`); `shortcuts: FxHashMap<Keybind, SmallVec<[NodeId; 1]>>` maintained by
   `set_shortcut`/removal (replaces the `key_down` scan, `dispatch.rs:604`; GPUI resolves bindings
   from an index, `keymap.rs:165-240`).
7. **`refresh_visual` without cloning `NodeStyle`** (`cx.rs:730`): variants may only carry visual
   and text props (enforced at `cx.rs` `add_variant`), so apply them to a `Visual` copy (it is
   `Copy`) and a borrowed `TextOverrides`. Split `StyleProp::apply` into `apply_visual`,
   `apply_text`, `apply_layout` in `view/src/style_prop.rs`.

Acceptance (benches): `unchanged` frame UI cost is flat across 256/1k/10k nodes; `hover` visits a
constant number of nodes; `text_change` on one leaf visits only its ancestor chain; zero semantics
rebuilds during pointer moves.

### 4.2 Phase 1 (continued): hashing and allocation hygiene

GPUI reference: `collections/src/collections.rs` (`FxHashMap` as the default map), `GlobalElementId`
precomputed hashes (`element.rs:218-283`, the comment there says path hashing had been a visible
share of a frame), `SmallVec<[LayoutId; 2]>` children (`div.rs:1903`), `Frame::clear` keeping
capacity (`window.rs:1126-1150`).

1. Add `rustc-hash` to `[workspace.dependencies]`; use `FxHashMap`/`FxHashSet` in `TextCache`
   (`text_cache.rs:46`), `LayoutState` (`layout.rs:20`), `Dispatcher` (`dispatch.rs:83`),
   `lazy_for_each`/`lazy_index` row maps, `semantic_index`, `RenderTree.nodes`
   (`runtime/src/render/tree.rs:58`), `ImpellerRenderer.boundaries`, `DrawResources` maps,
   `SegmentTextures`. Keys are small integers or precomputed `u64`s, where SipHash buys nothing.
2. `ElementNode.children: SmallVec<[NodeId; 4]>` and `listeners: SmallVec<[Listener; 2]>`
   (`node.rs:133,145`); `RenderNode.children` likewise.
3. Drop the taffy reverse map `LayoutState.elements` (`layout.rs:20`): store the `NodeId` in the taffy
   node context (`taffy_layout::MeasuredLeaf(NodeId)`), so the measure closure gets it for free.
4. Reuse per-frame buffers: `Composition.ops` (`compose.rs`), the `DrawList` in `RenderTree::prepare`
   (already reused), the ops `Vec` in `take_frame`, hit-test paths. `clear()`, never `new()`.

Acceptance: allocation count per `unchanged` and `hover` frame reported by the bench harness drops to
a small constant.

### 4.3 Phase 1 (continued): text measurement

GPUI reference: `elements/text.rs:686-700` (cached size when the wrap width matches),
`line_layout.rs:684-734` (current-frame hit, then move from previous frame, then shape),
`line_layout.rs:838-885` (borrowed `CacheKeyRef` so a hit allocates nothing).

1. **Hash once.** `TextContent` stores `text_hash: u64` recomputed on `Command::Text`;
   `ElementNode.resolved_text` stores the `TextStyle` with its `shaping_key` computed once at
   resolution. `TextCache::key` (`text_cache.rs:89`) takes both hashes; `hash_text` and
   `TextStyle::shaping_key` (`graphics/src/text.rs:83`) switch to `FxHasher`.
2. **Per-node measure memo**: `TextContent.last_measure: Option<(Option<u32> width_steps, TextMetrics)>`.
   Taffy's repeated min-content, max-content and definite calls on one leaf hit this before the cache.
3. **Resolve text styles top-down** during the dirty-subtree walk in `sync_styles` instead of walking
   ancestors per dirty text node (`layout.rs`, fn `resolve_text_style`). `TextOverrides.font_family`
   becomes `Option<Arc<str>>` so inheritance does not clone a `String` (GPUI `SharedString`).
4. **Eviction by generation sweep**: replace the linear `min_by_key` per eviction (`text_cache.rs:228`)
   with a `generation` stamp per entry bumped per layout pass and an amortised sweep of entries older
   than N passes when over budget. Keep the byte budget.
5. Not portable: GPUI wraps as an index overlay over one shaped line (`WrappedLineLayout`,
   `line_layout.rs:305-324`) so width changes never reshape. Impeller's `Paragraph` reshapes per
   width; the 0.5 px width quantum already in `TextCache` is the right mitigation.

Acceptance: `text_change` bench shows one shaping call per changed leaf; `root_resize` on a text-heavy
tree shows measure calls equal to text leaves, not two to three times that.

### 4.4 Phase 2: scrolling without layout

GPUI reference: the scroll offset lives beside the element and is added in `layout_bounds`
(`window.rs:5525-5538`) after taffy; `paint_scroll_listener` mutates it and notifies the view
(`div.rs:3385-3449`); `UniformList`/`List` compute the visible range from the offset arithmetically.

1. `scroll_nearest` (`dispatch.rs:564-602`) stops setting `layout_needed` and `dirty.frame`. It
   applies `StyleProp::ContentOffset` (one `RenderOp::Visual`; the render thread already moves
   children through a layer transform, `compose.rs:95-105`) and calls a new
   `ElementTree::translate_subtree(node, delta)` that shifts descendants' `window_rect` without
   touching taffy and marks their semantic bounds for the in-place patch of §4.1.5.
2. On the render thread, `RenderOp::Visual` sets `needs_record` only when the paint fields differ
   (`tree.rs`, `RenderOp::Visual` arm; compare with `VisualValues::paint_equals`). Today a scroll
   re-records the container's own display list every tick.
3. `ScrollHandle::attach` and the lazy lists keep working unchanged: they read the offset signal.

Acceptance: `scroll_tick` bench records zero `compute_layout` calls, zero boundaries re-recorded, and
visits only the scrolled subtree.

### 4.5 Phase 2 (continued): hit testing

GPUI reference: `insert_hitbox` during prepaint with the content mask captured
(`window.rs:5545-5559`), `Frame::hit_test` reverse scan with `BlockMouse` (`window.rs:1169-1191`),
`should_insert_hitbox` only for elements that consume input (`div.rs:2461-2486`), result cached in
`window.mouse_hit_test` and computed once per frame and once per mouse event.

1. **Hitbox list** (`view/src/hitbox.rs`): `Vec<Hitbox { node, rect, clip, pointer_events, top_layer }>`
   in paint order, rebuilt only when any node's frame, visual (`hidden`, `z_index`, `top_layer`,
   `clip`), listeners or `pointer_events` changed (one `hitboxes_dirty` flag), otherwise translated
   in place by §4.4's `translate_subtree`. Only nodes that can receive pointer input enter it:
   listeners, variants (hover/active), `cursor`, `Overflow::Scroll`, focusable, `pointer_events`
   overrides. Paint order needs sibling z-index order; keep a per-node `paint_order:
   SmallVec<[NodeId; 4]>` sorted when `children` or a child's `z_index` changes, so neither the UI
   thread nor the compositor sorts per frame (§4.7).
2. **`hit_test` = reverse scan** returning the hit node; the ancestor path comes from the parent chain
   into a `SmallVec<[NodeId; 8]>`. `top_layers` and the per-node `Vec` + sort in `hit_test_node`
   (`tree.rs:217-286`) go away.
3. **One hit test per pointer move**: `pointer_move` (`dispatch.rs:437`) computes the path once and
   shares it between `update_hover` and the `PointerMove` dispatch. `update_hover` runs
   `apply_pending` once after the enter/leave diff, not once per node (`dispatch_along` does it per
   call today).

Acceptance: `hover` bench shows one hit test per event, cost proportional to hitbox count, zero
allocations beyond the path `SmallVec`.

### 4.6 Phase 2 (continued): resize and input verification

GPUI reference: interactive resizes are acked but not applied more than once per vblank
(`gpui_linux/src/linux/wayland/window.rs:1121-1129`); keyboard repeat timers carry a generation id
so stale timers cancel themselves (`client.rs:2232-2271`).

1. Verify `windowing/src/wayland/window/state.rs` applies only the latest configure per frame during an
   interactive resize and that `set_viewport` is not called for an unchanged size. Fix if not.
2. Verify key repeat cancels on focus loss and key change. CrownUI's `seat/keyboard.rs` likely does;
   confirm with a test.

### 4.7 Phase 3: render thread and composition

GPUI reference: `Scene::insert_primitive` drops primitives whose clipped bounds are empty
(`scene.rs:87-95`); `paint_layer` groups geometry under one order (`window.rs:4783-4800`);
`request_animation_frame` keeps only animating views active (`window.rs:2942-2945`); per-frame
Vecs are cleared, not reallocated.

1. **Active sets in `RenderTree`** (`runtime/src/render/tree.rs`): `animating: Vec<NodeKey>` and
   `needs_record: Vec<NodeKey>` maintained by `apply` and `prepare`; `prepare` (`:235`) ticks and
   re-records only those; `is_animating` (`:289`) is `!animating.is_empty()`. Replace the
   `flush_releases` scan (`:184`) with per-id use counts incremented on `Content` ops and decremented
   on content change or removal. Replace the O(n²) lingering check in `SetChildren` (`:108`) with a
   small `FxHashSet`.
2. **Compose without sorting or cloning** (`compose.rs:55-120`): consume the pre-sorted paint order
   from §4.5.1 (ship it in `RenderOp::SetChildren` or sort once on the render thread when `children`
   or `z_index` change); reuse `Composition.ops` across frames; **skip the per-node `PushLayer`** when
   the placement is a pure translation at opacity 1 with no clip or backdrop, folding the translation
   into the child transforms. Most nodes are exactly that; Impeller then sees far fewer
   `save`/`restore` pairs in `compose_frame` (`impeller/src/composite.rs:48`).
3. **Viewport and clip culling in compose**: track the current clip in window space (axis-aligned
   transforms only, which is every node without `rotate` or non-uniform `scale`); skip a subtree
   whose frame does not intersect it. A plain `scroll_view` of a long `vstack` stops composing
   off-screen rows, and off-screen boundaries stop being recorded at all.
4. **Subtree display-list caching** (stretch, after 1-3 are measured): for a node whose subtree has
   had no frame, visual or content change for one frame and holds at least K descendants, cache the
   composed Impeller `DisplayList` of the whole subtree and replay it with `draw_display_list` next
   frame. This is GPUI's `Scene::replay` by range (`scene.rs:141-149`) in Impeller terms. Only worth
   it if `compose_us` still dominates after 1-3.

Acceptance: `unchanged` frame render-thread cost is flat across node counts; `compose_us` for 10k
nodes drops by the fraction of layer-free nodes; `scroll_tick` composes only the visible rows.

### 4.8 Phase 3 (continued): damage and opaque region

GPUI reference: opaque region set when the background is opaque and decorations are server side
(`gpui_linux/src/linux/wayland/window.rs:2459-2476`) because client-side decorations include
transparent shadows and antialiased corners.

1. **Damage**: the render thread already knows which boundaries it re-recorded, which frames moved and
   which values animated. Union their window-space rects (expanded by shadow blur), cap at eight rects
   or fall back to full damage, and put them in `SurfaceState.damage`. Today every frame damages the
   whole surface (`render_surface.rs:142`), so crownpositor recomposites the whole window for a hover.
2. **Opaque region**: when the root's background alpha is 1, no `material`, no root rounding and no
   `BodyEdge::Hard` shape, set `opaque_region` to the root rect; clear it otherwise. The compositor
   can then skip blending behind the window.

Acceptance: a hover in a 10k-node window produces one damage rect; `weston-debug`/crownpositor damage
logs confirm.

### 4.9 Phase 3 (continued): frame pacing policy

GPUI reference: `minimum_frame_interval` is the max of the inactive-window, low-power and thermal
intervals (`window.rs:1360-1383`); `FramePacer::should_defer` reschedules with 1 ms tolerance
(`:1331-1357`).

1. When the window is unfocused (`WindowEvent::Focused(false)`) and only render-thread animations
   demand frames, pace the render thread at half the refresh. Keep full rate while a pointer is
   inside or an input arrived within the last second.
2. Read `power-profiles-daemon` (or `/sys/class/power_supply`) once at start and on change; in
   power-saver mode cap animation frames at 60 Hz. Both are small changes in `render_thread.rs`
   around `wait_for_frame` (`:151`) plus a `RenderMessage::Pacing` from the UI thread.

Acceptance: an unfocused window with a running spinner presents at the capped rate (frame counter).

### 4.10 Phase 4: paths and icons

GPUI reference: SVGs parsed once and rasterised once per `(path, size, scale)` into the atlas,
keyed by `RenderSvgParams` in `tiles_by_key: FxHashMap` (`platform.rs:1826,1855-1874`); the font
database is built lazily once (`svg_renderer.rs:131-149`).

1. **Share paths**: `graphics::Path { commands: Arc<[PathCommand]>, hash: u64 }` built once with an
   `FxHasher` (`graphics/src/path.rs:20`). Every clone along `draw` → `Command::Canvas` →
   `collect_render_ops` → render thread becomes a refcount bump, and `SegmentTextures` keys by the
   stored hash (`impeller/src/segment_textures.rs:14-32`) instead of rehashing every command. Replace
   the clear-all at 256 with an LRU keyed by last-used frame. Plain `Arc<T>` sharing immutable data is
   not the locked shared state the invariants forbid; confirm in `AGENTS.md` wording when
   implementing.
2. **Parse SVGs once**: a `util/svg_cache.rs` in `widgets` keyed by `(body pointer or hash,
   stroke width)` returning `Arc<SvgPicture>`; `mount_svg` (`svg.rs:151`) and `IconKind::mount`
   (`icon.rs:31`) hit it. A thousand rows with the same icon parse once.
3. `draw` (`svg.rs:106`) then pushes shared paths, so repaint on tint change is allocation-free.

Acceptance: `icon_grid_1k_mount` bench shows one usvg parse; render-thread `record_us` for an icon
re-tint has no hashing of path commands in the profile.

### 4.11 Phase 4 (continued): lists

GPUI reference: `UniformList::measure_item` renders one item to learn the row height
(`uniform_list.rs:661-684`); `List` keeps `SumTree<ListItem>` with `Unmeasured { size_hint }` and
`Measured { size }`, seeks by `Height` or `Count` in O(log n), splices measured items back, and
anchors the scroll position at `(item_ix, offset_in_item)` (`list.rs:62-75,231-299,1036-1268,867-924`).

1. **Auto row height for `lazy_index`**: `ItemLayout::rows(0.0)` mounts one item, observes its size,
   and uses that height for the layout, re-measuring on `revision` or width change.
2. **`lazy_measured`** (`view/src/lazy_measured.rs`): variable-height virtualization. Heights live in a
   Fenwick tree over `count` items (prefix sums and point updates in O(log n); far smaller than
   GPUI's persistent B+ tree and sufficient because item indices are stable). Unmeasured items use an
   estimate; mounted items feed real heights through `observe_size`. Visible range by prefix-sum
   seek from the scroll offset; the viewport anchors on `(index, offset_in_item)` so measuring rows
   above does not jump the view. `ScrollHandle::scroll_to_index`/`ensure_visible` consult the tree.
3. Row mount cost already shrinks from §4.3 (text hashes), §4.10 (icon parse) and §4.2 (SmallVec).

Acceptance: `lazy_list_100k_scroll` bench mounts only `visible + overscan` rows per tick and does no
layout outside them; a variable-height list of 100k items with mixed heights scrolls at the same
cost as the fixed one.

### 4.12 Phase 5: shader work

GPUI reference: shadows are drawn as one quad expanded by three blur radii; the fragment shader
integrates a closed-form `erf` along X and four Gaussian samples along Y, with an SDF fallback at
blur 0 (`gpui_wgpu/src/shaders.wgsl:325-338,950-1046`). No blur textures, no extra passes.

1. **Analytic box shadow** `impeller/shaders/box_shadow.frag` using the same packed-`vec4` uniform
   scheme as `rounded_box.frag` (`impeller/src/util/shape_uniforms.rs`). `DrawCommand::Shadow` with
   round corners draws one quad through it instead of `MaskFilter::new_blur`
   (`impeller/src/translate/mod.rs:130`), which costs Impeller an offscreen texture and two blur
   passes per shadow. Keep the mask filter for squircle corners (the erf form assumes circular
   corners) until an approximation is validated against the snapshot tests.
2. **Border-only strips** (optional): a stroke whose interior is empty draws four thin quads through
   the SDF shader instead of one quad covering the box (`window.rs:4886-5000`). Only matters for
   large outlined panels; measure before doing it.

Acceptance: a kit card grid with shadows shows `render_us` drop and no blur passes in a RenderDoc or
`VK_LAYER_LUNARG_api_dump` trace; snapshot tests pass within the existing tolerance.

## 5. Order and dependencies

```
4.0 measure
 ├─ 4.1 dirty queues ──┬─ 4.2 hashing/alloc ── 4.3 text memo
 │                     ├─ 4.4 scroll w/o layout ── 4.5 hitboxes ── 4.6 verify
 │                     └─ 4.7 render thread ── 4.8 damage/opaque ── 4.9 pacing
 ├─ 4.10 paths/icons
 ├─ 4.11 lists (needs 4.4)
 └─ 4.12 shaders
```

4.1, 4.4 and 4.7 are the three that change the cost class (O(n) per frame to O(changes)). 4.10 and
4.12 are independent and can run in parallel with them. Each phase lands as its own commit series
with the bench deltas in the message.

## 6. What not to port, and why

- **Arena-allocated immediate-mode element tree and double-buffered frames** (`arena.rs`,
  `window.rs:1254`): CrownUI's retained tree already avoids the per-frame rebuild these make cheap.
- **Scene replay by index range** (`scene.rs:141-149`, `window.rs:4310-4356`): the retained
  `RenderTree` plus per-boundary display lists is the same idea in a different shape; §4.7.4 is its
  Impeller equivalent if compose cost ever dominates.
- **BoundsTree draw ordering, per-kind primitive batching, instance storage buffers, nine pipelines**
  (`bounds_tree.rs`, `wgpu_renderer.rs`): Impeller owns batching and draw order.
- **Glyph atlas, subpixel dual-source text, cosmic-text fallback chains, char-width pools**: Impeller's
  `Paragraph` owns shaping and rasterisation. Revisit only if CrownUI leaves Impeller.
- **Entity map with lease/end_lease, observe/subscribe**: `reactive::Runtime` signals already give
  fine-grained invalidation without entity clones.
- **Weighted priority executor and deterministic test scheduler**: `cx.spawn` goes through the tokio
  bridge; `InlineApp`'s manual clock already gives deterministic tests.
- **Mailbox present**: already in the working tree (`surface_extents.rs:125-141`).
- **Per-frame scroll coalescing**: sctk already emits one axis event per `wl_pointer.frame`.

## 7. Invariants to keep while implementing

- No `Mutex`, `RefCell`, `Rc` or `Arc<Mutex<..>>` in framework code; the watchdog and pacing use
  channels. Shared immutable `Arc<[T]>` for paths is the one new sharing form; confirm it is accepted.
- One concept per file; helpers in `src/util/<name>.rs`; `mod.rs` and `lib.rs` only declare.
- Layering (`layers.toml`): benches live in `runtime`; the hitbox list and dirty queues live in `view`;
  the Fenwick tree is a `view/src/util/prefix_sums.rs`; the shader is in `impeller`.
- `thiserror` in libraries, no `unwrap`/`expect`; new diagnostics get `CUI` codes in `docs/errors.md`.
- Every phase keeps `cargo xtask check`, `cargo xtask api --check` and the insta snapshots green;
  render-order changes in §4.7 must leave `kit_previews` snapshots byte-identical.

## 8. Source notes

The three research reports this plan was distilled from, with GPUI file:line references for every
claim, are in `docs/research/gpui/` (`gpui-renderer-report.md`, `gpui-frame-report.md`,
`gpui-platform-report.md`) alongside `crownui-findings.md`, the CrownUI-side notes. Key GPUI files to read when implementing:
`gpui/src/window.rs` (frame lifecycle, hitboxes, pacing), `gpui/src/elements/list.rs` and
`uniform_list.rs` (virtualization), `gpui/src/text_system/line_layout.rs` (text cache),
`gpui/src/scene.rs` and `bounds_tree.rs` (culling and ordering), `gpui_wgpu/src/shaders.wgsl`
(shadow and SDF shaders), `gpui_linux/src/linux/wayland/window.rs` (frame loop, opaque region,
resize throttle), `gpui_platform/benches/randomized_element_tree.rs` and
`bench_metrics/src/bench_metrics.rs` (benchmark design).
