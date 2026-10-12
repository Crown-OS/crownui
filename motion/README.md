# motion

Animation math with no state of its own, shared by the CrownUI render thread and the compositor
(so both evaluate exactly the same curve).

- `Animatable`: anything with add/sub/scale (implemented for `f32` and every `geometry` type).
- `Spring`: closed-form damped harmonic oscillator. `x(t)` is evaluated directly, so sampling at
  any timestamp (presentation time, compositor scanout) gives identical results — no integration
  drift. Presets: `snappy`, `smooth`, `crisp`, `bouncy`, `interactive`.
- `Curve`: a spring or a CSS-style timing curve (`ease`, `ease_in_out(ms)` …, cubic-bezier); `.repeating()` loops a timing curve forever (spinners).
- `Animation<T>`: one segment from `from` to `to`; `retarget` starts a new segment from the
  current value **and velocity**, so interrupted animations never jump.
- `Transaction`: the curve attached to a batch of property changes.

Times are `Duration`s on one monotonic clock. Non-goals: keyframes, physics beyond springs.
