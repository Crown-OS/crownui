# Naming

One obvious name per thing, taken from vocabularies people and agents already know.

## Tailwind class → style method (literal values in logical pixels)
| Tailwind | Method |
|---|---|
| `p-4 px-4 py-2 pt pr pb pl` | `.p(16.0) .px(16.0) .py(8.0) .pt() .pr() .pb() .pl()` |
| `m-* mx my mt mr mb ml` | `.m() .mx() .my() .mt() .mr() .mb() .ml()` |
| `w-* h-* size-* min-w max-w min-h max-h aspect-*` | `.w() .h() .size() .min_w() .max_w() .min_h() .max_h() .aspect()` |
| `w-full h-full w-1/2` | `.w_full() .h_full() .w_frac(0.5) .h_frac()` |
| `flex-row flex-col flex-row-reverse flex-col-reverse flex-wrap flex-1` | `.flex_row() .flex_col() .flex_row_reverse() .flex_col_reverse() .flex_wrap() .flex_1()` |
| `grow shrink basis-*` | `.grow(1.0) .shrink(0.0) .basis()` |
| `gap gap-x gap-y` | `.gap() .gap_x() .gap_y()` |
| `items-{start,center,end,stretch,baseline}` | `.items_start() .items_center() .items_end() .items_stretch() .items_baseline()` |
| `self-{start,center,end,stretch}` | `.self_start() .self_center() .self_end() .self_stretch()` |
| `justify-{start,center,end,between,around,evenly}` | `.justify_start() … .justify_evenly()` |
| `grid grid-cols-* col-span-* row-span-*` | `.grid() .grid_cols(3) .grid_template_cols(..) .col_span() .row_span()` |
| `absolute relative inset-* top right bottom left z-*` | `.absolute() .relative() .inset() .top() .right() .bottom() .left() .z()` |
| `overflow-hidden overflow-scroll` | `.overflow_hidden() .overflow_scroll()` |
| `bg-*` | `.bg(color)` |
| `text-*` (color / size) | `.text_color(color) .text_size(px)` |
| `font-{medium,semibold,bold} font-* italic` | `.font_medium() .font_semibold() .font_bold() .font_weight(600) .font_family(..) .italic()` |
| `leading-* tracking-* truncate line-clamp-*` | `.leading() .tracking() .truncate() .line_clamp()` |
| `text-{left,center,right,justify}` | `.text_left() .text_center() .text_right() .text_justify()` |
| `rounded rounded-{t,r,b,l,tl,tr,br,bl}` | `.rounded() .rounded_t() … .rounded_bl()` |
| `border border-* (color) ring outline` | `.border(1.0) .border_color(c) .ring(w, c) .outline(w, c)` |
| `shadow-{sm,md,lg,xl} shadow-none` | `.shadow(Shadow::md()) .shadow_none()` |
| `opacity-* backdrop-blur-*` | `.opacity(0.5) .backdrop_blur(24.0)` |
| `translate-x translate-y scale-* scale-x-* scale-y-* rotate-* origin-*` | `.translate_x() .translate_y() .translate(v) .scale() .scale_x() .scale_y() .rotate(deg) .origin(x, y)` |
| `pointer-events-none` / `pointer-events-auto` | `.pointer_events_none()` `.pointer_events_auto()` — on a root, the opted-in boxes become the surface's input region |
| `invisible` / `hidden` (display: none) `cursor-*` | `.hidden(true)` (keeps its space) `.display_none()` `.cursor(CursorIcon::Pointer)` |
| `transition duration ease-*` | `.transition(Spring::snappy())`, `.transition(Curve::ease_in_out(200))` |
| `hover: active: focus: focus-visible: disabled: dark:` | `.hover(|s| ..) .active() .focus() .focus_visible() .when_disabled() .dark()` |
| (CrownOS) | `.material(Material::thin())` — compositor-rendered glass; may be bound to fade it |
| `requestAnimationFrame` | `cx.request_animation_frame(\|cx, now\| ..)` — UI-thread stepping for motion `.transition` cannot express |

## DOM event → listener
| DOM | Listener |
|---|---|
| `click dblclick contextmenu` | `on_click on_double_click on_context_menu` |
| `pointerdown/up/move/enter/leave/cancel` | `on_pointer_down … on_pointer_cancel` |
| `wheel scroll` | `on_wheel on_scroll` |
| `keydown keyup` | `on_key_down on_key_up` |
| `input change submit` | `on_input on_change on_submit` |
| `focus blur` | `on_focus on_blur` |
| `dragstart drag dragend` (pointer gesture) | `on_drag_start on_drag on_drag_end` |
| HTML DnD `dragenter`/`dragover dragleave drop` | `on_drag_over` (`event.accept(mime)`) `on_drag_leave` `on_drop`; start one with `cx.start_drag` |
| `compositionupdate / beforeinput` | `on_text_input` |
| (touch/touchpad, no DOM equivalent) | `on_long_press on_pinch` |
| (CrownOS) the compositor asks a launcher to start an app | `cx.on_launch_app(\|cx, app_id\| ..)` after `cx.activate_app_icon(app_id)` |

`event.stop_propagation()` and `event.prevent_default()` behave as on the web.

## Actions (crowntest, MCP `ui_act`, accessibility)
`click double_click hover focus type_text press_key scroll drag`; queries `role(..) name(..) text(..)`.
