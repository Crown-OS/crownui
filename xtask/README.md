# xtask

Workspace automation, run as `cargo xtask <command>`. Every command prints a human summary, or a
machine-readable report with `--json` (`{"command", "ok", "steps": [{"name", "ok", "detail"}]}`),
and exits non-zero on failure — so agents can parse results instead of scraping logs.

| Command | Does |
|---|---|
| `check [crate..]` | rustfmt `--check` on changed `.rs` files, then clippy `-D warnings` and tests for the named or changed crates |
| `layers` | enforces `layers.toml`: no crate depends on its own or a higher layer unless allowed |
| `api [--check]` | writes / compares `api/<crate>.txt` with `cargo public-api` (requires `cargo install cargo-public-api`) |
| `snapshot` | lists pending `.snap.new` insta snapshots awaiting review |
| `preview <name> [--png <path>]` | renders a `widgets::kit` preview as a text snapshot, or to PNG via the offscreen Impeller renderer |
