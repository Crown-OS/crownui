# geometry

Plain value types every CrownUI crate shares: `Point`, `Vector`, `Size`, `Rect`, `Insets`,
`CornerRadii`, `CornerShape`, `Transform` and `Color`. All lengths are `f32` logical pixels; device pixels only
exist inside backends.

- Everything is `Copy`, has no allocation and no dependencies (optional `serde` feature for `inspect`).
- `Color` stores straight (non-premultiplied) sRGB `f32` components; build it with `rgb(0x3b82f6)`
  or `rgba(0x3b82f680)`.
- `Transform` is a 2D affine matrix; compose with `then`.

Non-goals: units other than logical pixels, 3D transforms, color spaces beyond sRGB.
