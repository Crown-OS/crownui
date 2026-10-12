use geometry::{Color, Vector};
use graphics::{FontWeight, TextAlign};
use layouting::{Align, Dimension, Display, FlexDirection, GridTrack, Justify, Overflow, Position};
use windowing::{CursorIcon, Material};

use crate::{
    modifier::Modifier,
    prop::IntoProp,
    style::Shadow,
    style_prop::{Corners, Edges, SizeProperty, StyleProp},
};

macro_rules! length_methods {
    ($($(#[$doc:meta])* $name:ident => $make:expr;)+) => {
        $($(#[$doc])* fn $name(self, length: impl IntoProp<f32>) -> Self {
            let make: fn(f32) -> StyleProp = $make;
            self.modifier(Modifier::from_prop(length.into_prop(), make))
        })+
    };
}

macro_rules! flag_methods {
    ($($(#[$doc:meta])* $name:ident => $prop:expr;)+) => {
        $($(#[$doc])* fn $name(self) -> Self {
            self.modifier(Modifier::Style($prop))
        })+
    };
}

/// Style methods named after Tailwind utility classes, taking literal values in logical pixels
/// (or signals and [`bind`](crate::bind) closures for reactive values).
pub trait Styled: Sized {
    /// Appends a modifier; every other method is built on this.
    fn modifier(self, modifier: Modifier) -> Self;

    length_methods! {
        /// `p-*`: padding on every edge.
        p => |value| StyleProp::Padding(Edges::All, value);
        /// `px-*`: left and right padding.
        px => |value| StyleProp::Padding(Edges::Horizontal, value);
        /// `py-*`: top and bottom padding.
        py => |value| StyleProp::Padding(Edges::Vertical, value);
        /// `pt-*`.
        pt => |value| StyleProp::Padding(Edges::Top, value);
        /// `pr-*`.
        pr => |value| StyleProp::Padding(Edges::Right, value);
        /// `pb-*`.
        pb => |value| StyleProp::Padding(Edges::Bottom, value);
        /// `pl-*`.
        pl => |value| StyleProp::Padding(Edges::Left, value);
        /// `m-*`: margin on every edge.
        m => |value| StyleProp::Margin(Edges::All, value);
        /// `mx-*`.
        mx => |value| StyleProp::Margin(Edges::Horizontal, value);
        /// `my-*`.
        my => |value| StyleProp::Margin(Edges::Vertical, value);
        /// `mt-*`.
        mt => |value| StyleProp::Margin(Edges::Top, value);
        /// `mr-*`.
        mr => |value| StyleProp::Margin(Edges::Right, value);
        /// `mb-*`.
        mb => |value| StyleProp::Margin(Edges::Bottom, value);
        /// `ml-*`.
        ml => |value| StyleProp::Margin(Edges::Left, value);
        /// `w-*`: width.
        w => |value| StyleProp::Size(SizeProperty::Width, Dimension::Length(value));
        /// `h-*`: height.
        h => |value| StyleProp::Size(SizeProperty::Height, Dimension::Length(value));
        /// `min-w-*`.
        min_w => |value| StyleProp::Size(SizeProperty::MinWidth, Dimension::Length(value));
        /// `min-h-*`.
        min_h => |value| StyleProp::Size(SizeProperty::MinHeight, Dimension::Length(value));
        /// `max-w-*`.
        max_w => |value| StyleProp::Size(SizeProperty::MaxWidth, Dimension::Length(value));
        /// `max-h-*`.
        max_h => |value| StyleProp::Size(SizeProperty::MaxHeight, Dimension::Length(value));
        /// `aspect-*`: width divided by height.
        aspect => StyleProp::AspectRatio;
        /// `inset-*`: offsets on every edge (with `absolute`).
        inset => |value| StyleProp::Inset(Edges::All, value);
        /// `top-*`.
        top => |value| StyleProp::Inset(Edges::Top, value);
        /// `right-*`.
        right => |value| StyleProp::Inset(Edges::Right, value);
        /// `bottom-*`.
        bottom => |value| StyleProp::Inset(Edges::Bottom, value);
        /// `left-*`.
        left => |value| StyleProp::Inset(Edges::Left, value);
        /// `gap-*`: gap between rows and columns.
        gap => |value| StyleProp::Gap(Some(value), Some(value));
        /// `gap-x-*`: gap between columns.
        gap_x => |value| StyleProp::Gap(Some(value), None);
        /// `gap-y-*`: gap between rows.
        gap_y => |value| StyleProp::Gap(None, Some(value));
        /// `grow` / `grow-*`.
        grow => StyleProp::Grow;
        /// `shrink` / `shrink-*`.
        shrink => StyleProp::Shrink;
        /// `basis-*`.
        basis => |value| StyleProp::Basis(Dimension::Length(value));
        /// `border-*`: border width.
        border => StyleProp::BorderWidth;
        /// `rounded-*`: every corner.
        rounded => |value| StyleProp::Radius(Corners::All, value);
        /// `rounded-t-*`.
        rounded_t => |value| StyleProp::Radius(Corners::Top, value);
        /// `rounded-r-*`.
        rounded_r => |value| StyleProp::Radius(Corners::Right, value);
        /// `rounded-b-*`.
        rounded_b => |value| StyleProp::Radius(Corners::Bottom, value);
        /// `rounded-l-*`.
        rounded_l => |value| StyleProp::Radius(Corners::Left, value);
        /// `rounded-tl-*`.
        rounded_tl => |value| StyleProp::Radius(Corners::TopLeft, value);
        /// `rounded-tr-*`.
        rounded_tr => |value| StyleProp::Radius(Corners::TopRight, value);
        /// `rounded-br-*`.
        rounded_br => |value| StyleProp::Radius(Corners::BottomRight, value);
        /// `rounded-bl-*`.
        rounded_bl => |value| StyleProp::Radius(Corners::BottomLeft, value);
        /// `opacity-*`, as a fraction.
        opacity => StyleProp::Opacity;
        /// `translate-x-*`.
        translate_x => |value| StyleProp::Translate(Vector::new(value, 0.0));
        /// `translate-y-*`.
        translate_y => |value| StyleProp::Translate(Vector::new(0.0, value));
        /// `scale-*`, as a factor.
        scale => StyleProp::Scale;
        /// `scale-x-*`: horizontal scale only.
        scale_x => StyleProp::ScaleX;
        /// `scale-y-*`: vertical scale only.
        scale_y => StyleProp::ScaleY;
        /// `rotate-*`, in degrees.
        rotate => StyleProp::Rotate;
        /// `backdrop-blur-*`: blurs what is behind the node.
        backdrop_blur => StyleProp::BackdropBlur;
        /// `text-*` (size): font size.
        text_size => StyleProp::TextSize;
        /// `leading-*`: line height as a multiple of the font size.
        leading => StyleProp::LineHeight;
        /// `tracking-*`: letter spacing.
        tracking => StyleProp::Tracking;
    }

    flag_methods! {
        /// `w-full`.
        w_full => StyleProp::Size(SizeProperty::Width, Dimension::Percent(1.0));
        /// `h-full`.
        h_full => StyleProp::Size(SizeProperty::Height, Dimension::Percent(1.0));
        /// `flex` + `flex-row`.
        flex_row => StyleProp::FlexDirection(FlexDirection::Row);
        /// `flex` + `flex-col`.
        flex_col => StyleProp::FlexDirection(FlexDirection::Column);
        /// `flex-row-reverse`.
        flex_row_reverse => StyleProp::FlexDirection(FlexDirection::RowReverse);
        /// `flex-col-reverse`.
        flex_col_reverse => StyleProp::FlexDirection(FlexDirection::ColumnReverse);
        /// `flex-wrap`.
        flex_wrap => StyleProp::FlexWrap(true);
        /// `items-start`.
        items_start => StyleProp::AlignItems(Align::Start);
        /// `items-center`.
        items_center => StyleProp::AlignItems(Align::Center);
        /// `items-end`.
        items_end => StyleProp::AlignItems(Align::End);
        /// `items-stretch`.
        items_stretch => StyleProp::AlignItems(Align::Stretch);
        /// `items-baseline`.
        items_baseline => StyleProp::AlignItems(Align::Baseline);
        /// `self-start`.
        self_start => StyleProp::AlignSelf(Align::Start);
        /// `self-center`.
        self_center => StyleProp::AlignSelf(Align::Center);
        /// `self-end`.
        self_end => StyleProp::AlignSelf(Align::End);
        /// `self-stretch`.
        self_stretch => StyleProp::AlignSelf(Align::Stretch);
        /// `justify-start`.
        justify_start => StyleProp::Justify(Justify::Start);
        /// `justify-center`.
        justify_center => StyleProp::Justify(Justify::Center);
        /// `justify-end`.
        justify_end => StyleProp::Justify(Justify::End);
        /// `justify-between`.
        justify_between => StyleProp::Justify(Justify::SpaceBetween);
        /// `justify-around`.
        justify_around => StyleProp::Justify(Justify::SpaceAround);
        /// `justify-evenly`.
        justify_evenly => StyleProp::Justify(Justify::SpaceEvenly);
        /// `absolute`.
        absolute => StyleProp::Position(Position::Absolute);
        /// `relative`.
        relative => StyleProp::Position(Position::Relative);
        /// `grid`.
        grid => StyleProp::Display(Display::Grid);
        /// Tailwind's `hidden` (`display: none`): removed from layout, taking no space or gap.
        /// [`Styled::hidden`] keeps the space and only stops painting.
        display_none => StyleProp::Display(Display::None);
        /// `overflow-hidden`.
        overflow_hidden => StyleProp::Overflow(Overflow::Clip);
        /// `overflow-scroll`.
        overflow_scroll => StyleProp::Overflow(Overflow::Scroll);
        /// `italic`.
        italic => StyleProp::Italic(true);
        /// `font-medium`.
        font_medium => StyleProp::FontWeight(FontWeight::MEDIUM);
        /// `font-semibold`.
        font_semibold => StyleProp::FontWeight(FontWeight::SEMIBOLD);
        /// `font-bold`.
        font_bold => StyleProp::FontWeight(FontWeight::BOLD);
        /// `text-left`.
        text_left => StyleProp::TextAlign(TextAlign::Start);
        /// `text-center`.
        text_center => StyleProp::TextAlign(TextAlign::Center);
        /// `text-right`.
        text_right => StyleProp::TextAlign(TextAlign::End);
        /// `text-justify`.
        text_justify => StyleProp::TextAlign(TextAlign::Justify);
        /// `truncate`: one line with an ellipsis.
        truncate => StyleProp::Truncate(true);
    }

    /// `flex-1`: grow and shrink from a zero basis.
    fn flex_1(self) -> Self {
        self.grow(1.0).shrink(1.0).basis(0.0)
    }

    /// `size-*`: width and height.
    fn size(self, length: f32) -> Self {
        self.w(length).h(length)
    }

    /// `w-1/2` and friends: width as a fraction of the parent.
    fn w_frac(self, fraction: f32) -> Self {
        self.modifier(Modifier::Style(StyleProp::Size(
            SizeProperty::Width,
            Dimension::Percent(fraction),
        )))
    }

    /// `h-1/2` and friends: height as a fraction of the parent.
    fn h_frac(self, fraction: f32) -> Self {
        self.modifier(Modifier::Style(StyleProp::Size(
            SizeProperty::Height,
            Dimension::Percent(fraction),
        )))
    }

    /// `grid-cols-*`: equal columns.
    fn grid_cols(self, count: u16) -> Self {
        let tracks = vec![GridTrack::Fraction(1.0); usize::from(count.max(1))];
        self.modifier(Modifier::Style(StyleProp::GridColumns(tracks)))
    }

    /// `grid-template-columns` with explicit tracks.
    fn grid_template_cols(self, tracks: Vec<GridTrack>) -> Self {
        self.modifier(Modifier::Style(StyleProp::GridColumns(tracks)))
    }

    /// `col-span-*`.
    fn col_span(self, span: u16) -> Self {
        self.modifier(Modifier::Style(StyleProp::ColumnSpan(span)))
    }

    /// `row-span-*`.
    fn row_span(self, span: u16) -> Self {
        self.modifier(Modifier::Style(StyleProp::RowSpan(span)))
    }

    /// `bg-*`: background color.
    fn bg(self, color: impl IntoProp<Color>) -> Self {
        self.modifier(Modifier::from_prop(color.into_prop(), |color| {
            StyleProp::Background(Some(color))
        }))
    }

    /// `border-*` (color).
    fn border_color(self, color: impl IntoProp<Color>) -> Self {
        self.modifier(Modifier::from_prop(
            color.into_prop(),
            StyleProp::BorderColor,
        ))
    }

    /// `text-*` (color).
    fn text_color(self, color: impl IntoProp<Color>) -> Self {
        self.modifier(Modifier::from_prop(color.into_prop(), StyleProp::TextColor))
    }

    /// `ring-*`: an outline outside the border.
    fn ring(self, width: f32, color: impl IntoProp<Color>) -> Self {
        let ring = color
            .into_prop()
            .map(move |color| StyleProp::Ring(Some((width, color))));
        self.modifier(Modifier::from_style(ring))
    }

    /// `outline-*`: same as a ring.
    fn outline(self, width: f32, color: Color) -> Self {
        self.ring(width, color)
    }

    /// `shadow-*`: a drop shadow (`Shadow::md()` …).
    fn shadow(self, shadow: Shadow) -> Self {
        self.modifier(Modifier::Style(StyleProp::Shadow(Some(shadow))))
    }

    /// `shadow-none`.
    fn shadow_none(self) -> Self {
        self.modifier(Modifier::Style(StyleProp::Shadow(None)))
    }

    /// `translate-*` on both axes.
    fn translate(self, offset: impl IntoProp<Vector>) -> Self
    where
        Vector: 'static,
    {
        self.modifier(Modifier::from_prop(
            offset.into_prop(),
            StyleProp::Translate,
        ))
    }

    /// `origin-*`: transform origin as a fraction of the size.
    fn origin(self, x: f32, y: f32) -> Self {
        self.modifier(Modifier::Style(StyleProp::Origin(Vector::new(x, y))))
    }

    /// `font-*` (weight) on the 100–900 scale.
    fn font_weight(self, weight: u16) -> Self {
        self.modifier(Modifier::Style(StyleProp::FontWeight(FontWeight(weight))))
    }

    /// `font-*` (family).
    fn font_family(self, family: impl Into<String>) -> Self {
        self.modifier(Modifier::Style(StyleProp::FontFamily(family.into())))
    }

    /// `line-clamp-*`.
    fn line_clamp(self, lines: u32) -> Self {
        self.modifier(Modifier::Style(StyleProp::LineClamp(Some(lines))))
    }

    /// `hidden` / visible.
    fn hidden(self, hidden: impl IntoProp<bool>) -> Self {
        self.modifier(Modifier::from_prop(hidden.into_prop(), StyleProp::Hidden))
    }

    /// `z-*`.
    fn z(self, index: i16) -> Self {
        self.modifier(Modifier::Style(StyleProp::ZIndex(index)))
    }

    /// `cursor-*`.
    fn cursor(self, cursor: CursorIcon) -> Self {
        self.modifier(Modifier::Style(StyleProp::Cursor(cursor)))
    }

    /// CrownOS extra: a compositor-rendered material (frosted glass) behind the node; may be bound
    /// to fade the blur in and out with the node.
    fn material(self, material: impl IntoProp<Material>) -> Self {
        self.modifier(Modifier::from_prop(material.into_prop(), |material| {
            StyleProp::Material(Some(material))
        }))
    }

    /// `pointer-events-none`: the pointer passes through this node and, unless they opt back in,
    /// its descendants. On a window's root it also limits the surface's input region.
    fn pointer_events_none(self) -> Self {
        self.modifier(Modifier::Style(StyleProp::PointerEvents(
            crate::PointerEvents::None,
        )))
    }

    /// `pointer-events-auto`: opts a node back into pointer events under a `pointer-events-none`
    /// ancestor.
    fn pointer_events_auto(self) -> Self {
        self.modifier(Modifier::Style(StyleProp::PointerEvents(
            crate::PointerEvents::Auto,
        )))
    }
}

impl IntoProp<Vector> for Vector {
    fn into_prop(self) -> crate::prop::Prop<Vector> {
        crate::prop::Prop::Static(self)
    }
}

impl IntoProp<Material> for Material {
    fn into_prop(self) -> crate::prop::Prop<Material> {
        crate::prop::Prop::Static(self)
    }
}
