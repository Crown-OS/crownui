# crowndev

Developer tools for running CrownUI apps, over the MCP socket every app opens
(`$XDG_RUNTIME_DIR/crownui/<app>-<pid>.sock`).

```text
crowndev apps                                  running apps, one per line
crowndev tree [app]                            the semantic tree as text
crowndev inspect [app]                         the inspect snapshot as pretty JSON
crowndev act <app> <query> <action>            ui_query, then ui_act at the current revision
crowndev watch [app]                           reprint the tree when it changes (500 ms poll)
```

- `app` is a name (`counter`) or `name-pid`; it may be left out while exactly one app runs.
- `query` is `role=…,name=…,text=…` with any subset of the keys: `role=button,name=+1`. A comma not
  followed by a key belongs to the value (`name=Hello, world`). It must match exactly one node.
- `action` is `click`, `double_click`, `hover`, `focus`, `type_text=hello`, `press_key=Ctrl+A`,
  `scroll=0,120`, `drag=40,300`, or a raw `ui_act` action object such as `{"action":"click"}`.
- `--socket-dir <dir>` before the command overrides the socket directory; `CROWNMCP_TOKEN` is sent in
  the token handshake when set.
