use layouting::LayoutStyle;
use taffy::{LengthPercentage, Size, Style};

use super::{alignment, flow, grid, length};

pub(crate) fn taffy_style(style: &LayoutStyle) -> Style {
    let (column_gap, row_gap) = style.gap;
    Style {
        display: flow::display(style.display),
        position: flow::position(style.position),
        inset: length::inset_rect(style.inset),
        size: Size {
            width: length::size_dimension(style.width),
            height: length::size_dimension(style.height),
        },
        min_size: Size {
            width: length::length_percentage_auto(style.min_width),
            height: length::length_percentage_auto(style.min_height),
        },
        max_size: Size {
            width: length::length_percentage_auto(style.max_width),
            height: length::length_percentage_auto(style.max_height),
        },
        aspect_ratio: style.aspect_ratio,
        padding: length::padding_rect(style.padding),
        margin: length::margin_rect(style.margin),
        gap: Size {
            width: LengthPercentage::length(column_gap),
            height: LengthPercentage::length(row_gap),
        },
        flex_direction: flow::flex_direction(style.flex_direction),
        flex_wrap: flow::flex_wrap(style.flex_wrap),
        align_items: style.align_items.map(alignment::align_items),
        align_self: style.align_self.map(alignment::align_items),
        justify_content: style.justify_content.map(alignment::justify_content),
        flex_grow: style.flex_grow,
        flex_shrink: style.flex_shrink,
        flex_basis: length::size_dimension(style.flex_basis),
        grid_template_columns: grid::template_tracks(&style.grid_columns),
        grid_template_rows: grid::template_tracks(&style.grid_rows),
        grid_column: grid::span_placement(style.column_span),
        grid_row: grid::span_placement(style.row_span),
        overflow: flow::overflow(style.overflow),
        scrollbar_width: 0.0,
        ..Style::DEFAULT
    }
}
