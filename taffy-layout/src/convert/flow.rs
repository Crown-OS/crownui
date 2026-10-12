use layouting::{Display, FlexDirection, Overflow, Position};

pub(super) fn display(display: Display) -> taffy::Display {
    match display {
        Display::Flex => taffy::Display::Flex,
        Display::Grid => taffy::Display::Grid,
        Display::None => taffy::Display::None,
    }
}

pub(super) fn position(position: Position) -> taffy::Position {
    match position {
        Position::Relative => taffy::Position::Relative,
        Position::Absolute => taffy::Position::Absolute,
    }
}

pub(super) fn flex_direction(direction: FlexDirection) -> taffy::FlexDirection {
    match direction {
        FlexDirection::Row => taffy::FlexDirection::Row,
        FlexDirection::Column => taffy::FlexDirection::Column,
        FlexDirection::RowReverse => taffy::FlexDirection::RowReverse,
        FlexDirection::ColumnReverse => taffy::FlexDirection::ColumnReverse,
    }
}

pub(super) fn flex_wrap(wraps: bool) -> taffy::FlexWrap {
    if wraps {
        taffy::FlexWrap::Wrap
    } else {
        taffy::FlexWrap::NoWrap
    }
}

pub(super) fn overflow(overflow: Overflow) -> taffy::Point<taffy::Overflow> {
    let axis = match overflow {
        Overflow::Visible => taffy::Overflow::Visible,
        Overflow::Clip => taffy::Overflow::Clip,
        Overflow::Scroll => taffy::Overflow::Scroll,
    };
    taffy::Point { x: axis, y: axis }
}
