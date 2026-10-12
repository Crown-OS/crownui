use geometry::Color;
use semantics::Role;

use crate::kit::{SidebarColors, TEXT_SIZE};

pub(super) const ROW_HEIGHT: f32 = 36.0;
pub(super) const ROW_RADIUS: f32 = 16.0;
pub(super) const HORIZONTAL_PADDING: f32 = 10.0;
pub(super) const ICON_GAP: f32 = 10.0;
pub(super) const TRAILING_SIZE: f32 = 16.0;
pub(super) const LEADING_STROKE: f32 = 1.8;
pub(super) const TRAILING_STROKE: f32 = 1.6;
const ICON_SIZE: f32 = 18.0;
const BRAND_ICON_SIZE: f32 = 22.0;
const BRAND_HEIGHT: f32 = 44.0;
const BRAND_TEXT_SIZE: f32 = TEXT_SIZE + 1.0;
const SUBITEM_INDENT: f32 = 20.0;

/// How a sidebar row presents itself; one paint path serves all four.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SidebarItemVariant {
    /// A top-level navigation row with a leading icon.
    Item,
    /// An indented row without an icon, nested under a group.
    Subitem,
    /// A group's header: leading icon and a trailing chevron, "selected" while the group is open.
    GroupHeader,
    /// The top brand row: a bigger icon and label, no pill.
    Brand,
}

impl SidebarItemVariant {
    pub(super) const fn height(self) -> f32 {
        match self {
            Self::Brand => BRAND_HEIGHT,
            _ => ROW_HEIGHT,
        }
    }

    pub(super) const fn text_size(self) -> f32 {
        match self {
            Self::Brand => BRAND_TEXT_SIZE,
            _ => TEXT_SIZE,
        }
    }

    pub(super) const fn icon_size(self) -> f32 {
        match self {
            Self::Brand => BRAND_ICON_SIZE,
            _ => ICON_SIZE,
        }
    }

    pub(super) const fn leading_padding(self) -> f32 {
        match self {
            Self::Subitem => HORIZONTAL_PADDING + SUBITEM_INDENT,
            _ => HORIZONTAL_PADDING,
        }
    }

    pub(super) const fn has_pill(self) -> bool {
        !matches!(self, Self::Brand)
    }

    pub(super) const fn role(self) -> Role {
        match self {
            Self::Brand => Role::Group,
            Self::GroupHeader => Role::Button,
            Self::Item | Self::Subitem => Role::MenuItem,
        }
    }

    pub(super) const fn text_color(self, colors: &SidebarColors, selected: bool) -> Color {
        match (self, selected) {
            (Self::Brand, _) => colors.brand_text,
            (_, true) => colors.selected_text,
            (Self::Subitem, false) => colors.subitem_text,
            (_, false) => colors.text,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kit::Theme;

    #[test]
    fn only_the_brand_row_is_taller_and_pill_less() {
        assert_eq!(SidebarItemVariant::Item.height(), 36.0);
        assert_eq!(SidebarItemVariant::Brand.height(), 44.0);
        assert!(SidebarItemVariant::GroupHeader.has_pill());
        assert!(!SidebarItemVariant::Brand.has_pill());
    }

    #[test]
    fn subitems_indent_their_label_by_twenty_pixels() {
        assert_eq!(SidebarItemVariant::Subitem.leading_padding(), 30.0);
        assert_eq!(SidebarItemVariant::Item.leading_padding(), 10.0);
    }

    #[test]
    fn the_label_color_follows_variant_and_selection() {
        let colors = Theme::dark().sidebar;
        assert_eq!(
            SidebarItemVariant::Item.text_color(&colors, false),
            colors.text
        );
        assert_eq!(
            SidebarItemVariant::Subitem.text_color(&colors, false),
            colors.subitem_text
        );
        assert_eq!(
            SidebarItemVariant::Subitem.text_color(&colors, true),
            colors.selected_text
        );
        assert_eq!(
            SidebarItemVariant::Brand.text_color(&colors, true),
            colors.brand_text
        );
    }
}
