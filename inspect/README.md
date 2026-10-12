# inspect

Plain, serde-serializable snapshot types that every runtime layer can produce on demand. The
in-app MCP server exposes them as `inspect://…` resources and the future `crowndev` devtools read
the same data, so there is exactly one inspection protocol.

| Type | Produced by | Contents |
|---|---|---|
| `LayoutSnapshot` | runtime | every node's path, rect and style summary |
| `SignalGraphSnapshot` | runtime (from `reactive`) | signal/observer/scope counts and the scope tree |
| `RenderTreeSnapshot` | runtime | repaint boundaries, cached list sizes, active animations |
| `FrameStats` | runtime | per-frame timings, kept in a ring buffer |
| `InspectSnapshot` | runtime | everything above plus the semantic tree and draw dump as text |

`SNAPSHOT_VERSION` increments on any breaking change. Producers stay cheap: nothing is collected
until a client asks.
