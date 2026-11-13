<p align="center">
  <img src="docs/assets/banner.png" alt="CrownUI widgets: calendar, menus, quick panel, inputs, toggles, buttons, sliders and callouts" width="100%">
</p>

# CrownUI

CrownUI is a declarative UI toolkit for Rust and the native toolkit of CrownOS. You build views once in a SwiftUI style, drive them with fine-grained signals, style them with Tailwind names and handle DOM-named events. Impeller renders every frame on a dedicated thread, and one semantic tree serves screen readers, tests and AI agents.

```rust
use crownui::prelude::*;

fn main() -> Result<(), crownui::RunError> {
    crownui::run(
        component(|cx| {
            let count = cx.signal(0);
            vstack((
                text(bind(move |rt| format!("Count: {}", count.get(rt)))).text_size(24.0),
                button("+1").on_click(move |cx, _| count.update(cx, |count| *count += 1)),
            ))
            .p(24.0)
            .gap(12.0)
        }),
        AppConfig::new("Counter"),
    )
}
```

## Installation

CrownUI needs Rust 1.97 or newer and runs on Linux under a Wayland compositor.

```sh
cargo add crownui
```

The first build downloads the Impeller SDK (about 23 MB) into `.impeller_cache/`. Set `IMPELLER_CACHE_DIR` to a vendored copy for offline builds, and enable the `static` feature to link Impeller into your binary.

Install the native libraries first.

Debian and Ubuntu:

```sh
sudo apt install build-essential pkg-config libwayland-dev wayland-protocols \
  libxkbcommon-dev libvulkan-dev libegl1-mesa-dev libgbm-dev \
  libfontconfig-dev libfreetype-dev libdbus-1-dev
```

Arch Linux:

```sh
sudo pacman -S base-devel wayland wayland-protocols libxkbcommon \
  vulkan-icd-loader vulkan-headers mesa fontconfig freetype2 dbus
```

### Features

| Feature | Default | What it adds |
|---|---|---|
| `wayland` | yes | The Wayland platform. |
| `impeller` | yes | The Impeller renderer on Vulkan with a GLES fallback. |
| `tokio` | yes | Async tasks through `cx.spawn`. |
| `mcp` | yes | An in-app MCP server so AI agents can drive the app. |
| `accessibility` | yes | Screen reader support through AT-SPI. |
| `static` | no | Links libimpeller statically. |

## Setup from source

The workspace depends on two sibling CrownOS repositories by path, so clone all three side by side.

```sh
mkdir crownos && cd crownos
git clone https://github.com/Crown-OS/crownui.git
git clone https://github.com/Crown-OS/crownos-config.git
git clone https://github.com/Crown-OS/crownos-protocols.git
cd crownui
cargo xtask check
```

`rust-toolchain.toml` pins the toolchain, so rustup installs the right version on the first build. `cargo xtask check` runs clippy, tests and rustfmt on the crates you changed. See [AGENTS.md](AGENTS.md) for every workspace command.

## Examples

| Example | What it shows |
|---|---|
| `counter` | Signals, buttons and hover and press variants. |
| `todo` | Keyed lists, per-row signals, text input and delete actions. |
| `gallery` | Buttons, toggles and a text field in styled sections with dark mode variants. |
| `glass_card` | A canvas gradient behind a backdrop blur card that expands with a spring. |
| `kit_gallery` | The themed CrownOS widgets following the desktop theme. |

Run one with:

```sh
cargo run -p crownui --example counter
```

Render any kit widget to a PNG without opening a window:

```sh
cargo xtask preview --list
cargo xtask preview button_primary --png button.png
```

## Documentation

The same pages are published on the CrownOS website under `/docs/crownui`.

- Get started: [Introduction](docs/introduction.md), [Quickstart](docs/quickstart.md), [Tutorial](docs/tutorial.md), [Why CrownUI](docs/why-crownui.md).
- Core concepts: [Views](docs/views.md), [State](docs/state.md), [Lists](docs/lists.md), [Styling](docs/styling.md), [Layout](docs/layout.md), [Events](docs/events.md), [Animation](docs/animation.md).
- Building apps: [Windows](docs/windows.md), [Async work](docs/async.md), [Drawing](docs/drawing.md), [Clipboard and drag](docs/clipboard-and-drag.md), [Accessibility](docs/accessibility.md), [Testing](docs/testing.md), [AI agents](docs/agents.md), [Audio](docs/audio.md), [Custom components](docs/custom-components.md).
- The kit: [Overview and theming](docs/kit.md), [Controls](docs/kit-controls.md), [Inputs](docs/kit-inputs.md), [Feedback](docs/kit-feedback.md), [Layout](docs/kit-layout.md), [Overlays](docs/kit-overlays.md).
- Architecture: [Overview](docs/architecture.md), [Frame pipeline](docs/frame-pipeline.md), [Reactivity](docs/reactivity.md), [Rendering](docs/rendering.md), [Semantics](docs/semantics.md), [Backends](docs/backends.md), [Performance](docs/performance.md).
- Reference: [API reference](docs/api.md), [Naming](docs/naming.md), [Errors](docs/errors.md), [Contributing](docs/contributing.md), [Impeller notes](docs/impeller.md).

Each crate also has its own `README.md` with its public API and invariants.

## License

CrownUI is dual licensed under [MIT](LICENSE-MIT) or [Apache 2.0](LICENSE-APACHE), at your option.
