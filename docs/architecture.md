# Architecture

CrownUI is a workspace of small crates stacked in layers. A crate depends only on crates in lower layers, and `cargo xtask layers` enforces this on every CI run. The rules live in [`layers.toml`](../layers.toml).

## Layers

| Layer | Crates | Role |
|---|---|---|
| base | `geometry` | Points, rects, colors, transforms and lengths. |
| foundations | `input`, `reactive`, `motion`, `inspect`, `crownconfig` | Events, signals, spring math, devtools snapshots and CrownOS settings. |
| contracts | `windowing`, `graphics`, `layouting`, `semantics`, `gesture` | Traits and plain data that the backends implement. |
| implementations | `impeller`, `taffy-layout`, `accessibility`, `mcp` | The default renderer, layout engine, screen reader bridge and agent server. |
| core | `view`, `runtime` | The element tree, modifiers, the `Ui` pipeline and the app runner. |
| components | `widgets` | Primitive widgets and the themed `kit`. |
| public | `crownui`, `crowntest`, `crownmcp`, `crowndev` | The facade apps depend on, the test harness, the agent broker and the devtools CLI. |

Every crate has its own `README.md` that lists its purpose, public API, invariants and non-goals. Start there when you work inside a crate.

## How a frame is made

```text
 platform events
       |
       v
+--------------------- UI thread ----------------------+
|  input -> gesture -> view listeners -> signals       |
|  reactive flush -> view update -> taffy layout       |
|  text shaping -> RenderOps + SurfaceState            |
+------------------------------------------------------+
       |  channel
       v
+------------------- render thread --------------------+
|  RenderTree <- RenderOps                             |
|  animations on their own clock                       |
|  re-record changed repaint boundaries                |
|  compose -> Impeller (Vulkan or GLES) -> present     |
+------------------------------------------------------+
```

The UI thread owns all state. It runs event handlers, flushes signals, updates the element tree, computes layout and shapes text. It then sends `RenderOp`s to the render thread over a channel.

The render thread owns the GPU renderer and a `RenderTree`. It applies the ops, steps every animation on its own clock, records only the repaint boundaries that changed, composes the frame and presents it. A slow handler on the UI thread never freezes a running animation.

## State without locks

Framework code has no `Mutex`, `RefCell`, `Rc` or `Arc<Mutex<..>>`. All reactive values live in one `reactive::Runtime`. A `Signal<T>` is a `Copy` key into that runtime, so passing it into closures costs nothing. Every read and write goes through `&mut Cx`, which derefs to the runtime. Threads talk to each other only through channels.

The flush is glitch free. Observers run in height order, memos stop propagation when their value does not change, and a pass cap reports cycles instead of hanging.

## The semantic tree

Each mounted element can carry a role, an accessible name, a value, states and supported actions. Together they form a `SemanticTree` with ids that stay stable across frames. Four consumers read the same tree:

- `accessibility` maps it to AccessKit so screen readers work over AT-SPI.
- `mcp` serves it to AI agents and runs their actions.
- `crowntest` queries it in tests and prints it in snapshots.
- `crowndev` shows it in the terminal.

Because the tree is shared, a button a screen reader can press is also a button a test or an agent can click.

## Backends are type parameters

`runtime::Runner<G, L>` is generic over a `GraphicsBackend` and a `LayoutEngine`, and it drives any `windowing::Platform`. The defaults are Wayland, Impeller and taffy. The `headless` platform and the `RecordingBackend` run the same pipeline with no GPU, which is how tests and previews work.

To add a backend, implement the contract trait in a new crate or a feature-gated module. Nothing above the contract changes.

## Rendering

The `impeller` crate renders with Flutter's Impeller engine. Vulkan is the default and GLES is the fallback. Set `CROWNUI_RENDERER=gles` to force GLES. Rounded rects, strokes and shadows use an analytic edge shader, so edges stay one device pixel wide at any scale or transform. Notes on the Impeller integration and its measured frame costs are in [impeller.md](impeller.md).

## Diagnostics

Every warning and error carries a stable code such as `CUI0101`. The full index with fixes is in [errors.md](errors.md).
