# gesture

Turns raw `input` events into gestures and keyboard focus moves. Names follow the Web DOM
(`Click`, `DragStart`, `focus-visible`) and Flutter (recognizers competing in a gesture arena).

- `Recognizer`: a state machine fed one pointer's events; it votes `Pending`, `Accept` or `Reject`
  and holds its `GestureOutput`s until the arena says it won. Built-ins: `ClickRecognizer`
  (click/dblclick/triple-click, any button), `LongPressRecognizer` (fires from `poll(now)`),
  `DragRecognizer` (slop, optional axis lock, release velocity) and `PinchRecognizer` (touchpad).
- `GestureArena<K>`: one arena per `PointerId`. The first `Accept` wins, the last survivor of a
  closed arena wins, and `sweep` on pointer up hands an undecided arena to its first member.
- `VelocityTracker`: least-squares velocity over the last 100 ms of samples, no allocation.
- `FocusManager<K>`: Tab / Shift+Tab order and arrow-key spatial navigation over window rects.
- Time is a monotonic `Duration`; tunables live in `pub const` defaults and per-recognizer configs.

Non-goals: hit testing and event bubbling (see `view`), multi-finger touchscreen gestures.
