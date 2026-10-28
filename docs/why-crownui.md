# Why CrownUI

CrownUI is the native toolkit for CrownOS. It borrows the best known ideas from web and mobile UI and builds them on a Rust core with no shared mutable state. This page lists what sets it apart.

## You already know the names

CrownUI does not invent a vocabulary. Views are SwiftUI nouns like `vstack` and `button`. Styles are Tailwind classes like `.p(16.0)` and `.rounded(8.0)`. Events are DOM names like `on_click` and `on_key_down`. Test actions are Testing Library names like `click` and `type_text`. People and AI agents can read and write CrownUI code without a manual.

## No macros in the authoring API

Every view, modifier and event is a plain Rust function or method. rust-analyzer can complete them, show their types and jump to their source. Agents see the same real types.

## Fine-grained updates with no locks

Views are built once. When a signal changes, only the properties that read it update. There is no virtual DOM diff and no full rebuild. State lives in one runtime that you reach through `&mut Cx`, so there is no `Rc`, `RefCell` or `Mutex` anywhere in the framework.

## Animations that never stutter

A dedicated render thread owns the GPU and runs every animation on its own clock. If a handler on the UI thread takes too long, springs and transitions keep moving. Springs are evaluated in closed form, so an interrupted animation keeps its velocity and never jumps. The CrownOS compositor uses the same `motion` crate, so app and compositor animations match exactly.

## Impeller rendering

CrownUI renders with Impeller, the engine behind Flutter. It runs on Vulkan with a GLES fallback, draws one-pixel analytic edges on rounded shapes at any scale, and supports backdrop blur. Only repaint boundaries that changed are re-recorded each frame.

## One tree for accessibility, tests and agents

Every widget describes what it means, not only how it looks. That semantic tree feeds screen readers through AT-SPI, test queries in `crowntest`, the in-app MCP server and the `crowndev` CLI. When a widget works for one of them, it works for all of them.

## Built for AI agents

Every app can run an MCP server on a private socket. An agent such as Claude Code can list running apps, read their UI tree, click buttons, type text and call tools the app registers. Secure fields never expose their values, destructive tools need confirmation, and actions are rate limited.

## Readable snapshots

Test snapshots are plain text that list the semantic tree, the layout and every draw command. A reviewer, human or AI, can read a diff and see exactly what changed on screen.

## Deep CrownOS integration

CrownUI speaks the CrownOS Wayland protocols for compositor-rendered glass materials, squircle corners, launcher app icons, layer shell panels and system-wide drag and drop. The kit reads the desktop's appearance and accent color and follows changes live. It still runs on other Wayland compositors and only uses CrownOS protocols when the compositor offers them.

## Swappable backends

The runner is generic over the platform, the renderer and the layout engine. A headless platform and a recording renderer run the full pipeline without a GPU, which keeps tests fast and deterministic.
