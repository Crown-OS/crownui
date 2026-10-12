# crownconfig

The only crate in the workspace that talks to `crownos-config`. Everything that reads, writes or
watches `~/.config/crownos/*.ron` goes through here, so the rest of CrownUI never names the
config crate (or its optional xilem integration).

- **Sections** — `load_section`, `save_section`, `watch_section` work on any serde type; a missing
  file is created with its defaults, a malformed one falls back to them.
- **Appearance** — `Appearance`, `AccentColor`, `AnimationProfile`, `load_appearance`,
  `save_appearance`: the desktop's dark mode, accent and animation profile.
- **Theme overrides** — `theme.ron` holds sparse edits (`accent`, `light`, `dark` slot maps) on
  top of the shipped palettes. `ThemeSettings` bundles it with the appearance, and
  `watch_theme_settings` follows both files.
- **Keybinds** — `Keybind`, `KeyCode`, `Mods` re-exported for shortcut widgets.
- **Watching** — `SectionWatch<T>` holds the latest value; `changed().await` resolves on the next
  edit, so a UI task can feed a signal with it.

Non-goals: colors (the theme in `widgets` interprets the strings), UI types, blocking I/O on the
UI thread beyond the initial load.
