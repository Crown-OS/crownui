# Components

Components are the building blocks under every CrownUI screen. They have plain, system-neutral defaults and no theme. The themed CrownOS controls in [widgets.md](widgets.md) are built from them.

Every component is a `view::Kind` with a lowercase builder that returns an `Element`. That means every Tailwind modifier, DOM listener and style variant works on all of them.

## Primitives

| Builder | What it does |
|---|---|
| `text(content)` | Shows text. The content can be a literal, a `Signal<String>`, a `Memo` or a `bind`. |
| `vstack(children)` | Lays out children in a column. |
| `hstack(children)` | Lays out children in a row. |
| `zstack(children)` | Stacks children on top of each other. |
| `spacer()` | Grows to fill the free space in a stack. |
| `divider()` | Draws a one-pixel separator. |
| `button(label)` | A focusable button that clicks on Enter and Space. |
| `toggle(on)` | A switch bound to a `Signal<bool>` with an animated knob. |
| `text_field(value)` | An editable `Signal<String>` that fires `on_input`, `on_change` and `on_submit`. |
| `scroll_view(content)` | Scrolls its content with the wheel or touchpad. |
| `lazy_vstack(items, key, row_height, row)` | A virtual list that mounts only the visible rows. |
| `canvas(painter)` | Custom drawing into a `DrawList`. It repaints when its size or a signal it reads changes. |
| `image(width, height, rgba)` | Shows RGBA8 pixels and frees them on unmount. |
| `cover_image(image, natural_size, radius)` | Crops a registered image to cover its box with rounded corners. |
| `svg(source)` and `icon(source, tint)` | Draws vector art parsed with usvg. |
| `popover(anchor, open, content)` | Shows an overlay below an anchor. Escape closes it. |

## Structure

| Builder | What it does |
|---|---|
| `component(\|cx\| ..)` | Builds a view once and owns the signals created inside it. |
| `show(condition, then, otherwise)` | Mounts `then()` while the condition holds and `otherwise()` when it does not. |
| `keyed(key, build)` | Rebuilds a branch when a derived key changes. |
| `for_each(items, key, row)` | Renders a keyed list and gives each row its own signal. |
| `lazy_for_each(..)` | A virtualized `for_each`. |
| `error_boundary(content, fallback)` | Catches a panic in its subtree and shows the fallback. |

Use `.enter(|style| ..)` and `.exit(|style| ..)` on any element to animate it in and out.

## Modifiers

Every element accepts the same modifier set.

- Layout: `p`, `m`, `w`, `h`, `size`, `gap`, `flex_row`, `flex_col`, `grow`, `items_center`, `justify_between`, `grid`, `grid_cols`, `absolute`, `inset`.
- Paint: `bg`, `text_color`, `rounded`, `border`, `ring`, `shadow`, `opacity`, `backdrop_blur`, `material`.
- Transform: `translate`, `scale`, `rotate`, `origin`.
- Text: `text_size`, `font_semibold`, `font_family`, `leading`, `tracking`, `truncate`, `line_clamp`.
- Variants: `hover`, `active`, `focus`, `focus_visible`, `when_disabled`, `dark`.
- Motion: `transition(Spring::snappy())` or `transition(Curve::ease_in_out(200))`.
- Semantics: `label`, `role`.

The complete table is in [naming.md](naming.md).

## Writing your own component

1. Create a struct that implements `view::Kind` with a `NAME` and a `mount` method.
2. In `mount`, set the role with `cx.set_role`, default styles with `cx.set_default_style`, children with `cx.set_children` and listeners with `cx.add_listener`.
3. Return it from a lowercase builder function such as `pub fn rating(..) -> Element<RatingKind>`.
4. Test it with `crowntest` by clicking it through its role and name.

For fully custom drawing, use `canvas` or call `cx.set_canvas` inside `mount`. The `crownui::ext` module exports `Kind`, `DrawList`, `Paint` and the other extension types.
