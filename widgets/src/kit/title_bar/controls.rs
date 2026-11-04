use layouting::{Align, Justify};
use semantics::Role;
use view::{Cx, Element, Kind, Listener, NodeId, Semantic, StyleProp, Styled, bind};

use crate::{
    kit::{icon::tinted_icon, icons, use_theme},
    stack::{StackKind, hstack, vstack},
    util::{interaction::Interaction, themed::themed},
};

const DISC_DIAMETER: f32 = 16.0;
/// Each disc's hit target; the 6px around a disc, doubled between neighbours and added to the
/// sidebar layout's 6px padding at the window's top and trailing edges, spaces every disc 12px
/// from the next and from both edges.
const SLOT_SIZE: f32 = 28.0;
const GLYPH_EXTENT: f32 = 6.7;
const GLYPH_STROKE: f32 = 1.44;
const VIEW_BOX: f32 = 24.0;

/// One of the buttons a client-drawn title bar offers the window manager.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowControl {
    /// Maximizes the window, or restores it.
    Maximize,
    /// Minimizes the window.
    Minimize,
    /// Asks to close the window.
    Close,
}

impl WindowControl {
    /// The Lucide glyph and how much of the 24-unit view box its strokes span.
    const fn glyph(self) -> (&'static str, f32) {
        match self {
            Self::Maximize => (icons::SQUARE, 18.0),
            Self::Minimize => (icons::MINUS, 14.0),
            Self::Close => (icons::X, 12.0),
        }
    }

    const fn label(self) -> &'static str {
        match self {
            Self::Maximize => "Maximize",
            Self::Minimize => "Minimize",
            Self::Close => "Close",
        }
    }

    fn press(self, cx: &mut Cx) {
        match self {
            Self::Maximize => cx.toggle_maximize_window(),
            Self::Minimize => cx.minimize_window(),
            Self::Close => cx.request_close(),
        }
    }
}

/// The kind behind each of [`window_controls`]' buttons.
pub struct WindowControlKind {
    control: WindowControl,
}

impl Kind for WindowControlKind {
    const NAME: &'static str = "kit_window_control";

    fn mount(self, cx: &mut Cx, node: NodeId) {
        let theme = use_theme(cx);
        let interaction = Interaction::track(cx, node);
        let control = self.control;
        cx.set_role(node, Role::Button);
        for prop in [
            StyleProp::AlignItems(Align::Center),
            StyleProp::Justify(Justify::Center),
            StyleProp::Shrink(0.0),
        ] {
            cx.set_default_style(node, prop);
        }
        cx.add_listener(
            node,
            Listener::Click(Box::new(move |cx, _| control.press(cx))),
        );
        let (body, span) = control.glyph();
        let size = GLYPH_EXTENT * VIEW_BOX / span;
        let glyph = tinted_icon(
            body,
            GLYPH_STROKE * VIEW_BOX / size,
            themed(theme, |theme| theme.title_bar.control_glyph),
        )
        .size(size);
        let fill = bind(move |runtime| {
            let hovered = interaction.is_hovered(runtime);
            theme.with(runtime, |theme| {
                if hovered {
                    theme.title_bar.control_hover
                } else {
                    theme.title_bar.control
                }
            })
        });
        let disc = vstack(glyph)
            .size(DISC_DIAMETER)
            .rounded(DISC_DIAMETER / 2.0)
            .items_center()
            .justify_center()
            .bg(fill);
        let disc = cx.build(disc);
        cx.adopt_children(node, vec![disc]);
    }
}

fn control_button(control: WindowControl) -> Element<WindowControlKind> {
    Element::new(WindowControlKind { control })
        .size(SLOT_SIZE)
        .label(control.label())
}

type Controls = Element<
    StackKind<(
        Element<WindowControlKind>,
        Element<WindowControlKind>,
        Element<WindowControlKind>,
    )>,
>;

/// Maximize, minimize and close, drawn and ordered like the compositor's own title bar, for a
/// window with client-side decorations to put in its toolbar.
pub fn window_controls() -> Controls {
    hstack((
        control_button(WindowControl::Maximize),
        control_button(WindowControl::Minimize),
        control_button(WindowControl::Close),
    ))
    .self_start()
}
