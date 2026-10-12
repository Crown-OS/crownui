# input

Platform-neutral input events. Names follow the Web DOM so they read the same in app code,
tests and MCP: `PointerEvent`, `KeyboardEvent` (`key`, `code`, `shift_key` …), `WheelEvent`,
`TextInputEvent` (IME preedit/commit), `TouchpadGesture` (pinch/swipe/hold, which the DOM lacks)
and `Keybind` (`"Ctrl+Shift+K"`, parsed and printed).

- Windowing backends translate native events into these; nothing here knows about Wayland.
- Positions are logical pixels relative to the window's top-left corner.

Non-goals: dispatch, bubbling and gesture recognition (see `gesture` and `view`).
