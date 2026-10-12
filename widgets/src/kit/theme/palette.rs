use geometry::Color;

use super::accent::GradientStops;

/// A palette group's flat color slots by name, so `theme.ron` can edit them.
pub(super) type NamedSlots<'a, const COUNT: usize> = [(&'static str, &'a mut Color); COUNT];

/// The stack of backgrounds a window is built from, ordered by elevation, plus the lines between.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SurfaceColors {
    /// The window's own background.
    pub bg: Color,
    /// Anything lifted off the page: cards, popovers, menus.
    pub raised: Color,
    /// Anything recessed into it: sidebars, wells.
    pub sunken: Color,
    /// 1px border around a raised surface.
    pub border: Color,
    /// Hairline between rows inside one surface.
    pub hairline: Color,
    /// Translucent hover wash painted over whatever is beneath.
    pub hover: Color,
    /// Drop shadow behind raised surfaces.
    pub shadow: Color,
}

impl SurfaceColors {
    pub(super) fn slots_mut(&mut self) -> NamedSlots<'_, 7> {
        [
            ("bg", &mut self.bg),
            ("raised", &mut self.raised),
            ("sunken", &mut self.sunken),
            ("border", &mut self.border),
            ("hairline", &mut self.hairline),
            ("hover", &mut self.hover),
            ("shadow", &mut self.shadow),
        ]
    }
}

/// Foreground colors in descending emphasis; icons get two slots because a stroked glyph reads
/// lighter than text at the same value.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextColors {
    /// Headings and selected rows.
    pub primary: Color,
    /// Body copy and control labels.
    pub body: Color,
    /// Descriptions, section headers, read-only values, placeholders.
    pub muted: Color,
    /// Text that is present but not actionable.
    pub disabled: Color,
    /// Text and glyphs drawn on an accent fill.
    pub on_accent: Color,
    /// Icon stroke matching `body`.
    pub icon: Color,
    /// Icon stroke matching `muted`.
    pub icon_muted: Color,
}

impl TextColors {
    pub(super) fn slots_mut(&mut self) -> NamedSlots<'_, 7> {
        [
            ("primary", &mut self.primary),
            ("body", &mut self.body),
            ("muted", &mut self.muted),
            ("disabled", &mut self.disabled),
            ("on_accent", &mut self.on_accent),
            ("icon", &mut self.icon),
            ("icon_muted", &mut self.icon_muted),
        ]
    }
}

/// Slots shared by the hand-painted controls (toggle, slider).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ControlColors {
    /// A slider's unfilled track.
    pub track: Color,
    /// The toggle's knob and the slider's thumb.
    pub knob: Color,
    /// Drop shadow under the knob.
    pub knob_shadow: Color,
}

impl ControlColors {
    pub(super) fn slots_mut(&mut self) -> NamedSlots<'_, 3> {
        [
            ("track", &mut self.track),
            ("knob", &mut self.knob),
            ("knob_shadow", &mut self.knob_shadow),
        ]
    }
}

/// Semantic colors, deliberately independent of the accent so a red accent never hides an error.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StatusColors {
    /// Connected, saved, healthy.
    pub success: Color,
    /// Degraded but working.
    pub warning: Color,
    /// Failed, or about to destroy something.
    pub danger: Color,
    /// Nothing in particular; recedes next to the colored tones.
    pub neutral: Color,
    /// Fill behind a destructive control.
    pub danger_bg: Color,
    /// Text and glyphs drawn on `danger_bg`.
    pub on_danger: Color,
}

impl StatusColors {
    pub(super) fn slots_mut(&mut self) -> NamedSlots<'_, 6> {
        [
            ("success", &mut self.success),
            ("warning", &mut self.warning),
            ("danger", &mut self.danger),
            ("neutral", &mut self.neutral),
            ("danger_bg", &mut self.danger_bg),
            ("on_danger", &mut self.on_danger),
        ]
    }
}

/// Dropdown surfaces: the select trigger and its panel.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PopoverColors {
    /// Panel background.
    pub bg: Color,
    /// Panel and trigger border at rest.
    pub border: Color,
    /// Option labels and the selected value.
    pub text: Color,
    /// Section headers, chevron, placeholder.
    pub muted_text: Color,
    /// Hover row background inside the panel.
    pub hover_bg: Color,
    /// Trigger background.
    pub trigger_bg: Color,
    /// Focus accent: the trigger's border travels to these stops.
    pub ring: GradientStops,
}

impl PopoverColors {
    pub(super) fn slots_mut(&mut self) -> NamedSlots<'_, 6> {
        [
            ("bg", &mut self.bg),
            ("border", &mut self.border),
            ("text", &mut self.text),
            ("muted_text", &mut self.muted_text),
            ("hover_bg", &mut self.hover_bg),
            ("trigger_bg", &mut self.trigger_bg),
        ]
    }
}

/// Radio groups: one 20px circle in its two states.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RadioColors {
    /// The selected circle's ring, projected top to bottom.
    pub accent: GradientStops,
    /// Interior of an unselected circle.
    pub unselected_bg: Color,
    /// 1px rim of an unselected circle; also the ring's start color.
    pub unselected_border: Color,
    /// The hole punched out of the selected circle's ring.
    pub hole: Color,
    /// Row label.
    pub text: Color,
}

impl RadioColors {
    pub(super) fn slots_mut(&mut self) -> NamedSlots<'_, 4> {
        [
            ("unselected_bg", &mut self.unselected_bg),
            ("unselected_border", &mut self.unselected_border),
            ("hole", &mut self.hole),
            ("text", &mut self.text),
        ]
    }
}

/// Context-menu rows: denser than a popover, selected by a filled accent bar.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MenuColors {
    /// Panel background.
    pub bg: Color,
    /// Panel border.
    pub border: Color,
    /// Row label.
    pub text: Color,
    /// Trailing keyboard shortcut.
    pub shortcut_text: Color,
    /// Label and shortcut of a disabled row.
    pub disabled_text: Color,
    /// Small uppercase section header.
    pub header_text: Color,
    /// Hairline between sections.
    pub separator: Color,
    /// Fill behind the highlighted row.
    pub selected_bg: Color,
    /// Label, icon and shortcut of the highlighted row.
    pub selected_text: Color,
}

impl MenuColors {
    pub(super) fn slots_mut(&mut self) -> NamedSlots<'_, 9> {
        [
            ("bg", &mut self.bg),
            ("border", &mut self.border),
            ("text", &mut self.text),
            ("shortcut_text", &mut self.shortcut_text),
            ("disabled_text", &mut self.disabled_text),
            ("header_text", &mut self.header_text),
            ("separator", &mut self.separator),
            ("selected_bg", &mut self.selected_bg),
            ("selected_text", &mut self.selected_text),
        ]
    }
}

/// The sidebar layout, which has three interpolated row states.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SidebarColors {
    /// The column's own fill.
    pub bg: Color,
    /// Hairline between groups of rows.
    pub separator: Color,
    /// An idle top-level row's label.
    pub text: Color,
    /// A selected row's label.
    pub selected_text: Color,
    /// An idle sub-item's label.
    pub subitem_text: Color,
    /// The brand row's label and icon.
    pub brand_text: Color,
    /// An idle row's leading icon.
    pub icon: Color,
    /// A selected row's leading icon.
    pub selected_icon: Color,
    /// Hover wash over an idle row.
    pub hover_bg: Color,
    /// The selected row's raised pill fill.
    pub selected_bg: Color,
    /// Hairline around that pill.
    pub selected_border: Color,
}

impl SidebarColors {
    pub(super) fn slots_mut(&mut self) -> NamedSlots<'_, 11> {
        [
            ("bg", &mut self.bg),
            ("separator", &mut self.separator),
            ("text", &mut self.text),
            ("selected_text", &mut self.selected_text),
            ("subitem_text", &mut self.subitem_text),
            ("brand_text", &mut self.brand_text),
            ("icon", &mut self.icon),
            ("selected_icon", &mut self.selected_icon),
            ("hover_bg", &mut self.hover_bg),
            ("selected_bg", &mut self.selected_bg),
            ("selected_border", &mut self.selected_border),
        ]
    }
}
