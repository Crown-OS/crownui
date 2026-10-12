mod kind;
mod options;
mod skin;

pub use kind::ButtonKind;
pub use options::ButtonOptions;
pub use skin::{ButtonSize, ButtonVariant};

use view::{Element, IntoProp};

/// A pill button showing `label`; attach behavior with `.on_click(..)` and shape it with
/// [`ButtonOptions`]. Hover and press are washes of the palette's own hover color.
pub fn button(label: impl IntoProp<String>) -> Element<ButtonKind> {
    Element::new(ButtonKind {
        label: label.into_prop(),
        variant: ButtonVariant::default(),
        size: ButtonSize::default(),
        icon: None,
        min_width: 0.0,
    })
}

/// A square, icon-only button; `label` is not drawn but names the button for assistive technology.
pub fn icon_button(body: &'static str, label: impl IntoProp<String>) -> Element<ButtonKind> {
    button(label)
        .leading_icon(body)
        .control_size(ButtonSize::Icon)
}
