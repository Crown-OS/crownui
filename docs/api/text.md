# text

`text` shows a run of text. Its content is a literal or a reactive value, and it wraps to the width its parent gives it.

```rust
use crownui::prelude::*;

fn greeting() -> impl View {
    component(|cx| {
        let name = cx.signal(String::from("Ada"));
        vstack((
            text("Welcome back").text_size(28.0).font_bold(),
            text(bind(move |rt| format!("Signed in as {}", name.get(rt))))
                .text_color(rgb(0x64748b)),
        ))
        .gap(4.0)
    })
}
```

## Signature

```rust
pub fn text(content: impl IntoProp<String>) -> Element<TextKind>
```

## Parameters

- `content: impl IntoProp<String>` (required). One of:
  - a literal: `&str`, `String` or `&String`
  - a `Signal<String>` or `Memo<String>`
  - a `bind(move |rt| ..)` closure that returns a `String`

A `bind` closure must return `String`, not `&str`. Call `.to_string()` or use `format!`.

## Returns

- `Element<TextKind>`. It has the `Text` role. Its accessible name is the text itself, so tests and agents find it with `role(Role::Text).name("Welcome back")`.

## Reactive content

When the content is a `Signal`, a `Memo` or a `bind`, only this node's text changes when the inputs change. The rest of the tree is untouched.

```rust
let seconds = cx.signal(0_u32);
text(bind(move |rt| {
    let total = seconds.get(rt);
    format!("{:02}:{:02}", total / 60, total % 60)
}))
.font_medium()
```

A `Signal<String>` works directly.

```rust
let status = cx.signal(String::from("Idle"));
text(status)
```

## Text style methods

Every method takes a literal. The ones marked bindable also take a `Signal`, a `Memo` or a `bind`.

- `.text_size(px)` sets the font size in logical pixels. Bindable. The default is 14.
- `.text_color(color)` sets the color. Bindable. The default is black.
- `.font_weight(weight)` sets a weight on the 100 to 900 scale.
- `.font_medium()`, `.font_semibold()`, `.font_bold()` set common weights.
- `.font_family(name)` picks a family by name. Register custom fonts with `AppConfig::font`.
- `.italic()` uses the italic style.
- `.leading(factor)` sets the line height as a multiple of the font size. Bindable.
- `.tracking(px)` sets the letter spacing in logical pixels. Bindable.
- `.text_left()`, `.text_center()`, `.text_right()`, `.text_justify()` align the lines. Left and right mean start and end, so they flip in a right-to-left layout.
- `.line_clamp(lines)` limits the text to a number of lines.
- `.truncate()` keeps the text on one line and ends it with an ellipsis.
- `.text_blur(radius)` blurs the glyphs alone by a radius in logical pixels. Bindable.

```rust
text("A very long file name that does not fit.pdf")
    .truncate()
    .w(180.0)

text(description)
    .text_size(13.0)
    .leading(1.4)
    .line_clamp(2)
```

## Inheritance

Text properties cascade like CSS. A `text` node without a size, color, weight, family, italic, line height, letter spacing or alignment takes the value from the nearest ancestor that sets it. Set them once on a container.

```rust
vstack((
    text("Title").font_bold(),
    text("Body copy in the same color and size."),
))
.text_size(15.0)
.text_color(rgb(0x0f172a))
```

`.line_clamp`, `.truncate` and `.text_blur` do not cascade. Set them on the `text` node itself.

## Layout

A `text` node measures itself. Its natural width is its longest line, and it wraps when its parent offers less. In an `hstack`, give the text `.grow(1.0)` so it takes the free width, and let it wrap or truncate there.

```rust
hstack((
    text(title).grow(1.0).truncate(),
    text("2 MB").text_color(rgb(0x64748b)),
))
.gap(8.0)
```

## Why it works this way

Impeller shapes text into a paragraph, and CrownUI caches the shaped layout by text, style and width. Unchanged text costs nothing per frame. Styles cascade so a page sets its typography once instead of on every label.

The cost is that every change of content shapes the text again. A label bound to a value that changes on every frame shapes on every frame. Keep fast-changing numbers short, and keep them in their own `text` node so the long text around them stays cached.

## See also

- [Styling](../styling.md)
- [App config](app-config.md) for registering fonts
- [Accessibility](../accessibility.md)
