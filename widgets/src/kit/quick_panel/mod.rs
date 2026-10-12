mod action;
mod header;
mod hover;
mod item;
mod metrics;
mod palette;
mod readout;
mod section;
mod separator;
mod slider_row;
mod surface;
mod typography;

pub use action::PanelActionKind;
pub use header::{PanelHeaderKind, PanelHeaderOptions};
pub use item::{PanelItemKind, PanelItemOptions};
pub use metrics::{PANEL_RADIUS, PANEL_WIDTH};
pub use palette::{panel_fill, panel_rim};
pub use readout::PanelReadoutKind;
pub use section::{PanelSectionKind, PanelSectionOptions};
pub use separator::PanelSeparatorKind;
pub use slider_row::{PanelSliderKind, PanelSliderOptions};
pub use surface::QuickPanelKind;
pub use typography::PanelGlyph;

use reactive::Signal;
use semantics::Role;
use view::{Element, IntoProp, Semantic, ViewSeq};

/// A control-centre panel: a fixed-width, translucent rounded card that a status bar or dock opens
/// over the desktop, holding the `panel_*` rows below. Pair it with a platform material so the
/// compositor blurs what is behind it.
pub fn quick_panel<C: ViewSeq>(rows: C) -> Element<QuickPanelKind<C>> {
    Element::new(QuickPanelKind { rows }).role(Role::Group)
}

/// The panel's title row, optionally with its master switch.
pub fn panel_header(title: impl Into<String>) -> Element<PanelHeaderKind> {
    Element::new(PanelHeaderKind {
        title: title.into(),
        switch: None,
    })
}

/// A continuous control bound to `value` in `0.0..=1.0`, with a glyph column at its left.
pub fn panel_slider(label: impl Into<String>, value: Signal<f32>) -> Element<PanelSliderKind> {
    Element::new(PanelSliderKind {
        value,
        label: label.into(),
        glyph: None,
    })
}

/// A small muted heading above a group of rows ("Output", "Known Networks").
pub fn panel_section(title: impl Into<String>) -> Element<PanelSectionKind> {
    let title = title.into();
    Element::new(PanelSectionKind {
        title: title.clone(),
        disclosure: false,
    })
    .label(title)
}

/// A device or network row: glyph, label, then optional detail and trailing views.
pub fn panel_item(label: impl Into<String>) -> Element<PanelItemKind> {
    let label = label.into();
    Element::new(PanelItemKind {
        label: label.clone(),
        glyph: None,
        badge: false,
        detail: None,
        trailing: Vec::new(),
        selected: view::Prop::Static(false),
        enabled: true,
        chevron: false,
    })
    .role(Role::Button)
    .label(label)
}

/// A footer row that hands off elsewhere ("Sound Settings…").
pub fn panel_action(label: impl Into<String>) -> Element<PanelActionKind> {
    let label = label.into();
    Element::new(PanelActionKind {
        label: label.clone(),
    })
    .label(label)
}

/// A large reading with a line beneath it, like a clock's time over its date.
pub fn panel_readout(
    primary: impl IntoProp<String>,
    secondary: impl IntoProp<String>,
) -> Element<PanelReadoutKind> {
    Element::new(readout::readout_kind(primary, secondary))
}

/// A hairline between groups of rows.
pub fn panel_separator() -> Element<PanelSeparatorKind> {
    Element::new(PanelSeparatorKind)
}
