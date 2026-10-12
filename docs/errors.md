# Diagnostic codes

| Code | Where | Meaning | Fix |
|---|---|---|---|
| CUI0001 | reactive | A signal/memo/store was used after its scope was disposed. | Use `try_get`/`try_set` for values that can outlive their component, or keep the handle inside its component. |
| CUI0101 | semantics | A query matched no node. The message lists similar nodes. | Fix the role/name, or wait for the element to appear (`advance_ms`). |
| CUI0102 | semantics | A query matched several nodes. | Narrow the query (`role(..).name(..)`), or add a `.label(..)`. |
| CUI0103 | view | An action named an unknown semantic id. | Re-read the tree (ids are stable, but the node may have unmounted). |
| CUI0104 | view | The node does not support the action. | Check the node's `actions` in the tree. |
| CUI0105 | view | Agents may not type into secure fields. | — (by design). |
| CUI0106 | view | The node is disabled. | Enable it first. |
| CUI0201 | view | A layout property was used in a style variant (`hover`, `dark` …); it is ignored. | Variants take paint/composite properties only. |
| CUI0202 | view | A bound or non-style modifier was used in a variant; it is ignored. | Use literal values in variants. |
| CUI0301 | view | A component panicked inside an `error_boundary`; the fallback is shown. | See the logged message and component path. |
| CUI0401 | runtime | An image upload was rejected (wrong byte length). | Pass `width × height × 4` straight-alpha RGBA bytes. |
| CUI0402 | runtime | The renderer could not be created. | Check GPU/driver; try `CROWNUI_RENDERER=gles`. |
| CUI0403 | runtime | The window surface was lost. | The compositor closed the connection. |
| CUI0404 | runtime | A frame failed to render. | See the backend error. |
| CUI0405 | runtime | The render thread stopped. | See the preceding CUI040x error. |
| CUI0406 | runtime | The created window has no surface. | Platform bug. |
| CUI0407 | runtime | The async task executor could not start. | `cx.spawn` returns `None`. |
| CUI0408 | runtime | The MCP server could not start. | Check `$XDG_RUNTIME_DIR/crownui` permissions. |
| CUI0501 | widgets | The SVG source could not be parsed; nothing is drawn. | Validate the SVG. |
| CUI0601 | crownconfig | A config section could not be written. | Check the permissions of `$CROWN_CONFIG_DIR` (default `~/.config/crownos`). |
