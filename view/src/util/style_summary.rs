use geometry::Length;
use layouting::{Dimension, Display, FlexDirection, Overflow, Position};

use crate::style::NodeStyle;

/// A Tailwind-like one-line description of a style, for devtools and diagnostics.
pub(crate) fn summarize(style: &NodeStyle) -> String {
    let layout = &style.layout;
    let mut out = String::new();
    let mut push = |text: String| {
        if !out.is_empty() {
            out.push(' ');
        }
        out.push_str(&text);
    };
    if layout.display == Display::Grid {
        push(format!("grid-cols-{}", layout.grid_columns.len()));
    } else if layout.flex_direction == FlexDirection::Row {
        push("flex-row".into());
    }
    if layout.position == Position::Absolute {
        push("absolute".into());
    }
    let padding = layout.padding;
    if padding.top == padding.bottom
        && padding.left == padding.right
        && padding.top == padding.left
        && padding.top > 0.0
    {
        push(format!("p-{}", Length(padding.top)));
    } else if padding.horizontal() > 0.0 || padding.vertical() > 0.0 {
        push(format!(
            "pt-{} pr-{} pb-{} pl-{}",
            Length(padding.top),
            Length(padding.right),
            Length(padding.bottom),
            Length(padding.left)
        ));
    }
    if layout.gap != (0.0, 0.0) {
        push(format!(
            "gap-{}/{}",
            Length(layout.gap.0),
            Length(layout.gap.1)
        ));
    }
    for (prefix, dimension) in [("w", layout.width), ("h", layout.height)] {
        match dimension {
            Dimension::Length(length) => push(format!("{prefix}-{}", Length(length))),
            Dimension::Percent(fraction) => push(format!("{prefix}-{}%", fraction * 100.0)),
            Dimension::Auto => {}
        }
    }
    if layout.flex_grow > 0.0 {
        push(format!("grow-{}", layout.flex_grow));
    }
    if layout.overflow != Overflow::Visible {
        push(format!("overflow-{:?}", layout.overflow).to_lowercase());
    }
    let visual = &style.visual;
    if let Some(color) = visual.background {
        let [red, green, blue, alpha] = color.to_rgba8();
        push(format!("bg-#{red:02x}{green:02x}{blue:02x}{alpha:02x}"));
    }
    if visual.radii.max() > 0.0 {
        push(format!("rounded-{}", Length(visual.radii.max())));
    }
    if visual.opacity < 1.0 {
        push(format!("opacity-{}", Length(visual.opacity)));
    }
    out
}
