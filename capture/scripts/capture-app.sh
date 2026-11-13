#!/usr/bin/env bash
# Runs an app's design capture harness in light and dark appearance.
# usage: capture-app.sh <app-crate-dir> [example=design_capture] [output-dir]
# The harness is an example (examples/<example>.rs) so it renders on the main thread.
set -euo pipefail
app_dir=$(realpath "$1")
example=${2:-design_capture}
out=$(realpath -m "${3:-${CROWN_DESIGN_DIR:-$app_dir/target/design}}")
user_config=${CROWN_CONFIG_DIR:-${XDG_CONFIG_HOME:-$HOME/.config}/crownos}
mkdir -p "$out"
for theme in light dark; do
  config="$out/.config-$theme"
  rm -rf "$config" && mkdir -p "$config"
  cp -r "$user_config"/. "$config"/
  dark=$([ "$theme" = dark ] && echo true || echo false)
  sed -i -E "s/dark_mode: *(true|false)/dark_mode: $dark/" "$config/appearance.ron"
  (cd "$app_dir" && CARGO_INCREMENTAL=0 CROWN_CONFIG_DIR="$config" CROWN_DESIGN_THEME="$theme" \
    CROWN_DESIGN_DIR="$out" cargo run --quiet --example "$example")
done
ls -1 "$out"/*.figma.png
