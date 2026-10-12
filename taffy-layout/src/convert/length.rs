use geometry::Insets;
use layouting::Dimension;
use taffy::{LengthPercentage, LengthPercentageAuto, Rect};

pub(super) fn size_dimension(dimension: Dimension) -> taffy::Dimension {
    match dimension {
        Dimension::Auto => taffy::Dimension::auto(),
        Dimension::Length(length) => taffy::Dimension::length(length),
        Dimension::Percent(fraction) => taffy::Dimension::percent(fraction),
    }
}

pub(super) fn length_percentage_auto(dimension: Dimension) -> LengthPercentageAuto {
    match dimension {
        Dimension::Auto => LengthPercentageAuto::auto(),
        Dimension::Length(length) => LengthPercentageAuto::length(length),
        Dimension::Percent(fraction) => LengthPercentageAuto::percent(fraction),
    }
}

pub(super) fn padding_rect(insets: Insets) -> Rect<LengthPercentage> {
    Rect {
        left: LengthPercentage::length(insets.left),
        right: LengthPercentage::length(insets.right),
        top: LengthPercentage::length(insets.top),
        bottom: LengthPercentage::length(insets.bottom),
    }
}

pub(super) fn margin_rect(insets: Insets) -> Rect<LengthPercentageAuto> {
    Rect {
        left: LengthPercentageAuto::length(insets.left),
        right: LengthPercentageAuto::length(insets.right),
        top: LengthPercentageAuto::length(insets.top),
        bottom: LengthPercentageAuto::length(insets.bottom),
    }
}

pub(super) fn inset_rect([top, right, bottom, left]: [Dimension; 4]) -> Rect<LengthPercentageAuto> {
    Rect {
        left: length_percentage_auto(left),
        right: length_percentage_auto(right),
        top: length_percentage_auto(top),
        bottom: length_percentage_auto(bottom),
    }
}
