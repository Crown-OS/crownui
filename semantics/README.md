# semantics

What the UI *means*, independent of how it looks: a tree of `SemanticNode`s with a `Role`, an
accessible `name`, a `value`, `States`, supported `ActionKind`s and screen `bounds`. Every consumer
reads the same tree:

- `accessibility` maps it to AccessKit (screen readers),
- `mcp` serves it as `ui://tree` and executes `UiAction`s,
- `crowntest` queries it (`role(Role::Button).name("Save")`) and snapshots it as text,
- devtools display it.

`UiAction` is the shared action vocabulary (Testing Library / Playwright names): `click`,
`double_click`, `hover`, `type_text`, `press_key`, `scroll`, `drag`, `focus`.

Ids (`SemanticId`) are stable across frames: the view layer derives them from the component path
and keys, so agents and tests can hold on to them. `SemanticTree::revision` increments on every
change; MCP rejects actions aimed at a stale revision.

Invariants: plain data; nodes marked `secure` never expose their value.
