use layouting::Dimension;
use view::{Modifier, SizeProperty, StyleProp, Styled};

/// Shrink-wrapping that never outgrows the parent, so wrapped text still wraps.
pub(crate) trait FitParent: Styled {
    fn fit_parent_width(self) -> Self {
        self.modifier(Modifier::Style(StyleProp::Size(
            SizeProperty::MaxWidth,
            Dimension::Percent(1.0),
        )))
    }
}

impl<T: Styled> FitParent for T {}
