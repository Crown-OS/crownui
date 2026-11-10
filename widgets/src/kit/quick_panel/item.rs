use geometry::{Color, CornerRadii, Point, Rect, TRANSPARENT};
use reactive::Memo;
use view::{AnyView, Cx, Element, IntoProp, Kind, NodeId, Prop, Styled, View};
use windowing::CursorIcon;

use super::{
    badge::{on_disc, plain},
    hover::track_hover,
    metrics::{
        BADGE_GLYPH, BODY_TEXT, CHEVRON, CHEVRON_RIGHT, DETAIL_TEXT, ICON_GAP, ICON_INSET,
        ITEM_HEIGHT, PAD_X, PLAIN_GLYPH, ROW_INSET, ROW_RADIUS, TRAILING_GAP,
    },
    palette::{badge_glyph_selected, badge_idle, badge_selected, row_selected},
    typography::{PanelGlyph, memo_of, role_text},
};
use crate::{
    canvas::canvas,
    kit::{Theme, icon::tinted_icon, use_theme},
    stack::hstack,
    util::themed::themed,
};

/// The kind behind [`panel_item`](super::panel_item).
pub struct PanelItemKind {
    pub(super) label: String,
    pub(super) glyph: Option<PanelGlyph>,
    pub(super) badge: bool,
    pub(super) detail: Option<Prop<String>>,
    pub(super) trailing: Vec<AnyView>,
    pub(super) selected: Prop<bool>,
    pub(super) enabled: bool,
    pub(super) chevron: bool,
}

/// The options of a [`panel_item`](super::panel_item), chained before its listeners.
pub trait PanelItemOptions: Sized {
    /// A glyph in a filled circle at the row's left, the way a device is marked.
    fn badge(self, glyph: impl FnOnce(Memo<Color>) -> AnyView + 'static) -> Self;
    /// A bare glyph at the row's left, where a badge's circle would be.
    fn glyph(self, glyph: impl FnOnce(Memo<Color>) -> AnyView + 'static) -> Self;
    /// Right-aligned secondary text, e.g. "70%".
    fn detail(self, detail: impl IntoProp<String>) -> Self;
    /// Views after the detail, e.g. a battery cell or a warning sign.
    fn trailing(self, view: impl View + 'static) -> Self;
    /// Marks the current choice (the default output, the joined network) with an accent capsule
    /// and, on a badged row, an accent badge.
    fn selected(self, selected: impl IntoProp<bool>) -> Self;
    /// A disclosure chevron at the right edge.
    fn chevron(self) -> Self;
    /// A dimmed, inert row, e.g. a status line.
    fn inert(self) -> Self;
}

impl PanelItemOptions for Element<PanelItemKind> {
    fn badge(self, glyph: impl FnOnce(Memo<Color>) -> AnyView + 'static) -> Self {
        let glyph: PanelGlyph = Box::new(glyph);
        self.configure(|kind| (kind.glyph, kind.badge) = (Some(glyph), true))
    }

    fn glyph(self, glyph: impl FnOnce(Memo<Color>) -> AnyView + 'static) -> Self {
        let glyph: PanelGlyph = Box::new(glyph);
        self.configure(|kind| (kind.glyph, kind.badge) = (Some(glyph), false))
    }

    fn detail(self, detail: impl IntoProp<String>) -> Self {
        self.configure(|kind| kind.detail = Some(detail.into_prop()))
    }

    fn trailing(self, view: impl View + 'static) -> Self {
        self.configure(|kind| kind.trailing.push(view.boxed()))
    }

    fn selected(self, selected: impl IntoProp<bool>) -> Self {
        self.configure(|kind| kind.selected = selected.into_prop())
    }

    fn chevron(self) -> Self {
        self.configure(|kind| kind.chevron = true)
    }

    fn inert(self) -> Self {
        self.configure(|kind| kind.enabled = false)
    }
}

fn hover_fill(theme: &Theme, selected: bool, hovered: bool) -> Color {
    if hovered && !selected {
        theme.popover.hover_bg
    } else {
        TRANSPARENT
    }
}

/// The capsule a chosen row stands on, drawn under its content.
fn selection(cx: &mut Cx, selected: Memo<bool>) -> impl View + use<> {
    let theme = use_theme(cx);
    canvas(move |runtime, bounds, list| {
        if selected.get(runtime) {
            theme.with(runtime, row_selected).paint(
                list,
                Rect::new(Point::ZERO, bounds),
                CornerRadii::uniform(bounds.height / 2.0),
            );
        }
    })
    .absolute()
    .inset(0.0)
}

fn leading(cx: &mut Cx, glyph: PanelGlyph, badge: bool, selected: Memo<bool>) -> AnyView {
    let theme = use_theme(cx);
    let tint = cx.memo(move |runtime| {
        let chosen = badge && selected.get(runtime);
        theme.with(runtime, |theme| {
            if chosen {
                badge_glyph_selected(theme)
            } else {
                theme.popover.text
            }
        })
    });
    let glyph = glyph(tint);
    if !badge {
        return plain(glyph, PLAIN_GLYPH).mr(ICON_GAP).boxed();
    }
    on_disc(theme, glyph, BADGE_GLYPH, move |runtime| {
        if selected.get(runtime) {
            badge_selected
        } else {
            badge_idle
        }
    })
    .mr(ICON_GAP)
    .boxed()
}

impl Kind for PanelItemKind {
    const NAME: &'static str = "kit_panel_item";

    fn mount(self, cx: &mut Cx, node: NodeId) {
        let theme = use_theme(cx);
        let selected = memo_of(cx, self.selected);
        let enabled = self.enabled;
        for prop in [
            view::StyleProp::FlexDirection(layouting::FlexDirection::Row),
            view::StyleProp::AlignItems(layouting::Align::Center),
            view::StyleProp::Margin(view::Edges::Horizontal, ROW_INSET),
            view::StyleProp::Padding(view::Edges::Right, PAD_X - ROW_INSET),
            view::StyleProp::Padding(
                view::Edges::Left,
                if self.glyph.is_some() {
                    ICON_INSET
                } else {
                    PAD_X - ROW_INSET
                },
            ),
            view::StyleProp::Position(layouting::Position::Relative),
            view::StyleProp::Size(
                view::SizeProperty::Height,
                layouting::Dimension::Length(ITEM_HEIGHT),
            ),
            view::StyleProp::Radius(view::Corners::All, ROW_RADIUS),
            view::StyleProp::Shrink(0.0),
        ] {
            cx.set_default_style(node, prop);
        }
        if enabled {
            cx.set_default_style(node, view::StyleProp::Cursor(CursorIcon::Pointer));
        }
        let hovered = enabled.then(|| track_hover(cx, node));
        cx.bind_style(node, move |runtime| {
            let selected = selected.get(runtime);
            let hovered = hovered.is_some_and(|hovered| hovered.get(runtime));
            view::StyleProp::Background(Some(
                theme.with(runtime, |theme| hover_fill(theme, selected, hovered)),
            ))
        });

        let mut children: Vec<AnyView> = Vec::with_capacity(5 + self.trailing.len());
        children.push(selection(cx, selected).boxed());
        if let Some(glyph) = self.glyph {
            children.push(leading(cx, glyph, self.badge, selected));
        }
        let label_color = move |theme: &Theme| {
            if enabled {
                theme.popover.text
            } else {
                theme.popover.muted_text
            }
        };
        children.push(
            role_text(self.label, BODY_TEXT)
                .grow(1.0)
                .shrink(1.0)
                .min_w(0.0)
                .truncate()
                .text_color(themed(theme, label_color))
                .boxed(),
        );
        if let Some(detail) = self.detail {
            children.push(
                role_text(detail, DETAIL_TEXT)
                    .shrink(0.0)
                    .ml(TRAILING_GAP)
                    .text_color(themed(theme, |theme| theme.popover.muted_text))
                    .boxed(),
            );
        }
        children.extend(
            self.trailing
                .into_iter()
                .map(|view| hstack(view).shrink(0.0).ml(TRAILING_GAP).boxed()),
        );
        if self.chevron {
            children.push(
                tinted_icon(
                    CHEVRON_RIGHT,
                    2.0,
                    themed(theme, |theme| theme.popover.muted_text),
                )
                .size(CHEVRON)
                .shrink(0.0)
                .ml(TRAILING_GAP)
                .boxed(),
            );
        }
        cx.set_children(node, children);
    }
}
