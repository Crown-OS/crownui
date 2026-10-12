# crownmcp

A stdio MCP server that agents (Claude Code, …) launch to reach running CrownUI apps. Each app runs
the in-app `mcp` server on `$XDG_RUNTIME_DIR/crownui/<app>-<pid>.sock`; `crownmcp` finds those
sockets and proxies one selected app.

```sh
cargo install --path crownmcp            # or: cargo build -p crownmcp --release
claude mcp add crownui -- crownmcp       # add `-e CROWNMCP_TOKEN=<token>` for TokenGate apps
```

Tools:

- `list_apps` — running apps (`app` = `name-pid`, `name`, `pid`, `socket`, `selected`); sockets of
  dead processes are pruned.
- `select_app {app}` — connect by name or `name-pid`. With exactly one app running it is selected
  automatically.
- Everything the selected app offers: `ui_tree`, `ui_query`, `ui_act`, `ui_screenshot`,
  `inspect_snapshot`, app tools, and the `ui://tree`, `ui://tree.json`, `inspect://snapshot`
  resources (`tools/list`, `tools/call`, `resources/list` and `resources/read` are forwarded).

`CROWNMCP_TOKEN`, when set, is presented to apps as the `TOKEN <token>` handshake line before the
MCP stream (see the `mcp` crate). App sockets are only reachable by the same user (0700 directory,
0600 sockets, `SO_PEERCRED` checks).
