// A rounded or squircle box, filled or outlined, with analytic edge coverage. Drawn as a
// rectangle a little larger than the box; `_fragCoord` is in the draw's local coordinates.
// Recompile with `cargo xtask shaders` after editing; uniforms are packed by
// `impeller/src/util/shape_shader.rs` in declaration order. Runtime stages take no arrays.
#version 460 core
precision highp float;

const float SQUIRCLE_EXTENT = 1.5;
const float MIN_GRADIENT = 1e-4;

// x, y, width, height.
uniform vec4 bounds;
// Corner radii of the (+x, +y), (+x, -y), (-x, +y) and (-x, -y) corners.
uniform vec4 radii;
// Squircle flag, stroke width (0 fills), gradient flag, stop count.
uniform vec4 style;
// Gradient start x, y and end x, y.
uniform vec4 gradient_line;
uniform vec4 color0;
uniform vec4 color1;
uniform vec4 color2;
uniform vec4 color3;
uniform vec4 color4;
uniform vec4 color5;
uniform vec4 color6;
uniform vec4 color7;
uniform vec4 offsets_low;
uniform vec4 offsets_high;

in vec2 _fragCoord;
out vec4 frag_color;

float corner_norm(vec2 q) {
  vec2 q2 = q * q;
  return style.x > 0.5 ? sqrt(sqrt(dot(q2, q2))) : length(q);
}

float box_distance(vec2 p, vec2 half_size, vec4 r) {
  r.xy = p.x > 0.0 ? r.xy : r.zw;
  float radius = p.y > 0.0 ? r.x : r.y;
  float extent = style.x > 0.5 ? SQUIRCLE_EXTENT : 1.0;
  float reach = min(radius * extent, min(half_size.x, half_size.y));
  vec2 q = abs(p) - half_size + reach;
  return min(max(q.x, q.y), 0.0) + corner_norm(max(q, 0.0)) - reach;
}

float coverage(vec2 p, vec2 half_size, vec4 r) {
  float distance = box_distance(p, half_size, r);
  float pixel = max(length(vec2(dFdx(distance), dFdy(distance))), MIN_GRADIENT);
  return clamp(0.5 - distance / pixel, 0.0, 1.0);
}

float stop_offset(int index) {
  vec4 group = index < 4 ? offsets_low : offsets_high;
  int lane = index < 4 ? index : index - 4;
  return lane == 0 ? group.x : lane == 1 ? group.y : lane == 2 ? group.z : group.w;
}

vec4 stop_color(int index) {
  return index == 0 ? color0
       : index == 1 ? color1
       : index == 2 ? color2
       : index == 3 ? color3
       : index == 4 ? color4
       : index == 5 ? color5
       : index == 6 ? color6
       : color7;
}

vec4 brush_color(vec2 p) {
  if (style.z < 0.5) {
    return color0;
  }
  vec2 axis = gradient_line.zw - gradient_line.xy;
  float t = clamp(dot(p - gradient_line.xy, axis) / max(dot(axis, axis), MIN_GRADIENT), 0.0, 1.0);
  vec4 color = color0;
  for (int index = 1; index < 8; index++) {
    if (float(index) >= style.w) {
      break;
    }
    float from = stop_offset(index - 1);
    float to = stop_offset(index);
    vec4 blended = mix(stop_color(index - 1), stop_color(index), clamp((t - from) / max(to - from, MIN_GRADIENT), 0.0, 1.0));
    color = t > from ? blended : color;
  }
  return color;
}

void main() {
  vec2 half_size = bounds.zw * 0.5;
  vec2 p = _fragCoord - bounds.xy - half_size;
  float half_stroke = style.y * 0.5;
  vec4 rounded = step(MIN_GRADIENT, radii);
  float outer = coverage(p, half_size + half_stroke, radii + half_stroke * rounded);
  vec2 inner_half_size = half_size - half_stroke;
  float inner = coverage(p, max(inner_half_size, 0.0), max(radii - half_stroke, 0.0));
  float has_hole = step(MIN_GRADIENT, half_stroke) * step(MIN_GRADIENT, min(inner_half_size.x, inner_half_size.y));
  float alpha_mask = outer * (1.0 - inner * has_hole);
  vec4 color = brush_color(_fragCoord);
  float alpha = color.a * alpha_mask;
  frag_color = vec4(color.rgb * alpha, alpha);
}
