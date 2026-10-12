# crowntest

Tests CrownUI views the way users (and agents) use them. Queries and actions share the vocabulary
of the MCP server and of Testing Library, so a test reads like a script of user steps:

```rust,ignore
let mut app = TestApp::new(counter());
app.click(role(Role::Button).name("+1"));
assert_eq!(app.text_of(role(Role::Text).text("Count")), "Count: 1");
insta::assert_snapshot!(app.snapshot());
```

- Runs the real pipeline headlessly (`runtime::InlineApp` + `RecordingBackend` + taffy), on a
  manual clock: `advance_ms(300)`, `settle()`.
- Actions: `click`, `double_click`, `hover`, `focus`, `type_text`, `press_key("Ctrl+S")`, `scroll`,
  `drag`; raw `pointer_*`/`key` events for gesture tests. Failing queries panic with the
  CUI0101/0102 explanation listing near matches.
- `snapshot()` returns the semantic tree, layout and recorded draw commands as plain text, so
  snapshot diffs are reviewable by humans and AI. Text is measured with a deterministic
  monospace approximation, so snapshots never depend on installed fonts.
