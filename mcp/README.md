# mcp

The in-app [MCP](https://modelcontextprotocol.io) server of a CrownUI app. Agents (usually through
the `crownmcp` broker) read the semantic tree, act on it, inspect the app and call app-defined tools.

```rust,ignore
let mut handle = McpServer::start(McpConfig::new("notes"))?;
handle.set_wakeup(move || proxy.wake());
handle.set_tools(vec![ToolSpec::new("new_note", "Create a note").input_schema(schema)]);
// in the event loop, never blocking:
for request in handle.pending_requests() {
    match request {
        McpRequest::Tree { reply } => reply.respond(tree.clone().into()),
        McpRequest::Act { id, revision, action, reply } => {
            reply.respond(validate_action(&tree, id, revision, &action).map(|_| dispatch(id, action)))
        }
        // CallTool, Inspect, Screenshot …
    }
}
```

**Threading.** The server runs on its own thread (current-thread tokio runtime) and talks to the UI
thread only through channels: requests carry a `Responder`; the UI thread answers when it likes.
Unanswered requests time out after 5 s.

**Transport and access.** `<socket_dir>/<app_name>-<pid>.sock` (default dir
`$XDG_RUNTIME_DIR/crownui`, mode 0700; socket 0600, removed on shutdown). Each connection's
`SO_PEERCRED` and optional token go to an `AccessGate` (`SameUserGate`, `TokenGate`, later the
compositor's `crownos-agent-access-v1`). **Token handshake:** before the MCP stream a client may send
one line `TOKEN <token>\n`; a stream starting with `{` carries no token. `mcp::connect` does this.

**MCP surface.** Resources `ui://tree` (text), `ui://tree.json`, `inspect://snapshot`. Tools
`ui_tree`, `ui_query {role?, name?, text?}`, `ui_act {id, revision, action}`, `ui_screenshot`,
`inspect_snapshot`, plus every `ToolSpec`; destructive ones carry `destructiveHint` and need
`"confirm": true`. Results with app text are wrapped as
`{"untrusted_app_content": …, "note": …}` against prompt injection. `ui_act` and app tools share a
token bucket (`actions_per_second`, default 20).
