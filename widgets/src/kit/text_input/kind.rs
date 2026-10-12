use geometry::Point;
use layouting::{Align, Dimension, FlexDirection, Overflow, Position};
use reactive::Signal;
use semantics::Role;
use view::{
    Corners, Cx, Element, Kind, Listener, NodeId, Semantic, SizeProperty, StyleProp, Styled, bind,
};
use windowing::CursorIcon;

use super::{
    ClearButton, FieldShape,
    chrome::mount_chrome,
    content::{Content, ContentInput, mount_content},
    editor::{Editor, fraction_along},
    metrics::{
        CLEAR_GAP, CLEAR_ICON, DEFAULT_WIDTH, FIELD_HEIGHT, H_PADDING, ICON_STROKE, LEADING_GAP,
        LEADING_ICON,
    },
};
use crate::{
    kit::{INTER_FAMILY, TEXT_SIZE, icon::tinted_icon, icons, use_theme},
    util::text_edit::Selection,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum FieldWidth {
    Fixed(f32),
    Fill,
}

/// The kind behind [`text_input`](super::text_input).
pub struct TextInputKind {
    pub(crate) value: Signal<String>,
    pub(crate) placeholder: String,
    pub(crate) shape: FieldShape,
    pub(crate) leading_icon: Option<&'static str>,
    pub(crate) clear_button: ClearButton,
    pub(crate) width: FieldWidth,
    pub(crate) secret: bool,
    pub(crate) read_only: bool,
    pub(crate) role: Role,
    pub(crate) selection: Option<Signal<Selection>>,
}

impl TextInputKind {
    pub(crate) fn new(value: Signal<String>) -> Self {
        Self {
            value,
            placeholder: String::new(),
            shape: FieldShape::default(),
            leading_icon: None,
            clear_button: ClearButton::default(),
            width: FieldWidth::Fixed(DEFAULT_WIDTH),
            secret: false,
            read_only: false,
            role: Role::TextField,
            selection: None,
        }
    }

    fn text_inset(&self) -> f32 {
        match self.leading_icon {
            Some(_) => H_PADDING + LEADING_ICON + LEADING_GAP,
            None => H_PADDING,
        }
    }

    fn trailing_inset(&self) -> f32 {
        match self.clear_button {
            ClearButton::Visible => H_PADDING + CLEAR_ICON + CLEAR_GAP,
            ClearButton::Hidden => H_PADDING,
        }
    }
}

impl Kind for TextInputKind {
    const NAME: &'static str = "kit_text_input";

    fn mount(self, cx: &mut Cx, node: NodeId) {
        let theme = use_theme(cx);
        let disabled = cx.observe_disabled(node);
        let focused = cx.signal(false);
        cx.add_listener(
            node,
            Listener::Focus(Box::new(move |cx, _| focused.set(cx, true))),
        );
        cx.add_listener(
            node,
            Listener::Blur(Box::new(move |cx, _| focused.set(cx, false))),
        );
        let radius = self.shape.radius();
        let (inset_left, inset_right) = (self.text_inset(), self.trailing_inset());
        let value = self.value;
        let initial_end = value.with(cx, String::len);
        let selection = self
            .selection
            .unwrap_or_else(|| cx.signal(Selection::at(initial_end)));
        let editor = Editor {
            node,
            value,
            selection,
            secret: self.secret,
            read_only: self.read_only,
        };
        let size = cx.observe_size(node);

        cx.set_role(node, self.role);
        cx.bind_value(node, move |runtime| value.get(runtime));
        let (width, align) = match self.width {
            FieldWidth::Fixed(width) => (Dimension::Length(width), Align::Start),
            FieldWidth::Fill => (Dimension::Auto, Align::Stretch),
        };
        for prop in [
            StyleProp::FlexDirection(FlexDirection::Row),
            StyleProp::AlignItems(Align::Center),
            StyleProp::AlignSelf(align),
            StyleProp::Position(Position::Relative),
            StyleProp::Size(SizeProperty::Width, width),
            StyleProp::Size(SizeProperty::Height, Dimension::Length(FIELD_HEIGHT)),
            StyleProp::Overflow(Overflow::Clip),
            StyleProp::Radius(Corners::All, radius),
            StyleProp::Shrink(0.0),
            StyleProp::Cursor(CursorIcon::Text),
            StyleProp::TextSize(TEXT_SIZE),
            StyleProp::FontFamily(INTER_FAMILY.to_owned()),
        ] {
            cx.set_default_style(node, prop);
        }
        cx.bind_style(node, move |runtime| {
            let disabled = disabled.get(runtime);
            StyleProp::TextColor(theme.with(runtime, |theme| {
                if disabled {
                    theme.text.disabled
                } else {
                    theme.popover.text
                }
            }))
        });

        let ring = move |runtime: &mut reactive::Runtime| {
            if focused.get(runtime) && !disabled.get(runtime) {
                1.0
            } else {
                0.0
            }
        };
        let mut layers = mount_chrome(cx, theme, radius, ring).to_vec();

        let content = mount_content(
            cx,
            ContentInput {
                theme,
                editor,
                focused,
                disabled,
                placeholder: self.placeholder,
                inset_left,
                inset_right,
            },
        );
        layers.push(content.viewport);

        if let Some(body) = self.leading_icon {
            let glyph = tinted_icon(
                body,
                ICON_STROKE,
                bind(move |runtime| {
                    let disabled = disabled.get(runtime);
                    theme.with(runtime, |theme| {
                        if disabled {
                            theme.text.disabled
                        } else {
                            theme.popover.muted_text
                        }
                    })
                }),
            )
            .size(LEADING_ICON)
            .absolute()
            .left(H_PADDING)
            .top((FIELD_HEIGHT - LEADING_ICON) / 2.0);
            layers.push(cx.build(glyph));
        }
        let clearable = self.clear_button == ClearButton::Visible;
        if clearable {
            let glyph = tinted_icon(
                icons::CIRCLE_X,
                ICON_STROKE,
                bind(move |runtime| theme.with(runtime, |theme| theme.popover.muted_text)),
            )
            .size(CLEAR_ICON)
            .absolute()
            .right(H_PADDING)
            .top((FIELD_HEIGHT - CLEAR_ICON) / 2.0)
            .hidden(bind(move |runtime| {
                disabled.get(runtime) || value.with(runtime, String::is_empty)
            }));
            layers.push(cx.build(glyph));
        }
        cx.adopt_children(node, layers);

        let Content {
            row_size, scroll, ..
        } = content;
        let fraction_of = move |cx: &mut Cx, position: Point| {
            let content_width = row_size.get(cx).width;
            let scroll = scroll.get(cx);
            fraction_along(position, inset_left, scroll, content_width)
        };
        cx.add_listener(
            node,
            Listener::KeyDown(Box::new(move |cx, event| editor.key_down(cx, event))),
        );
        cx.add_listener(
            node,
            Listener::TextInput(Box::new(move |cx, event| editor.text_input(cx, event))),
        );
        cx.add_listener(
            node,
            Listener::Blur(Box::new(move |cx, _| editor.commit_change(cx))),
        );
        cx.add_listener(
            node,
            Listener::Click(Box::new(move |cx, event| {
                let in_clear_button =
                    event.position.x >= size.get(cx).width - inset_right && clearable;
                let filled = value.with(cx, |text| !text.is_empty());
                if in_clear_button && filled {
                    editor.clear(cx);
                } else {
                    let fraction = fraction_of(cx, event.position);
                    editor.place_caret(cx, fraction, event.count);
                }
            })),
        );
        cx.add_listener(
            node,
            Listener::DragStart(Box::new(move |cx, event| {
                let fraction = fraction_of(cx, event.position);
                editor.place_caret(cx, fraction, 1);
            })),
        );
        cx.add_listener(
            node,
            Listener::Drag(Box::new(move |cx, event| {
                let fraction = fraction_of(cx, event.position);
                editor.drag_to(cx, fraction);
            })),
        );
    }
}

/// An editable single-line field bound to `value`. Listen with `.on_input` (every edit),
/// `.on_change` (blur or Enter) and `.on_submit` (Enter); shape it with
/// [`TextInputOptions`](super::TextInputOptions).
pub fn text_input(value: Signal<String>) -> Element<TextInputKind> {
    Element::new(TextInputKind::new(value)).label(String::new())
}
