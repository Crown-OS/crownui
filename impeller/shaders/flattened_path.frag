// A small path flattened into line segments, filled (non-zero) or stroked, with analytic edge
// coverage. Drawn as a rectangle a little larger than the path; `_fragCoord` is in the draw's
// local coordinates. Segments come from the one-row `segments` texture written by
// `impeller/src/util/segment_texture.rs`; uniforms are packed by
// `impeller/src/util/path_uniforms.rs` in declaration order. Strokes join segments round.
// Recompile with `cargo xtask shaders` after editing.
#version 460 core
precision highp float;

const int MAX_SEGMENTS = 2048;
const float FIXED_POINT_MAX = 32767.0;
const float FAR = 1e20;
const float MIN_LENGTH = 1e-6;
const float MIN_PIXEL = 1e-6;

// x, y, width, height of the box segment coordinates are fixed-point fractions of.
uniform vec4 bounds;
// Segment count, half stroke width (0 fills), round-cap flag, square-cap extension.
uniform vec4 style;
uniform vec4 color;
// Two texels per segment, start then end. A texel holds 15-bit x and y words (big-endian in r, g
// and b, a), shifted left by one; the low bit of x marks an open contour's end, which gets a cap.
uniform sampler2D segments;

in vec2 _fragCoord;
out vec4 frag_color;

// The point and its cap flag.
vec3 segment_point(int texel) {
  vec2 uv = vec2((float(texel) + 0.5) / (style.x * 2.0), 0.5);
  vec4 bytes = floor(texture(segments, uv) * 255.0 + 0.5);
  vec2 words = bytes.xz * 256.0 + bytes.yw;
  vec2 fixed_point = floor(words * 0.5);
  return vec3(bounds.xy + fixed_point / FIXED_POINT_MAX * bounds.zw, words.x - fixed_point.x * 2.0);
}

// Position of the nearest point of the segment, from 0 at `from` to 1 at `to`, unclamped.
float projection(vec2 offset, vec2 axis) {
  return dot(offset, axis) / max(dot(axis, axis), MIN_LENGTH * MIN_LENGTH);
}

float segment_distance(vec2 p, vec2 from, vec2 to) {
  vec2 offset = p - from;
  vec2 axis = to - from;
  return length(offset - axis * clamp(projection(offset, axis), 0.0, 1.0));
}

// Signed distance to the stroke of one segment with flat ends pushed out by `extension`.
float box_distance(vec2 p, vec2 from, vec2 to, vec2 extension, float half_width) {
  vec2 axis = to - from;
  float span = length(axis);
  vec2 direction = span > MIN_LENGTH ? axis / span : vec2(1.0, 0.0);
  vec2 offset = p - from;
  float along = dot(offset, direction);
  float across = abs(direction.x * offset.y - direction.y * offset.x);
  vec2 outside = vec2(max(-extension.x - along, along - span - extension.y), across - half_width);
  return length(max(outside, 0.0)) + min(max(outside.x, outside.y), 0.0);
}

// Signed distance to one segment's stroke: a capsule with round caps; otherwise a box with butt
// or square ends at caps and a disc at each join.
float stroke_distance(vec2 p, vec3 from, vec3 to) {
  float half_width = style.y;
  if (style.z > 0.5) {
    return segment_distance(p, from.xy, to.xy) - half_width;
  }
  float box = box_distance(p, from.xy, to.xy, vec2(from.z, to.z) * style.w, half_width);
  float start_join = from.z > 0.5 ? FAR : length(p - from.xy) - half_width;
  float end_join = to.z > 0.5 ? FAR : length(p - to.xy) - half_width;
  return min(box, min(start_join, end_join));
}

// Coverage of the band between the stroke's two edges, so strokes under a pixel fade.
float stroke_coverage(vec2 p, float pixel) {
  int count = int(style.x + 0.5);
  float nearest = FAR;
  for (int index = 0; index < MAX_SEGMENTS; index++) {
    if (index >= count) {
      break;
    }
    nearest = min(nearest, stroke_distance(p, segment_point(2 * index), segment_point(2 * index + 1)));
  }
  float near_edge = clamp(0.5 - nearest / pixel, 0.0, 1.0);
  float far_edge = clamp(0.5 - (nearest + 2.0 * style.y) / pixel, 0.0, 1.0);
  return near_edge - far_edge;
}

// Non-zero winding from a ray towards +x, and the distance to the nearest edge. An edge only
// shades its pixels when it separates inside from outside: crossing it mid-span changes the
// winding by `nearest_step`; near its ends or on its line the step is unknown (0) and it shades.
float fill_coverage(vec2 p, float pixel) {
  int count = int(style.x + 0.5);
  float nearest = FAR;
  float nearest_step = 0.0;
  float winding = 0.0;
  for (int index = 0; index < MAX_SEGMENTS; index++) {
    if (index >= count) {
      break;
    }
    vec2 from = segment_point(2 * index).xy;
    vec2 to = segment_point(2 * index + 1).xy;
    vec2 offset = p - from;
    vec2 axis = to - from;
    float along = projection(offset, axis);
    float distance = length(offset - axis * clamp(along, 0.0, 1.0));
    if (distance < nearest) {
      nearest = distance;
      float side = axis.x * offset.y - axis.y * offset.x;
      nearest_step = along > 0.0 && along < 1.0 ? sign(side) : 0.0;
    }
    if ((from.y <= p.y) != (to.y <= p.y)) {
      float crossing = from.x + (p.y - from.y) / (to.y - from.y) * axis.x;
      winding += crossing > p.x ? sign(axis.y) : 0.0;
    }
  }
  bool inside = abs(winding) > 0.5;
  bool boundary = nearest_step == 0.0 || inside != (abs(winding - nearest_step) > 0.5);
  float coverage = clamp(0.5 - (inside ? -nearest : nearest) / pixel, 0.0, 1.0);
  return boundary ? coverage : 1.0;
}

void main() {
  float pixel = max(length(vec2(dFdx(_fragCoord.x), dFdy(_fragCoord.x))), MIN_PIXEL);
  float coverage = style.y > 0.0 ? stroke_coverage(_fragCoord, pixel) : fill_coverage(_fragCoord, pixel);
  float alpha = color.a * coverage;
  frag_color = vec4(color.rgb * alpha, alpha);
}
